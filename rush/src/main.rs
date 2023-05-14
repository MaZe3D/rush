#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

mod command_parser;
mod rush_pin_manager;
mod rush_wifi;

use core::str::from_utf8;

use embassy_executor::_export::StaticCell;

use esp32s3_hal::prelude::*;

use embassy_executor::{Executor, SpawnError};
use embedded_svc::wifi::{AccessPointConfiguration, Configuration};
use esp32s3_hal::clock::{ClockControl, CpuClock};
use esp32s3_hal::{embassy, peripherals::Peripherals, timer::TimerGroup, Rtc};
use esp32s3_hal::{Rng, IO};
use esp_backtrace as _;
use esp_println::logger::init_logger;

use embassy_time::{Duration, Timer};
use embedded_io::asynch::Write;
use esp_wifi::wifi::WifiDevice;

use crate::command_parser::Command;

use crate::command_parser::parse;
use crate::rush_pin_manager::RushPinManager;
use embassy_net::tcp::TcpSocket;
use embassy_net::{
    IpListenEndpoint, Stack,
};

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

#[entry]
fn main() -> ! {
    init_logger(log::LevelFilter::Info);
    log::info!("program started - setting up peripherals...");

    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::configure(system.clock_control, CpuClock::Clock240MHz).freeze();

    // disable watchdog timers
    let mut rtc = Rtc::new(peripherals.RTC_CNTL);
    rtc.swd.disable();
    rtc.rwdt.disable();

    // setup pins
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let pin_manager = rush_pin_manager::RushPinManager::new(io.pins);

    // initialize wifi
    let rush_wifi = rush_wifi::RushWifi::new(
        TimerGroup::new(peripherals.TIMG1, &clocks).timer0,
        Rng::new(peripherals.RNG),
        system.radio_clock_control,
        &clocks,
        peripherals.RADIO.split().0,
        &Configuration::AccessPoint(AccessPointConfiguration {
            ssid: "esp-wifi".into(),
            ..Default::default()
        }),
    );

    // setup embassy
    let embassy_timer = TimerGroup::new(peripherals.TIMG0, &clocks).timer0;
    embassy::init(&clocks, embassy_timer);

    log::info!("setup done - starting embassy executor...");

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        let wifi_stack = rush_wifi.start(&spawner);
        if let Err(SpawnError::Busy) = spawner.spawn(main_loop(wifi_stack, pin_manager)) {
            panic!("could not spawn embassy task: main_loop - seems like it is already running? this should not be possible...");
        }
    });
}

#[embassy_executor::task]
async fn main_loop(stack: &'static Stack<WifiDevice<'static>>, mut pin_manager: RushPinManager) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));
    socket.set_keep_alive(Some(embassy_net::SmolDuration::from_secs(3)));

    loop {
        log::info!("waiting for connection...");
        if let Err(e) = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 80,
            })
            .await
        {
            log::error!("socket.accept() failed: {:?}", e);
            panic!();
        }
        log::info!("connected!");

        let mut buffer = [0u8; 1024];
        let mut pos = 0;
        loop {
            match socket.read(&mut buffer[pos..]).await {
                Ok(0) => break, // EOF received -> close socket and wait for new one
                Err(embassy_net::tcp::Error::ConnectionReset) => {
                    log::error!("could not receive commands - connection reset");
                    break; // close socket and wait for new one
                }
                Ok(len) => {
                    let buffer = &mut buffer[..pos + len]; // focus on filled part of buffer

                    if let Some(last_newline_index) =
                        buffer[pos..].iter().rposition(|x| *x == ('\n' as u8))
                    {
                        let last_newline_index = last_newline_index + pos;
                        let messages = buffer[..last_newline_index].split(|x| *x == ('\n' as u8));

                        if process_messages(messages, &mut socket, &mut pin_manager).await
                            == Err(embassy_net::tcp::Error::ConnectionReset)
                        {
                            log::error!("could not send response - connection reset");
                            break;
                        }

                        // copy remaining bytes to the front - these are the start of the next command
                        buffer.copy_within(
                            last_newline_index + 1..,
                            buffer.len() - last_newline_index - 1,
                        );
                        pos = buffer.len() - last_newline_index - 1;
                    } else {
                        pos += len;
                    }
                }
            };
        }
        socket.close();
        Timer::after(Duration::from_millis(1000)).await;
        socket.abort();

        log::info!("disconnected!");
    }
}

async fn process_messages<'a, I>(
    messages: I,
    socket: &mut TcpSocket<'_>,
    pin_manager: &mut RushPinManager,
) -> Result<(), embassy_net::tcp::Error>
where
    I: Iterator<Item = &'a [u8]>,
{
    for message in messages {
        let mut fmt_buffer = [0u8; 1024];
        let response_string = match from_utf8(message) {
            Err(_) => "could not parse command - conversion to utf8 failed\n",
            Ok(msg_as_str) => match parse(msg_as_str) {
                Err(_) => "invalid command\n",
                Ok((_, parsed_command)) => parsed_command.execute(&mut fmt_buffer, pin_manager),
            },
        };

        socket.write_all(response_string.as_bytes()).await?;
    }

    socket.flush().await?;
    Ok(())
}

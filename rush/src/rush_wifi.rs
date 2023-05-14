use core::str::from_utf8;

use embassy_executor::Spawner;
use embassy_executor::_export::StaticCell;
use embassy_net::tcp::TcpSocket;
use embassy_net::{
    Config, IpListenEndpoint, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfig,
};

use embassy_time::{Duration, Timer};
use embedded_io::asynch::Write;
use embedded_svc::wifi::{Configuration, Wifi};
use esp32s3_hal::clock::Clocks;
use esp32s3_hal::peripherals::TIMG1;
use esp32s3_hal::system::RadioClockControl;
use esp32s3_hal::Rng;
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{WifiController, WifiDevice, WifiMode};

use crate::command_parser::Command;

use crate::command_parser::parse;
use crate::rush_pin_manager::RushPinManager;

static STACK_RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
static NETWORK_STACK: StaticCell<Stack<WifiDevice>> = StaticCell::new();

pub struct RushWifi {
    wifi_controller: WifiController<'static>,
    network_stack: &'static Stack<WifiDevice<'static>>,
}

impl RushWifi {
    pub fn new(
        timer_timg1_timer0: esp32s3_hal::Timer<esp32s3_hal::timer::Timer0<TIMG1>>,
        mut rng: Rng,
        radio_clock_control: RadioClockControl,
        clocks: &Clocks,
        wifi_peripheral: esp32s3_hal::radio::Wifi,
        wifi_configuration: &Configuration,
    ) -> Self {
        // generate random network stack seed (before moving rng) as it is
        // "strongly recommended" for it to change for each boot
        let random_seed = (rng.random() as u64) << 32 | (rng.random() as u64);

        // esp-wifi setup - more or less taken from https://github.com/esp-rs/esp-wifi/ - esp32s3 - embassy access point example
        esp_wifi::initialize(timer_timg1_timer0, rng, radio_clock_control, &clocks).unwrap();

        let (wifi_interface, mut wifi_controller) =
            esp_wifi::wifi::new_with_mode(wifi_peripheral, WifiMode::Ap);
        wifi_controller
            .set_configuration(wifi_configuration)
            .unwrap();

        let config = Config::Static(StaticConfig {
            address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 2, 1), 24),
            gateway: Some(Ipv4Address::new(192, 168, 2, 1)),
            dns_servers: Default::default(),
        });

        // initialize network stack
        let network_stack = &*NETWORK_STACK.init(Stack::new(
            wifi_interface,
            config,
            STACK_RESOURCES.init(StackResources::new()),
            random_seed,
        ));

        RushWifi {
            wifi_controller,
            network_stack,
        }
    }

    pub fn start(self, embassy_spawner: &Spawner, pin_manager: RushPinManager) {
        embassy_spawner
            .spawn(run_wifi(self.wifi_controller, &self.network_stack))
            .ok();
        embassy_spawner
            .spawn(task(&self.network_stack, pin_manager))
            .ok();
    }
}

#[embassy_executor::task]
async fn run_wifi(
    mut controller: WifiController<'static>,
    network_stack: &'static Stack<WifiDevice<'static>>,
) -> ! {
    println!("start connection task");
    println!("Device capabilities: {:?}", controller.get_capabilities());
    println!("Starting wifi");

    controller.start().await.unwrap();

    println!("Wifi started!");

    network_stack.run().await
}

#[embassy_executor::task]
async fn task(stack: &'static Stack<WifiDevice<'static>>, mut pin_manager: RushPinManager) {
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
            Err(_) => "could not parse command - utf8 conversion failed\n",
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

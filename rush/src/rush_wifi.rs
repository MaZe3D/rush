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
use esp_println::{print, println};
use esp_wifi::wifi::{WifiController, WifiDevice, WifiMode};

use stackfmt::fmt_truncate;

use crate::command_parser::parse;

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

    pub fn start(self, embassy_spawner: &Spawner) {
        embassy_spawner
            .spawn(run_wifi(self.wifi_controller, &self.network_stack))
            .ok();
        embassy_spawner.spawn(task(&self.network_stack)).ok();
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
async fn task(stack: &'static Stack<WifiDevice<'static>>) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }
    println!("Connect to the AP `esp-wifi` and point your browser to http://192.168.2.1:8080/");
    println!("Use a static IP in the range 192.168.2.2 .. 192.168.2.255, use gateway 192.168.2.1");

    let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_net::SmolDuration::from_secs(10)));
    socket.set_keep_alive(Some(embassy_net::SmolDuration::from_secs(3)));
    loop {
        println!("Wait for connection...");
        let r = socket
            .accept(IpListenEndpoint {
                addr: None,
                port: 80,
            })
            .await;
        println!("Connected...");

        if let Err(e) = r {
            println!("connect error: {:?}", e);
            Timer::after(Duration::from_millis(1000)).await;
            socket.close();
            Timer::after(Duration::from_millis(1000)).await;
            socket.abort();
            continue;
        }

        loop {
            let mut buffer = [0u8; 1024];
            match recieve(&mut socket, &mut buffer).await {
                Ok(n) => {
                    println!("recieved {} bytes", n);
                    let message = core::str::from_utf8(&buffer[..n]).unwrap();

                    socket.write_all(message.as_bytes()).await.unwrap();
                    let r = socket.flush().await;
                    if let Err(e) = r {
                        println!("flush error: {:?}", e);
                    }

                    println!("recieved message: {}", message);
                    match parse(message) {
                        Ok(parsed_command) => {
                            let mut buf = [0u8; 1024];
                            let msg = fmt_truncate(
                                &mut buf,
                                format_args!("parsed command: {:?}\n", parsed_command.1),
                            );
                            socket.write_all(msg.as_bytes()).await.unwrap();
                            let r = socket.flush().await;
                            if let Err(e) = r {
                                println!("flush error: {:?}", e);
                            }
                            println!("sending message: {}", msg);
                        }
                        Err(e) => {
                            println!("parse error: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("recieve error: {:?}", e);
                    break;
                }
            }
        }
        Timer::after(Duration::from_millis(1000)).await;
        socket.close();
        Timer::after(Duration::from_millis(1000)).await;
        socket.abort();
    }
}

async fn recieve(
    socket: &mut TcpSocket<'_>,
    buffer: &mut [u8],
) -> Result<usize, embassy_net::tcp::Error> {
    loop {
        match socket.read(&mut buffer[..]).await {
            Ok(0) => {
                println!("read EOF");
                continue;
            }
            Ok(len) => {
                return Ok(len);
            }
            Err(e) => {
                return Err(e);
            }
        };
    }
}

async fn send(socket: &mut TcpSocket<'_>, message: &str) {
    match socket.write_all(message.as_bytes()).await {
        Ok(_) => {}
        Err(e) => {
            println!("write error: {:?}", e);
        }
    };
    let r = socket.flush().await;
    if let Err(e) = r {
        println!("flush error: {:?}", e);
    }
}

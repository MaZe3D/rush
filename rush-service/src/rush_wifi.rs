use embassy_executor::_export::StaticCell;
use embassy_executor::{SpawnError, Spawner};
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, Stack, StackResources, StaticConfig};
use embedded_svc::wifi::{Configuration, Wifi};
use esp32s3_hal::clock::Clocks;
use esp32s3_hal::peripherals::TIMG1;
use esp32s3_hal::system::RadioClockControl;
use esp32s3_hal::Rng;
use esp_backtrace as _;
use esp_wifi::wifi::{WifiController, WifiDevice, WifiMode};

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
        if let Err(e) = esp_wifi::initialize(timer_timg1_timer0, rng, radio_clock_control, &clocks)
        {
            panic!("esp_wifi::initialize failed: {:?}", e);
        }

        let (wifi_interface, mut wifi_controller) =
            esp_wifi::wifi::new_with_mode(wifi_peripheral, WifiMode::Ap);
        if let Err(e) = wifi_controller.set_configuration(wifi_configuration) {
            panic!(
                "esp_wifi::wifi::WifiController.set_configuration failed: {:?}",
                e
            );
        }

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

    pub fn start(self, embassy_spawner: &Spawner) -> &'static Stack<WifiDevice<'static>> {
        if let Err(SpawnError::Busy) =
            embassy_spawner.spawn(run_wifi(self.wifi_controller, &self.network_stack))
        {
            panic!("could not spawn embassy task: run_wifi - seems like it is already running? this should not be possible...");
        }
        self.network_stack
    }
}

#[embassy_executor::task]
async fn run_wifi(
    mut controller: WifiController<'static>,
    network_stack: &'static Stack<WifiDevice<'static>>,
) -> ! {
    log::info!("starting wifi");
    if let Err(e) = controller.start().await {
        panic!(
            "esp_wifi::wifi::asynch::WifiController.start() failed: {:?}",
            e
        );
    }
    log::info!("wifi started!");
    network_stack.run().await
}

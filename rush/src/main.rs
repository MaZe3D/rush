#![no_std]
#![no_main]
#![feature(c_variadic)]
#![feature(const_mut_refs)]
#![feature(type_alias_impl_trait)]
#![feature(error_in_core)]

mod command_parser;
mod rush_wifi;

use embassy_executor::_export::StaticCell;

use embassy_executor::Executor;
use embedded_svc::wifi::{AccessPointConfiguration, Configuration};
use esp32s3_hal::clock::{ClockControl, CpuClock};
use esp32s3_hal::Rng;
use esp32s3_hal::{embassy, peripherals::Peripherals, prelude::*, timer::TimerGroup, Rtc};
use esp_backtrace as _;
use esp_println::logger::init_logger;

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
    executor.run(|spawner| rush_wifi.start(&spawner));
}

use esp32s3_hal::{clock::ClockControl, prelude::*, timer::TimerGroup, Rtc};

use esp_backtrace as _;
use esp_println::println;
trait IoOperations {
    fn set_gpio_mode(&self, gpio_id: u8, gpio_mode: GpioMode);
    fn write_pin(&self, gpio_id: u8, value: bool);
    fn read_pin(&self, gpio_id: u8) -> bool;
}

pub enum GpioMode {
    Input,
    Output,
}

struct GpioManager {
    pins: esp32s3_hal::soc::gpio::Pins,
}

/* impl GpioManager {
    pub fn init(&self, io_input:IO) {
        &self.io = io_input;
    }
} */

pub fn read_pin(pins: esp32s3_hal::soc::gpio::Pins, gpio_id: u8) -> bool {
    let mut pin =
        match gpio_id {
        0 => pins.gpio0,
        1 => pins.gpio1.into_pull_up_input(),
        2 => pins.gpio2,
        3 => pins.gpio3,
        4 => pins.gpio4,
        5 => pins.gpio5,
        6 => pins.gpio6,
        7 => pins.gpio7,

        _ => println!("Invalid gpio id"),
    }

    return result;
}

use alloc::boxed::Box;
use esp32s3_hal::prelude::*;

use esp_backtrace as _;

use core::convert::Infallible;
use core::fmt::Debug;

use core::error::Error;

use alloc::vec::Vec;

//trait IoOperations {
//    fn set_gpio_mode(&self, gpio_id: u8, gpio_mode: GpioMode);
//    fn write_pin(&self, gpio_id: u8, value: bool);
//    fn read_pin(&self, gpio_id: u8) -> bool;
//}
//
//pub enum GpioMode {
//    Input,
//    Output,
//}

pub struct GpioManager {
    pub rush_input_pins: Vec<Box<dyn _embedded_hal_digital_v2_InputPin<Error = Infallible>>>,
}

impl GpioManager {
    pub fn new(pins: esp32s3_hal::soc::gpio::Pins) -> Self {
        let mut a = GpioManager {
            rush_input_pins: Vec::new(),
        };

        // get all 48 GPIO pins
        a.rush_input_pins
            .push(Box::new(pins.gpio35.into_pull_up_input()));
        a
    }
}

pub trait PinError: Debug {
    fn description(&self) -> &str {
        "Error"
    }
    fn cause(&self) -> Option<&dyn Error> {
        None
    }
    fn source(&self) {
        ()
    }
}

/* impl GpioManager {
    pub fn init(&self, io_input:IO) {
        &self.io = io_input;
    }
} */

//pub fn write_pin(pins: esp32s3_hal::soc::gpio::Pins, gpio_id: u8) -> bool {
//    return false;
//}

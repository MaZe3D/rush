use alloc::boxed::Box;
use esp32s3_hal::{clock::ClockControl, prelude::*, timer::TimerGroup, Rtc};
use esp32s3_hal::gpio::*;

use esp_backtrace as _;
use esp_println::println;

use esp32s3_hal::prelude::*;
use core::convert::Infallible;
use core::fmt::Debug;
use core::{pin, result};

use enum_dispatch::enum_dispatch;

use core::error::Error;

use alloc::vec::{Vec, self};


trait IoOperations {
    fn write_pin(&self, gpio_id: u8, value: bool);
    fn read_pin(&self, gpio_id: u8) -> bool;
}
pub struct GpioManager {
    pub pins: Vec<GpioPin>,
}


pub enum GpioPin {
    Undefined,
    Input(Box<dyn _embedded_hal_digital_v2_InputPin<Error = Infallible>>),
    Output(Box<dyn _embedded_hal_digital_v2_OutputPin<Error = Infallible>>),
}

impl IoOperations for GpioPin {
    fn read_pin(&self, gpio_id: u8) -> bool {
        result = false;
        match self {
            GpioPin::Input(pin) => {
                result = pin.is_high().unwrap()
            }
            GpioPin::Output(pin) => {
                println!("Pin {} is not an input pin", gpio_id);
            }
            _ => {
                println!("Pin {} is not defined", gpio_id);
            }
        }

        result
    }

    fn write_pin(&self, gpio_id: u8, value: bool) {
        match self {
            GpioPin::Output(pin) => {
                if value {
                    pin.set_high().unwrap();
                } else {
                    pin.set_low().unwrap();
                }
            }
            GpioPin::Input(pin) => {
                println!("Pin {} is not an output pin", gpio_id);
            }
            _ => {
                println!("Pin {} is not defined", gpio_id);
            }
        }
    }
}

//Pin Access Error
#[derive(Debug)]
trait PinAccessError: Error + Debug {
    
}

impl GpioManager {
    pub fn new(pins: esp32s3_hal::soc::gpio::Pins) -> Self {
        let mut a = GpioManager {
            pins: Vec::new(),
        };

        // init vector with 48 unknown pins
        for i in 0..=48 {
            a.pins.push(GpioPin::Undefined);
        }

        //set read pins
        a.pins[ 0] = GpioPin::Input(Box::new(pins.gpio0 .into_pull_up_input()));
        a.pins[21] = GpioPin::Input(Box::new(pins.gpio21.into_pull_up_input()));
        a.pins[ 5] = GpioPin::Input(Box::new(pins.gpio5 .into_pull_up_input()));
        a.pins[ 4] = GpioPin::Input(Box::new(pins.gpio4 .into_pull_up_input()));
        a.pins[ 3] = GpioPin::Input(Box::new(pins.gpio3 .into_pull_up_input()));
        a.pins[ 2] = GpioPin::Input(Box::new(pins.gpio2 .into_pull_up_input()));
        a.pins[ 1] = GpioPin::Input(Box::new(pins.gpio1 .into_pull_up_input()));

        //set write pins 35, 37, 36, 34 9 8 7 6
        a.pins[35] = GpioPin::Output(Box::new(pins.gpio35.into_push_pull_output()));
        a.pins[37] = GpioPin::Output(Box::new(pins.gpio37.into_push_pull_output()));
        a.pins[36] = GpioPin::Output(Box::new(pins.gpio36.into_push_pull_output()));
        a.pins[34] = GpioPin::Output(Box::new(pins.gpio34.into_push_pull_output()));
        a.pins[ 9] = GpioPin::Output(Box::new(pins.gpio9 .into_push_pull_output()));
        a.pins[ 8] = GpioPin::Output(Box::new(pins.gpio8 .into_push_pull_output()));
        a.pins[ 7] = GpioPin::Output(Box::new(pins.gpio7 .into_push_pull_output()));
        a.pins[ 6] = GpioPin::Output(Box::new(pins.gpio6 .into_push_pull_output()));

        a
    }
}
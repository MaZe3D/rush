use embassy_time::{Duration, Timer};
use enum_dispatch::enum_dispatch;
use esp32s3_hal;
use esp32s3_hal::ehal::digital::v2::PinState;
use esp32s3_hal::gpio;
use esp32s3_hal::prelude::_embedded_hal_digital_v2_OutputPin;
use esp32s3_hal::prelude::eh1::_embedded_hal_digital_blocking_InputPin;
use stackfmt::fmt_truncate;

struct PinManagerCompoundPin {
    pin: Option<RushAnyPin>,
    last_state_if_watched: Option<bool>,
}

pub struct RushPinManager {
    pins: [PinManagerCompoundPin; 49],
    none_pin: Option<RushAnyPin>, // used inside get_pin() if index is out of bounds
    next_pin_to_poll: u8,
}

impl RushPinManager {
    #[rustfmt::skip]
    pub fn new(pins: esp32s3_hal::soc::gpio::Pins) -> Self {
        let mut pin_array = [(); 49].map(|_| PinManagerCompoundPin { pin: Option::<RushAnyPin>::None, last_state_if_watched: Option::None} );

        pin_array[ 0].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio0 ).into());
        pin_array[ 1].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio1 ).into());
        pin_array[ 2].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio2 ).into());
        pin_array[ 3].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio3 ).into());
        pin_array[ 4].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio4 ).into());
        pin_array[ 5].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio5 ).into());
        pin_array[ 6].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio6 ).into());
        pin_array[ 7].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio7 ).into());
        pin_array[ 8].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio8 ).into());
        pin_array[ 9].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio9 ).into());
        pin_array[10].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio10).into());
        pin_array[11].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio11).into());
        pin_array[12].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio12).into());
        pin_array[13].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio13).into());
        pin_array[14].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio14).into());
        pin_array[15].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio15).into());
        pin_array[16].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio16).into());
        pin_array[17].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio17).into());
        pin_array[18].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio18).into());
        pin_array[19].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio19).into());
        pin_array[20].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio20).into());
        pin_array[21].pin = Some(RushSinglePin::UnknownAnalogPin(pins.gpio21).into());
        // pins 22, 23, 24 and 25 just don't exist
        pin_array[26].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio26).into());
        pin_array[27].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio27).into());
        pin_array[28].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio28).into());
        pin_array[29].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio29).into());
        pin_array[30].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio30).into());
        pin_array[31].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio31).into());
        pin_array[32].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio32).into());
        pin_array[33].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio33).into());
        pin_array[34].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio34).into());
        pin_array[35].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio35).into());
        pin_array[36].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio36).into());
        pin_array[37].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio37).into());
        pin_array[38].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio38).into());
        pin_array[39].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio39).into());
        pin_array[40].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio40).into());
        pin_array[41].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio41).into());
        pin_array[42].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio42).into());
        pin_array[43].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio43).into());
        pin_array[44].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio44).into());
        pin_array[45].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio45).into());
        pin_array[46].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio46).into());
        pin_array[47].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio47).into());
        pin_array[48].pin = Some(RushSinglePin::UnknownDigitalPin(pins.gpio48).into());

        RushPinManager { pins: pin_array, none_pin: Option::<RushAnyPin>::None, next_pin_to_poll: 0 }
    }

    pub fn get_pin(&mut self, pin: u8) -> &mut Option<RushAnyPin> {
        let pin = pin as usize;
        if pin < self.pins.len() {
            return &mut self.pins[pin].pin;
        }
        &mut self.none_pin
    }

    pub fn watch_pin<'a, 'b>(&'a mut self, pin: u8) -> Result<bool, &'b str> {
        let current_state = self.get_pin(pin).to_input().read_state()?;
        self.pins[pin as usize].last_state_if_watched = Some(current_state);
        Ok(current_state)
    }

    pub fn unwatch_pin<'a, 'b>(&'a mut self, pin: u8) -> Result<(), &'b str> {
        match self.get_pin(pin) {
            None => Err("pin does not exist"),
            Some(_) => {
                self.pins[pin as usize].last_state_if_watched = None;
                Ok(())
            }
        }
    }

    pub async fn poll_watched_pins<'a, 'b>(&'a mut self, fmt_buffer: &'b mut [u8]) -> &'b str {
        loop {
            let pin_num = self.next_pin_to_poll;
            self.next_pin_to_poll += 1;
            self.next_pin_to_poll %= self.pins.len() as u8;

            let pin = &mut self.pins[pin_num as usize];
            if let Some(laststate) = pin.last_state_if_watched {
                match pin.pin.read_state() {
                    Err(e) => {
                        pin.last_state_if_watched = None;
                        return fmt_truncate(
                            fmt_buffer,
                            format_args!("{}\n  => stopped watching gpio.{}\n", e, pin_num),
                        );
                    }
                    Ok(state) => {
                        if state != laststate {
                            pin.last_state_if_watched = Some(state);
                            return fmt_truncate(
                                fmt_buffer,
                                format_args!("gpio.{} = {}\n", pin_num, state as u8),
                            );
                        }
                    }
                };
            }

            if pin_num == 0 {
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}

pub trait RushPinOperations {
    fn to_input(&mut self) -> &mut Self;
    fn to_output(&mut self) -> &mut Self;
    fn read_state<'a, 'b>(&'a mut self) -> Result<bool, &'b str>;
    fn set_state<'a, 'b>(&'a mut self, state: PinState) -> Result<(), &'b str>;
}

impl RushPinOperations for Option<RushAnyPin> {
    fn to_input(&mut self) -> &mut Self {
        match self.take() {
            None => (),
            Some(pin) => {
                self.replace(pin.to_input());
            }
        };
        self
    }
    fn to_output(&mut self) -> &mut Self {
        match self.take() {
            None => (),
            Some(pin) => {
                self.replace(pin.to_output());
            }
        };
        self
    }
    fn read_state<'a, 'b>(&'a mut self) -> Result<bool, &'b str> {
        match self {
            None => Err("pin does not exist"),
            Some(pin) => Ok(pin.read_state()?),
        }
    }
    fn set_state<'a, 'b>(&'a mut self, state: PinState) -> Result<(), &'b str> {
        match self {
            None => Err("pin does not exist"),
            Some(pin) => Ok(pin.set_state(state)?),
        }
    }
}

#[rustfmt::skip]
#[enum_dispatch]
pub enum RushAnyPin {
    Pin0 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio0Signals,   0>),
    Pin1 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio1Signals,   1>),
    Pin2 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio2Signals,   2>),
    Pin3 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio3Signals,   3>),
    Pin4 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio4Signals,   4>),
    Pin5 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio5Signals,   5>),
    Pin6 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio6Signals,   6>),
    Pin7 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio7Signals,   7>),
    Pin8 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio8Signals,   8>),
    Pin9 (RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio9Signals,   9>),
    Pin10(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio10Signals, 10>),
    Pin11(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio11Signals, 11>),
    Pin12(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio12Signals, 12>),
    Pin13(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio13Signals, 13>),
    Pin14(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio14Signals, 14>),
    Pin15(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio15Signals, 15>),
    Pin16(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio16Signals, 16>),
    Pin17(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio17Signals, 17>),
    Pin18(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio18Signals, 18>),
    Pin19(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio19Signals, 19>),
    Pin20(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio20Signals, 20>),
    Pin21(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio21Signals, 21>),
    // pins 22, 23, 24 and 25 just don't exist
    Pin26(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio26Signals, 26>),
    Pin27(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio27Signals, 27>),
    Pin28(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio28Signals, 28>),
    Pin29(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio29Signals, 29>),
    Pin30(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio30Signals, 30>),
    Pin31(RushSinglePin<gpio::Bank0GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank0, gpio::Gpio31Signals, 31>),
    Pin32(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio32Signals, 32>),
    Pin33(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio33Signals, 33>),
    Pin34(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio34Signals, 34>),
    Pin35(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio35Signals, 35>),
    Pin36(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio36Signals, 36>),
    Pin37(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio37Signals, 37>),
    Pin38(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio38Signals, 38>),
    Pin39(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio39Signals, 39>),
    Pin40(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio40Signals, 40>),
    Pin41(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio41Signals, 41>),
    Pin42(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio42Signals, 42>),
    Pin43(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio43Signals, 43>),
    Pin44(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio44Signals, 44>),
    Pin45(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio45Signals, 45>),
    Pin46(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio46Signals, 46>),
    Pin47(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio47Signals, 47>),
    Pin48(RushSinglePin<gpio::Bank1GpioRegisterAccess, gpio::SingleCoreInteruptStatusRegisterAccessBank1, gpio::Gpio48Signals, 48>),
}

#[rustfmt::skip]
pub enum RushSinglePin<RA, IRA, SIG, const GPIONUM: u8>
where
    RA: gpio::BankGpioRegisterAccess,
    IRA: gpio::InteruptStatusRegisterAccess,
    SIG: gpio::GpioSignal,
{
    UnknownAnalogPin (gpio::GpioPin<gpio::Unknown               , RA, IRA, gpio::InputOutputAnalogPinType, SIG, GPIONUM>),
    InputAnalogPin   (gpio::GpioPin<gpio::Input<gpio::Floating> , RA, IRA, gpio::InputOutputAnalogPinType, SIG, GPIONUM>),
    OutputAnalogPin  (gpio::GpioPin<gpio::Output<gpio::PushPull>, RA, IRA, gpio::InputOutputAnalogPinType, SIG, GPIONUM>),
    UnknownDigitalPin(gpio::GpioPin<gpio::Unknown               , RA, IRA, gpio::InputOutputPinType      , SIG, GPIONUM>),
    InputDigitalPin  (gpio::GpioPin<gpio::Input<gpio::Floating> , RA, IRA, gpio::InputOutputPinType      , SIG, GPIONUM>),
    OutputDigitalPin (gpio::GpioPin<gpio::Output<gpio::PushPull>, RA, IRA, gpio::InputOutputPinType      , SIG, GPIONUM>),
}

#[enum_dispatch(RushAnyPin)]
trait RushSinglePinOperations {
    fn to_input(self) -> Self;
    fn to_output(self) -> Self;
    fn read_state<'a, 'b>(&'a self) -> Result<bool, &'b str>;
    fn set_state<'a, 'b>(&'a mut self, state: PinState) -> Result<(), &'b str>;
}

impl<RA, IRA, SIG, const GPIONUM: u8> RushSinglePinOperations
    for RushSinglePin<RA, IRA, SIG, GPIONUM>
where
    RA: gpio::BankGpioRegisterAccess,
    IRA: gpio::InteruptStatusRegisterAccess,
    SIG: gpio::GpioSignal,
{
    fn to_input(self) -> Self {
        match self {
            Self::OutputAnalogPin(p) => Self::InputAnalogPin(p.into_floating_input()),
            Self::UnknownAnalogPin(p) => Self::InputAnalogPin(p.into_floating_input()),
            Self::OutputDigitalPin(p) => Self::InputDigitalPin(p.into_floating_input()),
            Self::UnknownDigitalPin(p) => Self::InputDigitalPin(p.into_floating_input()),
            _ => self,
        }
    }
    fn to_output(self) -> Self {
        match self {
            Self::InputAnalogPin(p) => Self::OutputAnalogPin(p.into_push_pull_output()),
            Self::UnknownAnalogPin(p) => Self::OutputAnalogPin(p.into_push_pull_output()),
            Self::InputDigitalPin(p) => Self::OutputDigitalPin(p.into_push_pull_output()),
            Self::UnknownDigitalPin(p) => Self::OutputDigitalPin(p.into_push_pull_output()),
            _ => self,
        }
    }
    fn read_state<'a, 'b>(&'a self) -> Result<bool, &'b str> {
        match self {
            Self::InputAnalogPin(p) => match p.is_high() {
                Ok(b) => Ok(b),
                Err(_) => Err("esp_hal_common::gpio::GpioPin.is_high() failed"),
            },
            Self::InputDigitalPin(p) => match p.is_high() {
                Ok(b) => Ok(b),
                Err(_) => Err("esp_hal_common::gpio::GpioPin.is_high() failed"),
            },
            _ => Err("read_state() was called on a non-input pin"),
        }
    }
    fn set_state<'a, 'b>(&'a mut self, state: PinState) -> Result<(), &'b str> {
        match self {
            Self::OutputAnalogPin(p) => match p.set_state(state) {
                Ok(_) => Ok(()),
                Err(_) => Err("embedded_hal::digital::v2::OutputPin.set_state() failed"),
            },
            Self::OutputDigitalPin(p) => match p.set_state(state) {
                Ok(_) => Ok(()),
                Err(_) => Err("embedded_hal::digital::v2::OutputPin.set_state() failed"),
            },
            _ => Err("set_state() was called on a non-output pin"),
        }
    }
}

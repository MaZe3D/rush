use enum_dispatch::enum_dispatch;
use esp32s3_hal;
use esp32s3_hal::ehal::digital::v2::PinState;
use esp32s3_hal::gpio;
use esp32s3_hal::prelude::_embedded_hal_digital_v2_OutputPin;
use esp32s3_hal::prelude::eh1::_embedded_hal_digital_blocking_InputPin;

pub struct RushPinManager {
    pins: [Option<RushAnyPin>; 49],
}

impl RushPinManager {
    #[rustfmt::skip]
    pub fn new(pins: esp32s3_hal::soc::gpio::Pins) -> Self {
        let mut pin_array = [(); 49].map(|_| Option::<RushAnyPin>::None);

        pin_array[0]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio0 ).into());
        pin_array[1]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio1 ).into());
        pin_array[2]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio2 ).into());
        pin_array[3]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio3 ).into());
        pin_array[4]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio4 ).into());
        pin_array[5]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio5 ).into());
        pin_array[6]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio6 ).into());
        pin_array[7]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio7 ).into());
        pin_array[8]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio8 ).into());
        pin_array[9]  = Some(RushSinglePin::UnknownAnalogPin(pins.gpio9 ).into());
        pin_array[10] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio10).into());
        pin_array[11] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio11).into());
        pin_array[12] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio12).into());
        pin_array[13] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio13).into());
        pin_array[14] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio14).into());
        pin_array[15] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio15).into());
        pin_array[16] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio16).into());
        pin_array[17] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio17).into());
        pin_array[18] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio18).into());
        pin_array[19] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio19).into());
        pin_array[20] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio20).into());
        pin_array[21] = Some(RushSinglePin::UnknownAnalogPin(pins.gpio21).into());
        // pins 22, 23, 24 and 25 just don't exist
        pin_array[26] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio26).into());
        pin_array[27] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio27).into());
        pin_array[28] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio28).into());
        pin_array[29] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio29).into());
        pin_array[30] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio30).into());
        pin_array[31] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio31).into());
        pin_array[32] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio32).into());
        pin_array[33] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio33).into());
        pin_array[34] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio34).into());
        pin_array[35] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio35).into());
        pin_array[36] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio36).into());
        pin_array[37] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio37).into());
        pin_array[38] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio38).into());
        pin_array[39] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio39).into());
        pin_array[40] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio40).into());
        pin_array[41] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio41).into());
        pin_array[42] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio42).into());
        pin_array[43] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio43).into());
        pin_array[44] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio44).into());
        pin_array[45] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio45).into());
        pin_array[46] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio46).into());
        pin_array[47] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio47).into());
        pin_array[48] = Some(RushSinglePin::UnknownDigitalPin(pins.gpio48).into());

        RushPinManager { pins: pin_array }
    }

    pub fn get_pin(&mut self, pin: usize) -> &mut Option<RushAnyPin> {
        &mut self.pins[pin]
    }
}

pub trait RushPinOperations {
    fn to_input(&mut self) -> &mut Self;
    fn to_output(&mut self) -> &mut Self;
    fn read_state(&mut self) -> Result<bool, &str>;
    fn set_state(&mut self, state: PinState) -> Result<(), &str>;
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
    fn read_state(&mut self) -> Result<bool, &str> {
        self.to_input();
        match self.as_ref() {
            None => Err("pin does not exist"), // somehow throw an error?
            Some(pin) => Ok(pin.read_state()?),
        }
    }
    fn set_state(&mut self, state: PinState) -> Result<(), &str> {
        match self.as_mut() {
            None => Err("pin does not exist"), // somehow throw an error?
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
    InputAnalogPin   (gpio::GpioPin<gpio::Input<gpio::PullUp>   , RA, IRA, gpio::InputOutputAnalogPinType, SIG, GPIONUM>),
    OutputAnalogPin  (gpio::GpioPin<gpio::Output<gpio::PushPull>, RA, IRA, gpio::InputOutputAnalogPinType, SIG, GPIONUM>),
    UnknownDigitalPin(gpio::GpioPin<gpio::Unknown               , RA, IRA, gpio::InputOutputPinType      , SIG, GPIONUM>),
    InputDigitalPin  (gpio::GpioPin<gpio::Input<gpio::PullUp>   , RA, IRA, gpio::InputOutputPinType      , SIG, GPIONUM>),
    OutputDigitalPin (gpio::GpioPin<gpio::Output<gpio::PushPull>, RA, IRA, gpio::InputOutputPinType      , SIG, GPIONUM>),
}

#[enum_dispatch(RushAnyPin)]
trait RushSinglePinOperations {
    fn to_input(self) -> Self;
    fn to_output(self) -> Self;
    fn read_state(&self) -> Result<bool, &str>;
    fn set_state(&mut self, state: PinState) -> Result<(), &str>;
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
            Self::OutputAnalogPin(p) => Self::InputAnalogPin(p.into_pull_up_input()),
            Self::UnknownAnalogPin(p) => Self::InputAnalogPin(p.into_pull_up_input()),
            Self::OutputDigitalPin(p) => Self::InputDigitalPin(p.into_pull_up_input()),
            Self::UnknownDigitalPin(p) => Self::InputDigitalPin(p.into_pull_up_input()),
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
    fn read_state(&self) -> Result<bool, &str> {
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
    fn set_state(&mut self, state: PinState) -> Result<(), &str> {
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

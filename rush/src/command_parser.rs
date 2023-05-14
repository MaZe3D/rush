/*
alle 6 folgenden commands koennen auch mehrere [gpio/uart_channel/led] gleichzeitig nehmen
read    [gpio/uart/led]
watch   [gpio/uart/led]
unwatch [gpio/uart/led/nothing] -> 'nothing' to stop all watching

write   [gpio/uart/led] [value]
shout   [gpio/uart/led]         -> interpret any invalid command as if prefixed with "write [gpio/uart_channel/led]" (repeat for any shouted id)
unshout [gpio/uart/led/nothing] -> 'nothing' to stop all shouting

[gpio/uart_channel/led/nothing] sieht dann iwie so aus: gpio.17 / led.8 / uart.5
evtl aliases dafuer? i.e. cooleLampe = led.8 xDD


dann noch so irgednwas um leds konfigurieren zu koennen und so, des gfaellt mir abo noch net so guad
list [gpio/uart_channel/led]  -> print all available gpios
*/

use enum_dispatch::enum_dispatch;
use nom::IResult;
use stackfmt::fmt_truncate;

#[enum_dispatch]
#[derive(Debug)]
pub enum CommandEnum {
    Read(ReadCommand),
    Watch(WatchCommand),
    Unwatch(UnwatchCommand),
    Write(WriteCommand),
    /*     Shout(ShoutCommand),
    Unshout(UnshoutCommand),
    List(ListCommand), */
}

#[enum_dispatch(CommandEnum)]
pub trait Command {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8]) -> &'a str;
}

#[derive(Debug)]
pub struct ReadCommand {
    pub id: Id,
}
impl Command for ReadCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8]) -> &'a str {
        match self.id {
            Id::Gpio(gpio) => {
                fmt_truncate(fmt_buffer, format_args!("state of gpio.{}: xx\n", gpio))
            }
        }
    }
}

#[derive(Debug)]
pub struct WatchCommand {
    pub id: Id,
}
impl Command for WatchCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8]) -> &'a str {
        fmt_truncate(fmt_buffer, format_args!("Watch command: {:?}\n", self))
    }
}

#[derive(Debug)]
pub struct UnwatchCommand {
    pub id: Id,
}
impl Command for UnwatchCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8]) -> &'a str {
        fmt_truncate(fmt_buffer, format_args!("Unwatch command: {:?}\n", self))
    }
}

#[derive(Debug)]
pub struct WriteCommand {
    pub id: Id,
    pub value: Value,
}
impl Command for WriteCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8]) -> &'a str {
        match self.id {
            Id::Gpio(gpio) => {
                fmt_truncate(fmt_buffer, format_args!("writing xx to gpio.{}\n", gpio))
            }
        }
    }
}

#[derive(Debug)]
pub enum Id {
    Gpio(u8),
}

#[derive(Debug)]

pub enum Value {
    Gpio(bool),
}

pub fn parse(input: &str) -> IResult<&str, CommandEnum> {
    let (input, command) = nom::branch::alt((
        read_command_parser,
        watch_command_parser,
        unwatch_command_parser,
        write_command_parser,
        /* shout_command_parser,
        unshout_command_parser,
        list_command_parser, */
    ))(input)?;

    Ok((input, command))
}

fn read_command_parser(input: &str) -> IResult<&str, CommandEnum> {
    let (input, _) = nom::bytes::complete::tag("read")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, CommandEnum::Read(ReadCommand { id })))
}

fn watch_command_parser(input: &str) -> IResult<&str, CommandEnum> {
    let (input, _) = nom::bytes::complete::tag("watch")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, CommandEnum::Watch(WatchCommand { id })))
}

fn unwatch_command_parser(input: &str) -> IResult<&str, CommandEnum> {
    let (input, _) = nom::bytes::complete::tag("unwatch")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, CommandEnum::Unwatch(UnwatchCommand { id })))
}

fn write_command_parser(input: &str) -> IResult<&str, CommandEnum> {
    let (input, _) = nom::bytes::complete::tag("write")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, value) = value_parser(input)?;

    Ok((input, CommandEnum::Write(WriteCommand { id, value })))
}

fn id_parser(input: &str) -> IResult<&str, Id> {
    let (input, id) = gpio_id_parser(input)?;

    Ok((input, id))
}

fn gpio_id_parser(input: &str) -> IResult<&str, Id> {
    let (input, _) = nom::bytes::complete::tag("gpio")(input)?;
    let (input, _) = nom::character::complete::char('.')(input)?;
    let (input, id) = nom::character::complete::digit1(input)?;
    let id = id.parse::<u8>().unwrap();

    Ok((input, Id::Gpio(id)))
}

fn value_parser(input: &str) -> IResult<&str, Value> {
    let (input, value) = gpio_value_parser(input)?;

    Ok((input, value))
}

fn gpio_value_parser(input: &str) -> IResult<&str, Value> {
    let (input, value) =
        nom::branch::alt((gpio_value_true_parser, gpio_value_false_parser))(input)?;

    Ok((input, value))
}

fn gpio_value_true_parser(input: &str) -> IResult<&str, Value> {
    let (input, _) = nom::bytes::complete::tag("true")(input)?;

    Ok((input, Value::Gpio(true)))
}

fn gpio_value_false_parser(input: &str) -> IResult<&str, Value> {
    let (input, _) = nom::bytes::complete::tag("false")(input)?;

    Ok((input, Value::Gpio(false)))
}

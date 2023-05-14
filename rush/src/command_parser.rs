/*
Command Examples:
read    [gpio]
write   [gpio] [value]
watch   [gpio]
unwatch [gpio]

[gpio] is expressed by gpio.[pin]
[value] is expressed by true or false
*/

use enum_dispatch::enum_dispatch;
use esp32s3_hal::ehal::digital::v2::PinState;
use nom::IResult;
use stackfmt::fmt_truncate;

use crate::rush_pin_manager::{RushPinManager, RushPinOperations};

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
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8], pin_manager: &mut RushPinManager) -> &'a str;
}

#[derive(Debug)]
pub struct ReadCommand {
    pub id: Id,
}
impl Command for ReadCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8], pin_manager: &mut RushPinManager) -> &'a str {
        let Id::Gpio(pin) = self.id;
        match pin_manager.get_pin(pin as usize).to_input().read_state() {
            Ok(state) => fmt_truncate(
                fmt_buffer,
                format_args!("state of gpio.{}: {}\n", pin, state),
            ),
            Err(err) => fmt_truncate(
                fmt_buffer,
                format_args!("error: could not read state of gpio.{}: {}", pin, err),
            ),
        }
    }
}

#[derive(Debug)]
pub struct WatchCommand {
    pub id: Id,
}
impl Command for WatchCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8], _pin_manager: &mut RushPinManager) -> &'a str {
        fmt_truncate(fmt_buffer, format_args!("Watch command: {:?}\n", self))
    }
}

#[derive(Debug)]
pub struct UnwatchCommand {
    pub id: Id,
}
impl Command for UnwatchCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8], _pin_manager: &mut RushPinManager) -> &'a str {
        fmt_truncate(fmt_buffer, format_args!("Unwatch command: {:?}\n", self))
    }
}

#[derive(Debug)]
pub struct WriteCommand {
    pub id: Id,
    pub value: Value,
}
impl Command for WriteCommand {
    fn execute<'a>(&self, fmt_buffer: &'a mut [u8], pin_manager: &mut RushPinManager) -> &'a str {
        let (Id::Gpio(pin), Value::Gpio(b)) = (&self.id, &self.value);
        match pin_manager
            .get_pin(*pin as usize)
            .to_output()
            .set_state(if *b { PinState::High } else { PinState::Low })
        {
            Ok(_) => fmt_truncate(fmt_buffer, format_args!("writing {} to gpio.{}\n", b, pin)),
            Err(err) => fmt_truncate(
                fmt_buffer,
                format_args!("error: could not write to gpio.{}: {}\n", pin, err),
            ),
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
    //Allow high, on, h, 1, true, t
    let (input, _) = nom::branch::alt((
        nom::bytes::complete::tag("high"),
        nom::bytes::complete::tag("on"),
        nom::bytes::complete::tag("h"),
        nom::bytes::complete::tag("1"),
        nom::bytes::complete::tag("true"),
        nom::bytes::complete::tag("t"),
    ))(input)?;

    Ok(()).map(|_| (input, Value::Gpio(true)))
}

fn gpio_value_false_parser(input: &str) -> IResult<&str, Value> {
    //allow low, off, l, 0, false, f
    let (input, _) = nom::branch::alt((
        nom::bytes::complete::tag("low"),
        nom::bytes::complete::tag("off"),
        nom::bytes::complete::tag("l"),
        nom::bytes::complete::tag("0"),
        nom::bytes::complete::tag("false"),
        nom::bytes::complete::tag("f"),
    ))(input)?;

    Ok(()).map(|_| (input, Value::Gpio(false)))
}

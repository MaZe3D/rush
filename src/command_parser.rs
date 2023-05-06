

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

use nom::IResult;



#[derive(Debug)]
pub enum Command {
    Read(ReadCommand),
    Watch(WatchCommand),
    Unwatch(UnwatchCommand),
    Write(WriteCommand),
    Shout(ShoutCommand),
    Unshout(UnshoutCommand),
    List(ListCommand),
}

#[derive(Debug)]
pub struct ReadCommand {
    pub id: Id,
}

#[derive(Debug)]
pub struct WatchCommand {
    pub id: Id,
}

#[derive(Debug)]
pub struct UnwatchCommand {
    pub id: Id,
}

#[derive(Debug)]
pub struct WriteCommand {
    pub id: Id,
    pub value: Value,
}

#[derive(Debug)]
pub struct ShoutCommand {
    pub id: Id,
}

#[derive(Debug)]
pub struct UnshoutCommand {
    pub id: Id,
}

#[derive(Debug)]
pub struct ListCommand {
    pub id: Id,
}

#[derive(Debug)]
pub enum Id {
    Gpio(u8),
    Uart(u8),
    Led(u8),
}

#[derive(Debug)]

pub enum Value {
    Gpio(bool),
    Uart(&str),
    Led(u8, u8, u8),
}

pub fn parse_command(command: &str) -> Command {
    match command_parser(command.trim()) {
        Ok((_, command)) => command,
        Err(_) => panic!("Invalid command!"),
    }
}

fn command_parser(input: &str) -> IResult<&str, Command> {
    let (input, command) = nom::branch::alt((
        read_command_parser,
        watch_command_parser,
        unwatch_command_parser,
        write_command_parser,
        shout_command_parser,
        unshout_command_parser,
        list_command_parser,
    ))(input)?;

    Ok((input, command))
}

fn read_command_parser(input: &str) -> IResult<&str, Command> {
    let (input, _) = nom::bytes::complete::tag("read")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, Command::Read(ReadCommand { id })))
}

fn watch_command_parser(input: &str) -> IResult<&str, Command> {
    let (input, _) = nom::bytes::complete::tag("watch")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, Command::Watch(WatchCommand { id })))
}

fn unwatch_command_parser(input: &str) -> IResult<&str, Command> {
    let (input, _) = nom::bytes::complete::tag("unwatch")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, Command::Unwatch(UnwatchCommand { id })))
}

fn write_command_parser(input: &str) -> IResult<&str, Command> {
    let (input, _) = nom::bytes::complete::tag("write")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, value) = value_parser(input)?;

    Ok((input, Command::Write(WriteCommand { id, value })))
}

fn shout_command_parser(input: &str) -> IResult<&str, Command> {
    let (input, _) = nom::bytes::complete::tag("shout")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, Command::Shout(ShoutCommand { id })))
}

fn unshout_command_parser(input: &str) -> IResult<&str, Command> {
    let (input, _) = nom::bytes::complete::tag("unshout")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, Command::Unshout(UnshoutCommand { id })))
}

fn list_command_parser(input: &str) -> IResult<&str, Command> {
    let (input, _) = nom::bytes::complete::tag("list")(input)?;
    let (input, _) = nom::character::complete::space1(input)?;
    let (input, id) = id_parser(input)?;

    Ok((input, Command::List(ListCommand { id })))
}

fn id_parser(input: &str) -> IResult<&str, Id> {
    let (input, id) = nom::branch::alt((
        gpio_id_parser,
        uart_id_parser,
        led_id_parser,
    ))(input)?;

    Ok((input, id))
}

fn gpio_id_parser(input: &str) -> IResult<&str, Id> {
    let (input, _) = nom::bytes::complete::tag("gpio")(input)?;
    let (input, _) = nom::character::complete::char('.')(input)?;
    let (input, id) = nom::character::complete::digit1(input)?;
    let id = id.parse::<u8>().unwrap();

    Ok((input, Id::Gpio(id)))
}

fn uart_id_parser(input: &str) -> IResult<&str, Id> {
    let (input, _) = nom::bytes::complete::tag("uart")(input)?;
    let (input, _) = nom::character::complete::char('.')(input)?;
    let (input, id) = nom::character::complete::digit1(input)?;
    let id = id.parse::<u8>().unwrap();

    Ok((input, Id::Uart(id)))
}

fn led_id_parser(input: &str) -> IResult<&str, Id> {
    let (input, _) = nom::bytes::complete::tag("led")(input)?;
    let (input, _) = nom::character::complete::char('.')(input)?;
    let (input, id) = nom::character::complete::digit1(input)?;
    let id = id.parse::<u8>().unwrap();

    Ok((input, Id::Led(id)))
}

fn value_parser(input: &str) -> IResult<&str, Value> {
    let (input, value) = nom::branch::alt((
        gpio_value_parser,
        uart_value_parser,
        led_value_parser,
    ))(input)?;

    Ok((input, value))
}

fn gpio_value_parser(input: &str) -> IResult<&str, Value> {
    let (input, value) = nom::branch::alt((
        gpio_value_true_parser,
        gpio_value_false_parser,
    ))(input)?;

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

fn uart_value_parser(input: &str) -> IResult<&str, Value> {
    //extracts string within quotes

    let (input, _) = nom::character::complete::char('"')(input)?;
    let (input, value) = nom::bytes::complete::is_not("\"")(input)?;
    let (input, _) = nom::character::complete::char('"')(input)?;

    Ok((input, Value::Uart(value)))
}

fn led_value_parser(input: &str) -> IResult<&str, Value> {
    //Led Values are RGB values in Decimal with ' ' or as hex beginning with # like #FF00FF
    let (input, value) = nom::branch::alt((
        led_value_decimal_parser,
        led_value_hex_parser,
    ))(input)?;

    Ok((input, value))
}

fn led_value_decimal_parser(input: &str) -> IResult<&str, Value> {
    let (input, r) = nom::character::complete::digit1(input)?;
    let (input, _) = nom::character::complete::char(' ')(input)?;
    let (input, g) = nom::character::complete::digit1(input)?;
    let (input, _) = nom::character::complete::char(' ')(input)?;
    let (input, b) = nom::character::complete::digit1(input)?;

    let r = r.parse::<u8>().unwrap();
    let g = g.parse::<u8>().unwrap();
    let b = b.parse::<u8>().unwrap();

    Ok((input, Value::Led(r, g, b)))
}

fn led_value_hex_parser(input: &str) -> IResult<&str, Value> {
    let (input, _) = nom::character::complete::char('#')(input)?;
    //read only 2 characters
    let (r, input) = input.split_at(2);
    let (g, input) = input.split_at(2);
    let (b, _) = input.split_at(2);

    let r = u8::from_str_radix(r, 16).unwrap();
    let g = u8::from_str_radix(g, 16).unwrap();
    let b = u8::from_str_radix(b, 16).unwrap();

    Ok((input, Value::Led(r, g, b)))
}

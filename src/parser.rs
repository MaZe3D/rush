use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, digit1},
    combinator::{map, opt},
    sequence::{delimited, pair},
    IResult,
};

use esp_println::println;

fn ledcolor(input: &str) -> IResult<&str, (u8, u8, u8)> {
    let (input, _) = tag("ledcolor(")(input)?;
    let (input, r) = digit1(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, g) = digit1(input)?;
    let (input, _) = tag(",")(input)?;
    let (input, b) = digit1(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, (r.parse().unwrap(), g.parse().unwrap(), b.parse().unwrap())))
}

fn ledbrightness(input: &str) -> IResult<&str, u8> {
    let (input, _) = tag("ledbrightness(")(input)?;
    let (input, brightness) = digit1(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, brightness.parse().unwrap()))
}

fn ledenable(input: &str) -> IResult<&str, ()> {
    map(tag("ledenable()"), |_| ())(&input)
}

fn gpio_set(input: &str) -> IResult<&str, (u8,bool)> {
    let (input, _) = tag("GPIO_set(")(input)?;
    let (input, pin) = digit1(input)?;
    let (input, _) = tag(",")(input)?;
    let (input,state) = alt((tag("high"),tag("low")))(input)?;
    let state_bool = match state {
        "high" => true,
        "low" => false,
        _ => unreachable!(),
    };
    let (input, _) = tag(")")(input)?;

    Ok((input,(pin.parse().unwrap(),state_bool)))
}

pub fn command_parser(input: &str) -> IResult<&str,&str> {
  take_while1(|c| c != '(')(input)
}


pub fn test_parser() {
  println!("{:?}", ledcolor("ledcolor(255,255,255)").unwrap());
  println!("{:?}", ledbrightness("ledbrightness(255)").unwrap());
  println!("{:?}", ledenable("ledenable()").unwrap());
}
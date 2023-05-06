use esp_println::println;
use nom::{
    IResult, error::ParseError, InputTake, Compare, InputLength,
};

pub use nom::bytes::complete::tag;

pub fn do_nothing_parser(input: &str) -> IResult<&str, &str> {
    Ok((input, ""))
}

pub fn test_parser() {
    let (remaining_input, output) = do_nothing_parser("my_input").unwrap();
    println!("remaining_input : {:?}", remaining_input);
    println!("output          : {:?}", output);

}
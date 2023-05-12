use esp_println::println;
//mod command_parser;
use crate::command_parser::*;

pub fn evaluate_command(command: &str) {
    match parse(command) {
        Ok((_, command)) => {
            println!("Command: {:?}", command);
            run_command(command);
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

fn run_command(command: CommandEnum) {
    println!("Command: {:?}", command);
    command.execute();
}
use embedded_io::asynch::Write;
use esp_println::println;
//mod command_parser;
use crate::command_parser::*;
use embassy_net::tcp::TcpSocket;

use stackfmt::fmt_truncate;

pub async fn evaluate_command(command: &str, socket: &mut TcpSocket<'_>) {
    let mut buffer = [0u8; 512];

    match parse(command) {
        Ok((_, parsed_command_enum)) => {
            let message = fmt_truncate(&mut buffer, format_args!("Command: {:?}", parsed_command_enum));
            println!("{}", message);
            run_command(parsed_command_enum);
            match socket.write_all(message.as_bytes(),).await {
                Ok(_) => {
                    println!("Command sent");
                }
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e);
        }
    }
}

fn run_command(command: CommandEnum) {
    command.execute();
}
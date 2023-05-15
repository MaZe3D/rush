use async_std::io::prelude::*;
use async_std::net::{SocketAddr, TcpStream};
use clap::Parser;
use crossterm::event::{
    DisableFocusChange, Event, EventStream, KeyCode, KeyEvent, KeyEventKind, KeyModifiers,
};
use crossterm::style::{Color, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{Clear, DisableLineWrap, EnableLineWrap};
use crossterm::{cursor, execute};
use futures::{select, FutureExt, StreamExt};
use std::error::Error;
use std::io::stdout;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    listen_address: SocketAddr,
}

fn print_error(e: impl Error) {
    print_with_style(format!("Error: {:?}", e).into_bytes(), "!", Color::Red);
    println!();
}

//print a vector of chars to the bottom line of the console
fn write_vec_to_console(vec: &Vec<char>) -> Result<(), std::io::Error> {
    execute! {
        stdout(),
        cursor::SavePosition,
        cursor::MoveToColumn(0),
        Clear(crossterm::terminal::ClearType::CurrentLine),
        Print(vec.iter().fold(String::new(), |acc, &num| acc + &num.to_string())),
        cursor::RestorePosition,
    }?;
    Ok(())
}

// print a vector to the bottom line of the console with specified start character and color
fn print_with_style(buffer: Vec<u8>, start_string: &str, color: Color) {
    let _output_string = match String::from_utf8(buffer.clone()) {
        Ok(output_string) => {
            match execute! {
                stdout(),
                cursor::MoveToColumn(0),
                Clear(crossterm::terminal::ClearType::CurrentLine),
                SetForegroundColor(color),
                Print(start_string),
                Print(output_string.clone()),
                ResetColor,
            } {
                Ok(_) => {}
                Err(e) => println!(
                    "Error: {:?} \n Tried to print {:?}",
                    e,
                    output_string.clone()
                ),
            };
        }
        Err(e) => println!("Error: {:?} \n Tried to print {:?}", e, buffer),
    };
}

// try to connect to the server, retrying every 5 seconds if it fails
async fn connect_to_tcp(address: SocketAddr) -> TcpStream {
    println!("connecting to: {}...", address);
    loop {
        match TcpStream::connect(address).await {
            Ok(try_stream) => {
                println!("connected!");
                return try_stream;
            }
            Err(e) => {
                print_error(e);
                println!("retrying in 5 seconds");
                async_std::task::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

// main loop of the client
async fn main_loop(address: SocketAddr) -> Result<(), std::io::Error> {
    let mut stream = connect_to_tcp(address).await;
    let mut buffer = [0u8; 1024];

    let mut reader = EventStream::new();
    let mut cursor_position: u16 = 0;
    let mut input_line: Vec<char> = Vec::new();
    let mut history: Vec<Vec<char>> = Vec::new();
    history.push(Vec::new()); //history[0] is initialized to an empty vector
    let mut history_position: usize = 0;

    loop {
        select! {
            //await inputs from tcp stream
            read_byte_count = stream.read(&mut buffer).fuse() => {
                let buffer = &buffer[..read_byte_count?];
                execute!{ stdout(), Clear(crossterm::terminal::ClearType::CurrentLine), EnableLineWrap }?;
                print_with_style(buffer.to_vec(), " IN ", Color::Cyan);
                execute!{stdout(), DisableLineWrap}?;
            }

            // catch any keyboard activity
            event = reader.next().fuse() => {
                if let Some(Err(e)) = event {
                    break Err(e);
                }
                if let Some(Ok(Event::Key(KeyEvent { code, modifiers, kind: KeyEventKind::Press, state: _ }))) = event {
                    match (code, modifiers) {

                        // event: any character key is pressed
                        // the character is added to the input vector at cursor position
                        (KeyCode::Char(c), KeyModifiers::NONE) |
                        (KeyCode::Char(c), KeyModifiers::SHIFT) => {
                            input_line.insert(cursor_position as usize, c);
                            cursor_position += 1;
                        }

                        // event: enter key is pressed
                        // the input vector is saved and sent to the server
                        // the input line is reset
                        (KeyCode::Enter, KeyModifiers::NONE) |
                        (KeyCode::Char('m'), KeyModifiers::CONTROL) |
                        (KeyCode::Char('j'), KeyModifiers::CONTROL) => {
                            if history.len() >= 2 {
                                if input_line != history[1] {
                                    history.insert(1, input_line.clone());
                                }
                            }
                            else {
                                history.insert(1, input_line.clone());
                            }
                            input_line.push('\n');
                            let input_u8_vector = &input_line.iter().map(|c| *c as u8).collect::<Vec<_>>();
                            execute!{stdout(), EnableLineWrap}?;
                            print_with_style(input_u8_vector.to_vec(), "OUT ", Color::Green);
                            execute!{stdout(), DisableLineWrap}?;
                            stream.write(&input_u8_vector).await?;
                            input_line.clear();
                            history_position = 0;
                            cursor_position = 0;
                        }

                        // event: backspace key is pressed
                        // the character at cursor position is removed from the input vector
                        (KeyCode::Backspace, KeyModifiers::NONE) |
                        (KeyCode::Char('h'), KeyModifiers::CONTROL) => {
                            if input_line.len() > 0 && cursor_position > 0{
                                input_line.remove(cursor_position as usize - 1);
                                cursor_position -= 1;
                            }
                        }

                        //event: left arrow key is pressed
                        // the cursor position is moved left
                        (KeyCode::Left, KeyModifiers::NONE) => {
                            if cursor_position > 0{
                                cursor_position -= 1;
                            }
                        }

                        // event: right arrow key is pressed
                        // the cursor position is moved right
                        (KeyCode::Right, KeyModifiers::NONE) => {
                            if cursor_position < input_line.len() as u16{
                                cursor_position += 1;
                            }
                        }

                        // event: up arrow key is pressed
                        // the input vector is replaced with the previous input vector in the history vector
                        (KeyCode::Up, KeyModifiers::NONE) |
                        (KeyCode::Char('p'), KeyModifiers::CONTROL) => {
                            if history_position < history.len()-1{
                                history_position += 1;
                                input_line = history[history_position].clone();
                                cursor_position = input_line.len() as u16;
                            }
                        }

                        // event: down arrow key is pressed
                        // the input vector is replaced with the next input vector in the history vector
                        (KeyCode::Down, KeyModifiers::NONE) |
                        (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                            if  history_position > 0{
                                history_position -= 1;
                                input_line = history[history_position].clone();
                                cursor_position = input_line.len() as u16;
                            }
                        }

                        // event: delete key is pressed
                        // the character at cursor position is removed from the input vector
                        (KeyCode::Delete, KeyModifiers::NONE) => {
                            if input_line.len() > 0 && cursor_position < input_line.len() as u16{
                                input_line.remove(cursor_position as usize);
                            }
                        }

                        _ => (),
                    }
                }
            }
        }

        if history_position == 0 {
            history[0] = input_line.clone(); // save current input line to first position in history vector
        }

        // move terminal cursor to the new cursor position
        execute! {
            stdout(),
            cursor::MoveTo(cursor_position, crossterm::terminal::size()?.0 - 1),
        }?;

        // display current input line
        write_vec_to_console(&input_line)?;
    }
}

#[async_std::main]
async fn main() {
    match execute! {
        stdout(),
        cursor::EnableBlinking,
        DisableFocusChange,
        Clear(crossterm::terminal::ClearType::All),
        DisableLineWrap
    } {
        Ok(_) => {}
        Err(e) => print_error(e),
    };

    let cli = Cli::parse();
    loop {
        match main_loop(cli.listen_address).await {
            Ok(()) => (),             // main_loop loops infinitely, so this is never reached
            Err(e) => print_error(e), // print error and retry
        };
    }
}

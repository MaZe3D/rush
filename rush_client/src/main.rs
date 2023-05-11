use core::panic;
use std::error::{Error};
use std::io::stdout;

use async_std::net::{TcpStream, SocketAddr};
use async_std::io::{prelude::*};

use clap::Parser;
use crossterm::{cursor, ExecutableCommand, execute};
use crossterm::event::{Event, KeyCode, EventStream, KeyEvent, KeyModifiers, DisableFocusChange};
use crossterm::style::{Print, SetForegroundColor, Color, ResetColor};
use crossterm::terminal::{Clear};

use futures::{FutureExt, select, StreamExt};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli{
    listen_address: SocketAddr,
}

//move the cursor to the bottom line of the console, at the specified column
fn move_cursor(n: u16) -> std::io::Result<()>{
    let bottom_row = crossterm::terminal::size()?.0 - 1;
    stdout().execute(cursor::MoveTo(n, bottom_row))?;
    Ok(())
}

//print a vector of chars to the bottom line of the console
fn write_vec_to_console(vec: &Vec<char>){
        match execute!{
        stdout(),
        cursor::SavePosition,
        cursor::MoveToColumn(0),
        Clear(crossterm::terminal::ClearType::CurrentLine),
    }{
        Ok(_) => {}
        Err(e) => println!("Error: {:?}", e),
    };
    print!("{}", vec.iter().fold(String::new(), |acc, &num| acc + &num.to_string()));
    stdout().execute(cursor::RestorePosition);    
}

fn print_with_style(buffer: Vec<u8>, start_char: char, color: Color){
    let _output_string = match String::from_utf8(buffer){
        Ok(output_string) => {
            match execute!{
                stdout(),
                cursor::MoveToColumn(0),
                Clear(crossterm::terminal::ClearType::CurrentLine),
                SetForegroundColor(color),
                Print(start_char),
                Print(output_string),
                ResetColor,
            }{
                Ok(_) => {}
                Err(e) => println!("Error: {:?}", e),
            };
            }
        Err(e) => println!("Error: {:?}", e),
    };
}

async fn connect_to_tcp(address:SocketAddr)->TcpStream{
    while let mut try_stream = TcpStream::connect(address).await{
        match try_stream{
            Ok(try_stream) => {
                println!("Connected to server");
                return try_stream;
            }
            Err(e) => {
                println!("Error: {:?}", e);
                println!("Retrying in 5 seconds");
                async_std::task::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
    panic!("Could not connect to server");
}

async fn main_loop(address:SocketAddr)->Result<(), impl Error>{
    let mut stream = connect_to_tcp(address).await;
    let mut buffer = [0u8; 250];

    let mut reader = EventStream::new();        
    let mut cursor_position: u16 = 0;
    let mut input_line: Vec<char> = Vec::new();

    loop{
        // println!("looping");
        let mut event = reader.next().fuse(); 

        select! {
            //await inputs from tcp stream
            _ = async{stream.peek(&mut [0u8,1]).await }.fuse() =>{
                stdout().execute(Clear(crossterm::terminal::ClearType::CurrentLine));
                let n = match stream.read(&mut buffer).await{
                    Ok(n) => n,
                    Err(e) => {
                        println!("Error: {:?}", e);
                        break Err(e);
                    }
                };
                print_with_style(buffer[..n].to_vec(), '>', Color::Cyan);
            }
            // catch any keyboard activity 
            maybe_event = event=> {
                match maybe_event{
                    Some(Ok(event)) => {
                        match event {
                            // Event: Any character key is pressed
                            // the character is added to the input vector at cursor position
                            Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) | 
                            Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT, kind: crossterm::event::KeyEventKind::Press, ..})=> {
                                input_line.insert(cursor_position as usize, c);
                                cursor_position += 1;
                            }
                            // Event: Enter key is pressed
                            // the input vector is saved and sent to the server
                            // the input line is reset
                            Event::Key(KeyEvent{code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                //save input to history
                                input_line.push('\n');
                                let input_u8_vector = &input_line.iter().map(|c| *c as u8).collect::<Vec<_>>();
                                print_with_style(input_u8_vector.to_vec(), '<', Color::Green);
                                stream.write(&input_u8_vector).await?;
                                input_line.clear();
                                cursor_position = 0;
                            }
                            // Event: Backspace key is pressed
                            // the character at cursor position is removed from the input vector
                            Event::Key(KeyEvent{code: KeyCode::Backspace, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                if input_line.len() > 0 && cursor_position > 0{
                                    input_line.remove(cursor_position as usize - 1);
                                    cursor_position -= 1;
                                }
                            }
                            //Event: Left arrow key is pressed
                            // the cursor position is moved left
                            Event::Key(KeyEvent{code: KeyCode::Left, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                // cursor::MoveLeft(1);
                                if cursor_position > 0{
                                    cursor_position -= 1;
                                }
                            }
                            // Event: Right arrow key is pressed
                            // the cursor position is moved right
                            Event::Key(KeyEvent{code: KeyCode::Right, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                // cursor::MoveRight(1);
                                if cursor_position < input_line.len() as u16{
                                    cursor_position += 1;
                                }
                            }     
                            _ => {}
                        }
                    }
                    Some(Err(e)) => println!("Error:{:?}", e),
                    None =>  {},
                }
            }
        }        
        move_cursor(cursor_position);          //move terminal cursor to the new cursor position
        write_vec_to_console(&input_line);      //display current input line
        buffer = [0u8; 250];                    //clear buffer            
    }
}

#[async_std::main]
async fn main()-> Result<(), Box<dyn Error>> {
    stdout()
        .execute(cursor::EnableBlinking)?
        .execute(DisableFocusChange)?
        .execute(Clear(crossterm::terminal::ClearType::All))?;

    let cli = Cli::parse();
    println!("Listening on: {}", cli.listen_address);
    match main_loop(cli.listen_address).await{
        Ok(_) => {}
        Err(e) => {
            println!("Error: {:?}", e);
            main_loop(cli.listen_address).await?;
        }
    }
    Ok(())   
}
use std::error::Error;
use std::io::Write;
use std::vec;

use async_std::net::{TcpStream, SocketAddr};
use async_std::io::{prelude::*, stdin, BufReader};

use clap::Parser;
use crossterm::{cursor, ExecutableCommand,};
use crossterm::event::{poll, read, Event, KeyCode, EventStream, KeyEvent, KeyModifiers, DisableFocusChange};
use crossterm::style::{Print};
use crossterm::terminal::{Clear, size};

use futures::{FutureExt, select, StreamExt};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli{
    listen_address: SocketAddr,
}

fn move_cursor(n: u16) -> std::io::Result<()>{
    use std::io::{stdout};
    let bottom_row = crossterm::terminal::size()?.0 - 1;
    stdout().execute(cursor::MoveTo(n, bottom_row))?;
    Ok(())
}

fn write_vec_to_console(vec: &Vec<char>){
    use std::io::{stdout};
    stdout().execute(cursor::SavePosition);
    stdout().execute(cursor::MoveToColumn(0));   
    print!("{}", vec.iter().fold(String::new(), |acc, &num| acc + &num.to_string()));
    stdout().execute(cursor::RestorePosition);
}

#[async_std::main]
async fn main()-> Result<(), Box<dyn Error>> {
    use std::io::stdout;
    stdout()
        .execute(cursor::EnableBlinking)?
        .execute(DisableFocusChange)?
        .execute(Clear(crossterm::terminal::ClearType::All))?;

    let cli = Cli::parse();
    println!("Listening on: {}", cli.listen_address);

    let mut input_string = String::new();
    let mut stream = TcpStream::connect(&cli.listen_address).await?;
    let mut buffer = [0u8; 250];
    let mut buffered_stdin = BufReader::new(stdin());

    let mut reader = EventStream::new();
    let mut cursor_position: u16 = 0;
    let mut input_line: Vec<char> = Vec::new();

    loop{
        // println!("looping");
        let mut event = reader.next().fuse(); 

        select! {
            //await inputs from tcp stream
            _ = async{stream.peek(&mut [0u8,1]).await }.fuse() =>{
                println!("reading from stream");
                let n = stream.read(&mut buffer).await?;
                // stdout().write_all(&buffer[..n])?;
            }
            //await inputs from stdin
            // _ = async{buffered_stdin.read_line(&mut input_string).await}.fuse() =>{
            //     println!("writing to stream");
            //     stream.write(&input_string.as_bytes()).await?;
            // }
            maybe_event = event=> {
                match maybe_event{
                    Some(Ok(event)) => {
                        match event {
                            Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) | 
                            Event::Key(KeyEvent{code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT, kind: crossterm::event::KeyEventKind::Press, ..})=> {
                                input_line.insert(cursor_position as usize, c);
                                cursor_position += 1;
                            }
                            Event::Key(KeyEvent{code: KeyCode::Enter, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                println!("writing to stream");
                                stream.write(&input_line.iter().map(|c| *c as u8).collect::<Vec<_>>()).await?;
                            }
                            Event::Key(KeyEvent{code: KeyCode::Backspace, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                if input_line.len() > 0 && cursor_position > 0{
                                    input_line.remove(cursor_position as usize - 1);
                                    cursor_position -= 1;
                                }
                            }
                            Event::Key(KeyEvent{code: KeyCode::Left, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                // cursor::MoveLeft(1);
                                if cursor_position > 0{
                                    cursor_position -= 1;
                                }
                            }
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
                    None => break Ok(()),
                }
            }
        }
        // print_vector(&input_line);
        write_vec_to_console(&input_line);
        move_cursor(cursor_position)?;
        buffer = [0u8; 250];    //clear buffer
        input_string.clear();
    }
}
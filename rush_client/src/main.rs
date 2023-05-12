use std::error::{Error};
use std::io::stdout;

use async_std::net::{TcpStream, SocketAddr};
use async_std::io::{prelude::*};
use async_recursion::async_recursion;
use futures::{FutureExt, select, StreamExt};

use clap::Parser;
use crossterm::{cursor, execute};
use crossterm::event::{Event, KeyCode, EventStream, KeyEvent, KeyModifiers, DisableFocusChange};
use crossterm::style::{Print, SetForegroundColor, Color, ResetColor};
use crossterm::terminal::Clear;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli{
    listen_address: SocketAddr,
}

fn print_error(e: impl Error){
    print_with_style (format!("Error: {:?}", e).into_bytes(), '!', Color::Red);
}

//move the cursor to the bottom line of the console, at the specified column
fn move_cursor(n: u16) -> std::io::Result<()>{
    let bottom_row = match crossterm::terminal::size(){
        Ok(n) => n.0,
        Err(e) => {
            print_error(e);
            0
        }
    };
    match execute!{
        stdout(),
        cursor::MoveTo(n, bottom_row-1),
    }{
        Ok(_) => {}
        Err(e) => print_error(e),
    };
    Ok(())
}

//print a vector of chars to the bottom line of the console
fn write_vec_to_console(vec: &Vec<char>){
    let mut stdout = stdout();
        match execute!{
        stdout,
        cursor::SavePosition,
        cursor::MoveToColumn(0),
        Clear(crossterm::terminal::ClearType::CurrentLine),
    }{
        Ok(_) => {}
        Err(e) => print_error(e),
    };
    print!("{}", vec.iter().fold(String::new(), |acc, &num| acc + &num.to_string()));
    match execute!{stdout, cursor::RestorePosition}{
        Ok(_) => {}
        Err(e) => print_error(e),
    };   
}

// print a vector to the bottom line of the console with specified start character and color
fn print_with_style(buffer: Vec<u8>, start_char: char, color: Color){
    let _output_string = match String::from_utf8(buffer.clone()){
        Ok(output_string) => {
            match execute!{
                stdout(),
                cursor::MoveToColumn(0),
                Clear(crossterm::terminal::ClearType::CurrentLine),
                SetForegroundColor(color),
                Print(start_char),
                Print(output_string.clone()),
                ResetColor,
            }{
                Ok(_) => {}
                Err(e) => println!("Error: {:?} \n Tried to print {:?}", e, output_string.clone()),
            };
            }
        Err(e) => println!("Error: {:?} \n Tried to print {:?}", e, buffer),
    };
}

// try to connect to the server, retrying every 5 seconds if it fails
async fn connect_to_tcp(address:SocketAddr)->TcpStream{
    loop{
    let try_stream = TcpStream::connect(address).await;
        match try_stream{
            Ok(try_stream) => {
                println!("Connected to server");
                return try_stream;
            }
            Err(e) => {
                print_error(e);
                println!("Retrying in 5 seconds");
                async_std::task::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}

// main loop of the client
async fn main_loop(address:SocketAddr)->Result<(), impl Error>{
    let mut stream = connect_to_tcp(address).await;
    let mut buffer = [0u8; 250];

    let mut reader = EventStream::new();        
    let mut cursor_position: u16 = 0;
    let mut input_line: Vec<char> = Vec::new();
    let mut history: Vec<Vec<char>> = Vec::new();
    history.push(Vec::new());                       //history[0] is initialized to an empty vector  
    // history.push(Vec::new());                       //history[1] is initialized to an empty vector
    let mut history_position: usize = 0;

    loop{
        // println!("looping");
        let mut event = reader.next().fuse(); 

        select! {
            //await inputs from tcp stream
            _ = async{stream.peek(&mut [0u8,1]).await }.fuse() =>{
                match execute!{stdout(), Clear(crossterm::terminal::ClearType::CurrentLine)}{
                    Ok(_) => {}
                    Err(e) => print_error(e),
                };
                let n = match stream.read(&mut buffer).await{
                    Ok(n) => n,
                    Err(e) => {
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
                                if history.len() >= 2{
                                    if input_line != history[1]{
                                        history.insert(1, input_line.clone());
                                    }
                                }
                                else{
                                    history.insert(1, input_line.clone());
                                }
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
                            // Event: Up arrow key is pressed
                            // the input vector is replaced with the previous input vector in the history vector    
                            Event::Key(KeyEvent{code: KeyCode::Up, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                if history_position < history.len()-1{
                                    history_position += 1;
                                    input_line = history[history_position].clone();
                                    cursor_position = input_line.len() as u16;
                                }
                            }
                            // Event: Down arrow key is pressed
                            // the input vector is replaced with the next input vector in the history vector
                            Event::Key(KeyEvent{code: KeyCode::Down, modifiers: KeyModifiers::NONE, kind: crossterm::event::KeyEventKind::Press, ..}) => {
                                if  history_position > 0{
                                    history_position -= 1;
                                    input_line = history[history_position].clone();
                                    cursor_position = input_line.len() as u16;
                                }
                            }
                                
                            _ => {}
                        }
                    }
                    Some(Err(e)) => print_error(e),
                    None =>  {},
                }
            }
        }

        if history_position == 0{
            history[0] = input_line.clone();  // save current input line to first position in history vector
        }        
        match move_cursor(cursor_position){
            Ok(_) => {},
            Err(e) => {
                print_error(e);
            }
        }   //move terminal cursor to the new cursor position
        write_vec_to_console(&input_line);      //display current input line
        buffer = [0u8; 250];                        //clear buffer            
    }
}

#[async_recursion]
async fn resilient_main_loop(address:SocketAddr){
    match main_loop(address).await{
        Ok(_) => {}                 // main_loop loops infinitely, so this is never reached
        Err(e) => {     // if main_loop fails, it returns an error and it is reset 
            print_error(e);
            resilient_main_loop(address).await;
        }
    }
}

#[async_std::main]
async fn main(){
    match execute!{
        stdout(),
        cursor::EnableBlinking,
        DisableFocusChange,
        Clear(crossterm::terminal::ClearType::All)
    }{
        Ok(_) => {}
        Err(e) => print_error(e),
    };

    let cli = Cli::parse();
    println!("Connection: {}", cli.listen_address);
    resilient_main_loop(cli.listen_address).await;
}
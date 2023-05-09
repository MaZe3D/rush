use std::error::Error;
use std::io::Write;

use async_std::net::{TcpStream, SocketAddr};
use async_std::io::{prelude::*, stdin, BufReader};

use clap::Parser;
use crossterm::{cursor, ExecutableCommand,};

use futures::select;
use futures::FutureExt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli{
    listen_address: SocketAddr,
}

fn write_to_console(buffer: &[u8], n: usize) -> std::io::Result<()>{
    use std::io::{stdout};
    stdout()
        .execute(cursor::SavePosition)?
        .execute(cursor::MoveToPreviousLine(1))?;
    stdout().execute(cursor::RestorePosition)?;
    println!("Read from Buffer: ");
    stdout().write_all(&buffer[..n])?;

    Ok(())
}

#[async_std::main]
async fn main()-> Result<(), Box<dyn Error>> {
    use std::io::stdout;
    stdout().execute(cursor::EnableBlinking)?;

    let cli = Cli::parse();
    println!("Listening on: {}", cli.listen_address);

    let mut input_string = String::new();
    let mut stream = TcpStream::connect(&cli.listen_address).await?;
    let mut buffer = [0u8; 250];
    let mut buffered_stdin = BufReader::new(stdin());

    loop{
        println!("looping");
        select! {
            //await inputs from tcp stream
            _ = async{stream.peek(&mut [0u8,1]).await }.fuse() =>{
                println!("reading from stream");
                let n = stream.read(&mut buffer).await?;
                write_to_console(&buffer, n)?;
                // stdout().write_all(&buffer[..n])?;
            }
            //await inputs from stdin
            _ = async{buffered_stdin.read_line(&mut input_string).await}.fuse() =>{
                println!("writing to stream");
                stream.write(&input_string.as_bytes()).await?;
            }
        }
        buffer = [0u8; 250];    //clear buffer
        input_string.clear();
    }
}
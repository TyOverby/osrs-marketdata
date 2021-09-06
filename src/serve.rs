use byteorder::*;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::fs::{File};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listening_on ="0.0.0.0:8080" ;
    let listener = TcpListener::bind(listening_on).unwrap();
    println!("listening on {}", listening_on);

    for stream in listener.incoming() {
        std::thread::spawn(|| {
            let stream = stream.unwrap();
            let _ = handle_connection(stream);
        });
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let s  = String::from_utf8_lossy(&buffer);
    let s = s.as_ref();
    let parts = s.split_ascii_whitespace().collect::<Vec<_>>();
    let path = match parts.get(1) {
        | Some (s) => s.split("/").filter(|s| !s.is_empty()).collect::<Vec<_>>(),
        | None => vec![]
    };
    let id: u32 = path[0].parse()?;
    println!("request for {}!", id);

    let mut file = std::fs::File::open(format!("data/{}.bin", id))?;
    write!(&stream, "HTTP/1.1 200 OK\r\nAccess-Control-Allow-Origin: *\r\n\r\n")?;
    writeln!(&stream, "x, low, high")?;
    let _ = send_output(&mut stream, &mut file);
    Ok(())
}

fn send_output(stream: &mut TcpStream, file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        let low_time= file.read_u32::<LittleEndian>()?;
        let high_time = file.read_u32::<LittleEndian>()?;
        let low = file.read_u32::<LittleEndian>()?;
        let high = file.read_u32::<LittleEndian>()?;
        let time = low_time.max(high_time);
        writeln!(stream, "{}, {}, {}", time, low, high)?;
    }
}
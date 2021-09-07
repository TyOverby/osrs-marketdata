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
    //writeln!(&stream, "x, delta, low, high")?;
    writeln!(&stream, "[")?;
    let _ = send_output(&mut stream, &mut file);
    writeln!(&stream, "]")?;
    Ok(())
}

fn send_output(stream: &mut TcpStream, file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    let mut highest_low_time = 0;
    let mut highest_high_time = 0;
    let mut prev_low = 0;
    let mut prev_high= 0;
    let mut is_first = true;
    loop {
        let low_time= file.read_u32::<LittleEndian>()?;
        let high_time = file.read_u32::<LittleEndian>()?;
        let mut low = file.read_u32::<LittleEndian>()?;
        let mut high = file.read_u32::<LittleEndian>()?;

        // Ban time-traveling
        if low_time < highest_low_time {
            low = prev_low;
        }
        if high_time < highest_high_time {
            high = prev_high;
        }
        if low_time < highest_low_time && high_time < highest_high_time {
            continue;
        }

        highest_low_time = low_time;
        highest_high_time = high_time;
        prev_low = low;
        prev_high = high;

        let time = low_time.max(high_time);
        let delta = u32::saturating_sub(high, low);
        if !is_first {
            writeln!(stream, ",")?;
        }
        is_first = false;
        write!(stream, "[{}, {}, {}, {}]", time, delta, low, high)?;
    }
}
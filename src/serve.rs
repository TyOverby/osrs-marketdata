use byteorder::*;
use std::fs::File;
use std::fmt::Write;
use rouille::{Response, router};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    rouille::start_server("0.0.0.0:8080", move |request| {
        let response = 
        router!(request,
            (GET) (/metadata) => {
                match std::fs::File::open(format!("data/metadata")) {
                    Ok (file) => Response::from_file("application/json", file),
                    Err(_) =>  Response::empty_400()
                }
            },
            (GET) (/{id:u32}) => {
                match std::fs::File::open(format!("data/{}.bin", id)) {
                    Ok (mut file) => Response::from_data("application/json",response(&mut file)),
                    Err(_) =>  Response::empty_404()
                }
            },
            _ => Response::empty_404()
        );
        response.with_additional_header("Access-Control-Allow-Origin", "*")
    });
}

fn response(file: &mut File) -> String {
    let mut out = String::new();
    write!(&mut out,"[").unwrap();
    let mut highest_low_time = 0;
    let mut highest_high_time = 0;
    let mut prev_low = 0;
    let mut prev_high= 0;
    let mut is_first = true;
    let mut do_read = || -> Result<(), Box<dyn std::error::Error>> {
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
            return Ok(());
        }

        highest_low_time = low_time;
        highest_high_time = high_time;
        prev_low = low;
        prev_high = high;

        let time = low_time.max(high_time);
        if !is_first {
            writeln!(&mut out, ",")?;
        }
        is_first = false;
        write!(&mut out, "[{}, {}, {}]", time, low, high)?;
        Ok(())
    };

    loop {
        match do_read() {
            Ok(()) => (),
            Err(_) => break,
        }
    }
    write!(&mut out,"]").unwrap();
    out
}
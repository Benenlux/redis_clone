pub(crate) mod frame;

use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
};

use crate::frame::parse_stream;

fn handle_stream(mut stream: TcpStream) {
    let mut buf_reader = BufReader::new(&mut stream);

    loop {
        let parsed_stream = parse_stream(&mut buf_reader);

        let response = match parsed_stream {
            Err(e) => {
                eprintln!("Error while parsing stream: {}", e);

                "+Error\r\n"
            }
            Ok(_) => "+Ok\r\n",
        };
        if response.is_empty() {
            println!("Connection closed");
            break;
        }

        buf_reader.get_mut().write_all(response.as_bytes()).unwrap();
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| {
                    handle_stream(stream);
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

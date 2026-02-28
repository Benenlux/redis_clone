pub(crate) mod frame;
pub(crate) mod handler;
pub(crate) mod lib;
pub(crate) mod table;

use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use crate::{frame::parse_stream, handler::handle_request, table::Table};

fn handle_stream(mut stream: TcpStream, table: Arc<Table>) {
    let mut buf_reader = BufReader::new(&mut stream);

    loop {
        let parsed_stream = parse_stream(&mut buf_reader);

        let response = match parsed_stream {
            Err(e) => {
                eprintln!("Error while parsing stream: {}", e);

                String::from("+Error\r\n")
            }
            Ok(req) => handle_request(req, &table).unwrap_or_else(|e| e),
        };
        if response.is_empty() {
            println!("Connection closed");
            break;
        }

        buf_reader.get_mut().write_all(response.as_bytes()).unwrap();
    }
}

fn main() {
    let shared_table = Arc::new(Table::new());
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let table_clone = shared_table.clone();
                std::thread::spawn(|| {
                    handle_stream(stream, table_clone);
                });
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}

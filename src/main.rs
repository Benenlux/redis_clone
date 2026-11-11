use redis_clone::frame::parse_stream;
use redis_clone::handler::handle_request;
use redis_clone::table::Table;
use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
fn handle_stream(mut stream: TcpStream, table: Arc<Table>) {
    let mut buf_reader = BufReader::new(&mut stream);
    loop {
        let parsed_stream = parse_stream(&mut buf_reader);
        let req = match parsed_stream {
            Err(e) => {
                eprintln!("Error while parsing stream: {}", e);

                break;
            }
            Ok(req) => req,
        };
        if req.is_empty() {
            println!("Connection closed");
            break;
        }

        let writer = buf_reader.get_mut();
        if let Err(err_resp) = handle_request(req, &table, writer) {
            if cfg!(feature = "verbose-print") {
                println!("Sent back error: {:?}", err_resp);
            }
            if let Err(io_err) = writer.write_all(err_resp.as_bytes()) {
                eprintln!("Failed to write to client (disconnected): {}", io_err);
                break;
            }
        }
    }
}
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();
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

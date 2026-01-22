use io::ErrorKind::InvalidData as IoError;
use std::io::{self, BufRead};

fn extract_string_from_buffer<T: BufRead>(buf_reader: &mut T) -> std::io::Result<String> {
    let mut length_of_string = String::new();
    buf_reader.read_line(&mut length_of_string)?;

    let string_size: usize = match length_of_string.split_at(1) {
        ("$", size) if size.trim().parse::<usize>().is_ok() => {
            size.trim().parse::<usize>().expect("It is already ok!")
        }
        (_, _) => return Err(io::Error::new(IoError, "Invalid String")),
    };

    let mut buffer = vec![0u8; string_size];
    buf_reader.read_exact(&mut buffer)?;

    let mut clrf = [0u8; 2];
    buf_reader.read_exact(&mut clrf)?;

    String::from_utf8(buffer).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

pub fn parse_stream<T: BufRead>(buf_reader: &mut T) -> String {
    let mut resp_buffer = String::new();
    let stream_bytes = buf_reader.read_line(&mut resp_buffer);
    if stream_bytes.is_err() {
        return String::from("No command found");
    }
    let request_size = match resp_buffer[1..resp_buffer.len() - 2].parse::<usize>() {
        Ok(number) => number,
        // TODO: add proper handling for non-bulk strings
        Err(_) => return String::from("No bulk string seen!"),
    };
    let mut parsed_strings: Vec<String> = Vec::new();
    for _string in 0..request_size {
        match extract_string_from_buffer(buf_reader) {
            Ok(string) => parsed_strings.push(string),
            Err(_) => {
                return String::from("Unable to read bulk string");
            }
        }
    }
    println!("{:?}", parsed_strings);
    String::from("+OK\r\n")
}

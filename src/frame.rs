use io::ErrorKind::InvalidData as IoError;
use std::{
    fmt,
    io::{self, BufRead},
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum RespError {
    ConnectionClosed,
    Io(io::Error),
    InvalidProtocol(String),
    Utf8(FromUtf8Error),
}

impl fmt::Display for RespError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespError::ConnectionClosed => write!(f, "Connection closed by peer"),
            RespError::Io(e) => write!(f, "IO Error: {}", e),
            RespError::InvalidProtocol(msg) => write!(f, "Protocol Error: {}", msg),
            RespError::Utf8(e) => write!(f, "UTF-8 Error: {}", e),
        }
    }
}

impl std::error::Error for RespError {}

impl From<io::Error> for RespError {
    fn from(error: io::Error) -> Self {
        RespError::Io(error)
    }
}

impl From<FromUtf8Error> for RespError {
    fn from(error: FromUtf8Error) -> Self {
        RespError::Utf8(error)
    }
}

fn parse_bulk_string<T: BufRead>(buf_reader: &mut T) -> Result<String, RespError> {
    let mut length_of_string = String::new();
    match buf_reader.read_line(&mut length_of_string) {
        Err(_) => return Err(RespError::ConnectionClosed),
        Ok(0) => return Err(RespError::ConnectionClosed),
        Ok(size) => size,
    };

    if !length_of_string.starts_with('$') {
        return Err(RespError::InvalidProtocol(format!(
            "Expected '$', got '{}'",
            length_of_string.trim()
        )));
    }

    let len_str = length_of_string[1..].trim();
    let string_size = len_str.parse::<usize>().map_err(|_| {
        RespError::InvalidProtocol(format!("Invalid bulk string length: '{}'", len_str))
    })?;

    let mut buffer = vec![0u8; string_size];
    buf_reader.read_exact(&mut buffer)?;

    let mut clrf = [0u8; 2];
    buf_reader.read_exact(&mut clrf)?;

    Ok(String::from_utf8(buffer)?)
}

pub fn parse_stream<T: BufRead>(buf_reader: &mut T) -> Result<Vec<String>, RespError> {
    let mut resp_buffer = String::new();
    let stream_bytes = buf_reader.read_line(&mut resp_buffer);
    if stream_bytes.is_err() {
        return Err(RespError::ConnectionClosed);
    }
    let request_size = match resp_buffer[1..resp_buffer.len() - 2].parse::<usize>() {
        Ok(number) => number,
        // TODO: add proper handling for non-bulk strings
        Err(_) => {
            return Err(RespError::InvalidProtocol(
                "Expected bulk string".to_string(),
            ));
        }
    };
    let mut parsed_strings: Vec<String> = Vec::new();
    for _string in 0..request_size {
        match parse_bulk_string(buf_reader) {
            Ok(string) => parsed_strings.push(string),
            Err(e) => return Err(e),
        }
    }
    println!("{:?}", parsed_strings);
    Ok(parsed_strings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::panic;
    use std::io::Cursor;

    //Unit tests for parse_bulk_string
    #[test]
    fn test_parse_string() {
        let input = b"$5\r\nHello\r\n";
        let mut cursor = Cursor::new(&input[..]);
        let result = parse_bulk_string(&mut cursor).unwrap();
        assert_eq!(result, "Hello")
    } 
    #[test]
    fn test_empty_string() {
        let input = b"";
        let mut cursor = Cursor::new(&input[..]);
        let result = parse_bulk_string(&mut cursor);
        assert!(matches!(result, Err(RespError::ConnectionClosed)));
    }
    #[test]
    fn test_invalid_string_prefix() {
        let input = b":\r\nHello\r\n";
        let mut cursor = Cursor::new(&input[..]);
        let result = parse_bulk_string(&mut cursor);
        match result {
            Err(RespError::InvalidProtocol(msg)) => {
                assert_eq!(msg, "Expected '$', got ':'");
            }
            _ => panic!("Expected InvalidProtocol error, got {:?}", result),
        }
    }
    #[test]
    fn test_invalid_string_number() {
        let input = b"$abc\r\nHello\r\n";
        let mut cursor = Cursor::new(&input[..]);
        let result = parse_bulk_string(&mut cursor);
        match result {
            Err(RespError::InvalidProtocol(msg)) => {
                assert_eq!(msg, "Invalid bulk string length: 'abc'");
            }
            _ => panic!("Exptected InvalidProtocol error"),
        }
    }
    #[test]
    fn test_invalid_utf8_string() {
        let input = b"$1\r\n\xFF\r\n";
        let mut cursor = Cursor::new(&input[..]);
        let result = parse_bulk_string(&mut cursor);

        assert!(matches!(result, Err(RespError::Utf8(_))))
    }
    #[test]
    fn test_io_error_unexpected_eof() {
        let input = b"$5\r\nHi";
        let mut cursor = Cursor::new(&input[..]);
        let result = parse_bulk_string(&mut cursor);

        assert!(matches!(result, Err(RespError::Io(_))));
    }

    //Unit tests for parse_stream
    #[test]
    fn test_parse_bulk_string() {
        let input = b"*2\r\n$5\r\nHello\r\n$5\r\nWorld\r\n";
        let mut cursor = Cursor::new(&input[..]);
        let result = parse_stream(&mut cursor).unwrap();
        assert_eq!(result, Vec::from(["Hello", "World"]))
    }
}

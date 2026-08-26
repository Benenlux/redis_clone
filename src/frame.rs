use crate::error::*;
use std::io::BufRead;
// Extracts the individual word from the bulk string
// Returns the word inside of a new String
fn parse_bulk_string<T: BufRead>(buf_reader: &mut T) -> Result<String, RespError> {
    let mut length_of_string = String::new();

    //Try to extract the number of expected characters
    match buf_reader.read_line(&mut length_of_string) {
        Err(e) => return Err(RespError::Io(e)),
        Ok(0) => return Err(RespError::ConnectionClosed),
        Ok(size) => size,
    };

    if !length_of_string.starts_with('$') {
        return Err(RespError::InvalidProtocol(format!(
            "Expected '$', got '{}'",
            length_of_string.trim()
        )));
    }

    // Removes the new line and carriage return symbols, as well as the first symbol from the string
    let len_str = length_of_string[1..].trim();
    let string_size = len_str.parse::<usize>().map_err(|_| {
        RespError::InvalidProtocol(format!("Invalid bulk string length: '{}'", len_str))
    })?;

    let mut buffer = vec![0u8; string_size];
    buf_reader.read_exact(&mut buffer)?;

    //Read away the last two \r\n symbols from the bug reader for the next iteration
    let mut crlf = [0u8; 2];
    buf_reader.read_exact(&mut crlf)?;

    Ok(String::from_utf8(buffer)?)
}

// Parses the raw incomming stream into an array of strings for future parsing
// Currently takes in any type that implements the BufRead method
// Returns an error if the stream couldn't be read or if there is an encoding problem
pub fn parse_stream<T: BufRead>(buf_reader: &mut T) -> Result<Vec<String>, RespError> {
    let mut resp_buffer = String::new();

    //Read the first line of the stream, which contains the number of commands
    //Typically "*X\r\n" where X is the number of commands
    let stream_bytes = buf_reader.read_line(&mut resp_buffer);

    if stream_bytes.is_err() {
        return Err(RespError::ConnectionClosed);
    }

    if cfg!(feature = "verbose-print") {
        println!("Got stream: {:?}", resp_buffer);
    }

    if resp_buffer.len() < 2 {
        return Err(RespError::ConnectionClosed);
    }
    //Because this is a user request, the size is not known at compile time
    //Thus there needs to be a check to see if the number of elements is indeed correctly parsed
    let request_size = match resp_buffer[1..resp_buffer.len() - 2].parse::<usize>() {
        Ok(number) => number,
        Err(_) => {
            return Err(RespError::InvalidProtocol(
                "Invalid first line of command".to_string(),
            ));
        }
    };

    let mut parsed_strings: Vec<String> = Vec::new();
    for _string in 0..request_size {
        {
            let string = parse_bulk_string(buf_reader)?;
            parsed_strings.push(string)
        }
    }
    if cfg!(feature = "verbose-print") {
        println!("{:?}", parsed_strings);
    }
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
            _ => panic!("Expected InvalidProtocol error"),
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

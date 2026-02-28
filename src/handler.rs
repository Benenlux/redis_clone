use std::{str::FromStr, sync::Arc};

use redis_clone::encode_error;

use crate::table::Table;

#[derive(Debug, PartialEq)]
pub enum CommandTypes {
    Set,
    Get,
}

#[derive(Debug)]
pub struct CommandParseError;

impl FromStr for CommandTypes {
    type Err = CommandParseError;
    fn from_str(s: &str) -> Result<CommandTypes, Self::Err> {
        match s {
            "SET" => Ok(CommandTypes::Set),
            "GET" => Ok(CommandTypes::Get),
            _ => Err(CommandParseError),
        }
    }
}

pub fn handle_request(request: Vec<String>, table: &Arc<Table>) -> Result<String, String> {
    let mut req_iter = request.into_iter();
    let req_command = match req_iter.next() {
        Some(val) => val,
        None => return Err(encode_error("Expected command")),
    };
    let command = match req_command.parse::<CommandTypes>() {
        Ok(command) => command,
        Err(_) => return Err(encode_error("Received invalid command")),
    };
    match command {
        CommandTypes::Get => {
            let key = req_iter.next().ok_or(encode_error(
                "Wrong number of arguments for 'get' command, expected key",
            ))?;
            Ok(table.get(key))
        }
        CommandTypes::Set => {
            let key = req_iter.next().ok_or(encode_error(
                "Wrong number of arguments for 'set' command, expected key",
            ))?;
            let val = req_iter.next().ok_or(encode_error(
                "Wrong number of arguments for 'set' command, expected value",
            ))?;
            Ok(table.set(key, val))
        }
    }
}

#[cfg(test)]
mod tests {

    use redis_clone::encode_simple_string;

    use super::*;

    #[test]
    fn valid_set_command() {
        let table = Arc::new(Table::new());
        let request = vec![
            "SET".to_string(),
            "CAR".to_string(),
            "vroom vroom".to_string(),
        ];
        let response = handle_request(request, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response, encode_simple_string("OK"));
    }

    #[test]
    fn valid_set_get_command() {
        let table = Arc::new(Table::new());
        let request = vec![
            "SET".to_string(),
            "CAR".to_string(),
            "vroom vroom".to_string(),
        ];
        let response_set = handle_request(request, &table).unwrap_or_else(|e| e.to_string());

        let request_2 = vec![
            "GET".to_string(),
            "CAR".to_string(),
            "vroom vroom".to_string(),
        ];
        let response_get = handle_request(request_2, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, encode_simple_string("OK"));
        assert_eq!(response_get, "vroom vroom".to_string());
    }

    #[test]
    fn only_command() {
        let table = Arc::new(Table::new());
        let request = vec!["SET".to_string()];
        let response = handle_request(request, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(
            response,
            encode_error("Wrong number of arguments for 'set' command, expected key")
        )
    }
}

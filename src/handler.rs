use std::{str::FromStr, sync::Arc};

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

pub fn handle_request(request: Vec<String>, table: &Arc<Table>) -> Result<String, &str> {
    let mut req_iter = request.into_iter();
    let req_command = match req_iter.next() {
        Some(val) => val,
        None => return Err("+Error\r\n"),
    };
    let command = match req_command.parse::<CommandTypes>() {
        Ok(command) => command,
        Err(_) => return Err("+Error\r\n"),
    };
    match command {
        CommandTypes::Get => {
            let key = req_iter.next().ok_or("+Error\r\n")?;
            Ok(table.get(key))
        }
        CommandTypes::Set => {
            let key = req_iter.next().ok_or("+Error\r\n")?;
            let val = req_iter.next().ok_or("+Error\r\n")?;
            Ok(table.set(key, val))
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn valid_set_command() {
        let table = Arc::new(Table::new());
        let request = vec![
            "SET".to_string(),
            "CAR".to_string(),
            "vroom vroom".to_string(),
        ];
        let response = handle_request(request, &table);
        assert_eq!(response, "OK");
    }

    #[test]
    fn valid_set_get_command() {
        let table = Arc::new(Table::new());
        let request = vec![
            "SET".to_string(),
            "CAR".to_string(),
            "vroom vroom".to_string(),
        ];
        let response_set = handle_request(request, &table);

        let request_2 = vec![
            "GET".to_string(),
            "CAR".to_string(),
            "vroom vroom".to_string(),
        ];
        let response_get = handle_request(request_2, &table);
        assert_eq!(response_set, "OK");
        assert_eq!(response_get, "vroom vroom".to_string());
    }
}

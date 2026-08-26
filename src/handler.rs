use crate::error::RespError;
use crate::table::Table;
use crate::{commands::set, error::CommandError};
use redis_clone::encode_error;
use std::{str::FromStr, sync::Arc};

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
        None => return Err(CommandError::NoCommand.to_resp()),
    };
    let command = match req_command.parse::<CommandTypes>() {
        Ok(command) => command,
        Err(_) => return Err(CommandError::InvalidCommand(req_command).to_resp()),
    };
    match command {
        CommandTypes::Get => {
            let key = req_iter.next().ok_or(
                CommandError::InsufficientArgument("GET".to_string(), "key".to_string()).to_resp(),
            )?;
            Ok(table.get(&key))
        }
        CommandTypes::Set => set::handle_set(req_iter, table),
    }
}

#[cfg(test)]
mod tests {

    use redis_clone::{RespReplies, encode_simple_string};

    use crate::error::CommandError;

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
        assert_eq!(response, RespReplies::OKString.to_resp());
    }

    #[test]
    fn no_set_val() {
        let table = Arc::new(Table::new());
        let request = vec!["SET".to_string(), "CAR".to_string()];
        let response = handle_request(request, &table).unwrap_or_else(|e| e.to_string());

        assert_eq!(
            response,
            CommandError::InsufficientArgument("SET".to_string(), "value".to_string()).to_resp()
        )
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
        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("vroom vroom"));
    }
    #[test]
    fn invalid_set_get_command() {
        let table = Arc::new(Table::new());
        let request = vec![
            "SET".to_string(),
            "CAR".to_string(),
            "vroom vroom".to_string(),
        ];
        let response_set = handle_request(request, &table).unwrap_or_else(|e| e.to_string());

        let request_2 = vec![
            "GET".to_string(),
            "Bike".to_string(),
            "tring tring".to_string(),
        ];
        let response_get = handle_request(request_2, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, RespReplies::NullString.to_resp());
    }

    #[test]
    fn only_command() {
        let table = Arc::new(Table::new());
        let request = vec!["SET".to_string()];
        let response = handle_request(request, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(
            response,
            CommandError::InsufficientArgument("SET".to_string(), "key".to_string()).to_resp()
        )
    }

    #[test]
    fn no_command() {
        let table = Arc::new(Table::new());
        let request: Vec<String> = Vec::new();

        let response = handle_request(request, &table).unwrap_or_else(|e| e);

        assert_eq!(response, CommandError::NoCommand.to_resp())
    }

    #[test]
    fn invalid_command() {
        let table = Arc::new(Table::new());
        let request = vec!["SUPERCOOLCOMMAND".to_string()];

        let response = handle_request(request, &table).unwrap_or_else(|e| e);

        assert_eq!(
            response,
            CommandError::InvalidCommand("SUPERCOOLCOMMAND".to_string()).to_resp()
        )
    }
}

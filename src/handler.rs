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

pub fn handle_request(request: Vec<String>, table: &Arc<Table>) -> String {
    let mut req_iter = request.into_iter();
    let req_command = match req_iter.next() {
        Some(val) => val,
        None => return String::from("+Error\r\n"),
    };
    let command = match req_command.parse::<CommandTypes>() {
        Ok(command) => command,
        Err(_) => return String::from("+Error\r\n"),
    };
    match command {
        CommandTypes::Get => {
            let key = req_iter.next().unwrap_or_default();
            table.get(key)
        }
        CommandTypes::Set => {
            let key = req_iter.next().unwrap_or_default();
            let val = req_iter.next().unwrap_or_default();
            table.set(key, val)
        }
    }
}

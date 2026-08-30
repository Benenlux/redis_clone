use crate::table::Table;
use crate::{commands::set, error::CommandError};
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

use crate::table::Table;
use crate::utils::AsResp;
use crate::{commands::set, error::CommandError};
use std::io::{Error, ErrorKind, Write};
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

pub fn handle_request<W: Write>(
    request: Vec<String>,
    table: &Arc<Table>,
    writer: &mut W,
) -> std::io::Result<()> {
    let mut req_iter = request.into_iter();
    let req_command = match req_iter.next() {
        Some(val) => val,
        None => {
            CommandError::NoCommand.write_to(writer)?;
            return Ok(());
        }
    };
    let command = match req_command.parse::<CommandTypes>() {
        Ok(command) => command,
        Err(_) => {
            CommandError::InvalidCommand(req_command).write_to(writer)?;
            return Ok(());
        }
    };
    match command {
        CommandTypes::Get => {
            let key = match req_iter.next() {
                Some(k) => k,
                None => {
                    CommandError::InsufficientArgument("GET".to_string(), "key".to_string())
                        .write_to(writer)?;
                    return Ok(());
                }
            };

            let get_result = table.get_with(&key, |val_opt| -> std::io::Result<()> {
                if let Some(val) = val_opt {
                    val.as_resp().write_to(writer)?;
                } else {
                    writer.write_all(b"$-1\r\n")?;
                }
                Ok(())
            });
            match get_result {
                Ok(io_res) => io_res?,
                Err(err_str) => {
                    return Err(Error::new(ErrorKind::Other, err_str));
                }
            }
        }
        CommandTypes::Set => {
            set::handle_set(req_iter, table, writer)?;
        }
    }
    Ok(())
}

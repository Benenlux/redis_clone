use crate::table::Table;
use crate::{commands::set, error::CommandError};
use std::io::{IoSlice, Write};
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
) -> Result<(), String> {
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
        // CommandTypes::Get => {
        //     let key = req_iter.next().ok_or(
        //         CommandError::InsufficientArgument("GET".to_string(), "key".to_string()).to_resp(),
        //     )?;
        //     Ok(table.get(&key))
        //
        // }
        CommandTypes::Get => {
            let key = req_iter.next().ok_or_else(|| {
                CommandError::InsufficientArgument("GET".to_string(), "key".to_string()).to_resp()
            })?;

            table
                .get_with(&key, |val_opt| -> std::io::Result<()> {
                    if let Some(val) = val_opt {
                        let slices = [
                            IoSlice::new(b"+"),
                            IoSlice::new(val.as_bytes()),
                            IoSlice::new(b"\r\n"),
                        ];
                        let _amount = writer.write_vectored(&slices)?;
                    } else {
                        writer.write_all(b"$-1\r\n")?;
                    }
                    Ok(())
                })?
                .map_err(|e| e.to_string())?;
            Ok(())
        }
        CommandTypes::Set => {
            let response = set::handle_set(req_iter, table)?;
            writer
                .write_all(response.as_bytes())
                .map_err(|e| e.to_string())?;
            Ok(())
        }
    }
}

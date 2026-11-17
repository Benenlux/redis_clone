use crate::error::CommandError;
use crate::table::Table;
use crate::utils::{AsResp, RespReplies, encode_simple_string};

use std::io::Write;
use std::sync::Arc;

pub fn handle_set<W: Write>(
    mut commands: impl ExactSizeIterator<Item = String>,
    table: &Arc<Table>,
    writer: &mut W,
) -> std::io::Result<()> {
    let Some(key) = commands.next() else {
        CommandError::InsufficientArgument("SET".to_string(), "key".to_string())
            .write_to(writer)?;
        return Ok(());
    };

    let Some(val) = commands.next() else {
        CommandError::InsufficientArgument("SET".to_string(), "value".to_string())
            .write_to(writer)?;
        return Ok(());
    };

    if commands.len() == 0 {
        return table.set(key, val, writer);
    }

    handle_modified_set(key, val, commands, table, writer)?;
    Ok(())
}

//Handles conditionals and expiration modifiers
fn handle_modified_set<W: Write>(
    key: String,
    val: String,
    mut commands: impl ExactSizeIterator<Item = String>,
    table: &Arc<Table>,
    writer: &mut W,
) -> std::io::Result<()> {
    let single_conditionals = ["XX", "NX"];

    let Some(binding) = commands.next() else {
        CommandError::ParseError.write_to(writer)?;
        return Ok(());
    };
    let next_command = binding;
    let get_response = table.get(&key);
    if single_conditionals.contains(&next_command.as_str()) {
        match next_command.as_str() {
            //Can only set if key already exists
            "XX" => {
                if get_response.as_bytes() != RespReplies::NullString.to_resp() {
                    table.set(key, val, writer)
                } else {
                    writer.write_all(RespReplies::NullString.to_resp())
                }
            } //Can only set if key does not already exist
            "NX" => {
                if get_response.as_bytes() == RespReplies::NullString.to_resp() {
                    table.set(key, val, writer)
                } else {
                    writer.write_all(RespReplies::NullString.to_resp())
                }
            }
            &_ => CommandError::InvalidConditional.write_to(writer),
        }
    } else {
        next_command.as_resp().write_to(writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{RespReplies, encode_simple_string};

    /// Helper to execute `handle_set` and capture the written bytes as a `String`
    fn run_set(args: &[&str], table: &Arc<Table>) -> String {
        let mut buf = Vec::new();
        let iter = args.iter().map(|s| s.to_string());
        handle_set(iter, table, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn test_xx_conditional_nil() {
        let table = Arc::new(Table::new());

        let response_set = run_set(&["CAR", "vroom vroom"], &table);
        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());

        let response_set = run_set(&["BIKE", "tring tring", "XX"], &table);
        let response_get = table.get(&"BIKE".to_string());

        assert_eq!(response_set.as_bytes(), RespReplies::NullString.to_resp());
        assert_eq!(response_get.as_bytes(), RespReplies::NullString.to_resp());
    }

    #[test]
    fn test_xx_conditional_ok() {
        let table = Arc::new(Table::new());

        let response_set = run_set(&["CAR", "vroom vroom"], &table);
        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());

        let response_set = run_set(&["CAR", "tring tring", "XX"], &table);
        let response_get = table.get(&"CAR".to_string());

        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("tring tring"));
    }

    #[test]
    fn test_nx_conditional_nil() {
        let table = Arc::new(Table::new());

        let response_set = run_set(&["CAR", "vroom vroom"], &table);
        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());

        let response_set = run_set(&["CAR", "tring tring", "NX"], &table);
        // Note: fixed from "BIKE" to "CAR" to verify "CAR" wasn't overwritten
        let response_get = table.get(&"CAR".to_string());

        assert_eq!(response_set.as_bytes(), RespReplies::NullString.to_resp());
        assert_eq!(response_get, encode_simple_string("vroom vroom"));
    }

    #[test]
    fn test_nx_conditional_ok() {
        let table = Arc::new(Table::new());

        let response_set = run_set(&["CAR", "vroom vroom"], &table);
        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());

        let response_set = run_set(&["BIKE", "tring tring", "NX"], &table);
        let response_get = table.get(&"BIKE".to_string());

        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("tring tring"));
    }
}

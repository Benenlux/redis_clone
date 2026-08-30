use crate::error::CommandError;
use crate::table::Table;
use crate::utils::{RespReplies, encode_simple_string};

use std::sync::Arc;

pub fn handle_set(
    mut commands: impl ExactSizeIterator<Item = String>,
    table: &Arc<Table>,
) -> Result<String, String> {
    let key = commands.next().ok_or(
        CommandError::InsufficientArgument("SET".to_string(), "key".to_string()).to_resp(),
    )?;

    let val = commands.next().ok_or(
        CommandError::InsufficientArgument("SET".to_string(), "value".to_string()).to_resp(),
    )?;

    if commands.len() == 0 {
        return Ok(table.set(key, val));
    };

    handle_modified_set(key, val, commands, table)
}

//Handles conditionals and expiration modifiers
fn handle_modified_set(
    key: String,
    val: String,
    mut commands: impl ExactSizeIterator<Item = String>,
    table: &Arc<Table>,
) -> Result<String, String> {
    let single_conditionals = ["XX", "NX"];

    let binding = commands.next().ok_or(CommandError::ParseError.to_resp())?;
    let next_command = binding.as_str();
    let get_response = table.get(&key);
    if single_conditionals.contains(&next_command) {
        match next_command {
            //Can only set if key already exists
            "XX" => {
                if get_response != RespReplies::NullString.to_resp() {
                    Ok(table.set(key, val))
                } else {
                    Ok(RespReplies::NullString.to_resp())
                }
            } //Can only set if key does not already exist
            "NX" => {
                if get_response == RespReplies::NullString.to_resp() {
                    Ok(table.set(key, val))
                } else {
                    Ok(RespReplies::NullString.to_resp())
                }
            }
            &_ => Err(CommandError::InvalidConditional.to_resp()),
        }
    } else {
        Ok(encode_simple_string(format!(
            "Other extra's not implemented, got: {:?}",
            next_command,
        )))
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::{RespReplies, encode_simple_string};

    use super::*;

    #[test]
    fn test_xx_conditional_nil() {
        let table = Arc::new(Table::new());
        let commands_no_conditional =
            vec!["CAR".to_string(), "vroom vroom".to_string()].into_iter();

        let mut response_set =
            handle_set(commands_no_conditional, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, RespReplies::OKString.to_resp());

        let commands_conditional = vec![
            "BIKE".to_string(),
            "tring tring".to_string(),
            "XX".to_string(),
        ]
        .into_iter();
        response_set = handle_set(commands_conditional, &table).unwrap_or_else(|e| e.to_string());
        let response_get = table.get(&"BIKE".to_string());

        assert_eq!(response_set, RespReplies::NullString.to_resp());
        assert_eq!(response_get, RespReplies::NullString.to_resp());
    }

    #[test]
    fn test_xx_conditional_ok() {
        let table = Arc::new(Table::new());
        let commands_no_conditional =
            vec!["CAR".to_string(), "vroom vroom".to_string()].into_iter();

        let mut response_set =
            handle_set(commands_no_conditional, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, RespReplies::OKString.to_resp());

        let commands_conditional = vec![
            "CAR".to_string(),
            "tring tring".to_string(),
            "XX".to_string(),
        ]
        .into_iter();
        response_set = handle_set(commands_conditional, &table).unwrap_or_else(|e| e.to_string());
        let response_get = table.get(&"CAR".to_string());

        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("tring tring"));
    }

    #[test]
    fn test_nx_conditional_nil() {
        let table = Arc::new(Table::new());
        let commands_no_conditional =
            vec!["CAR".to_string(), "vroom vroom".to_string()].into_iter();

        let mut response_set =
            handle_set(commands_no_conditional, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, RespReplies::OKString.to_resp());

        let commands_conditional = vec![
            "CAR".to_string(),
            "tring tring".to_string(),
            "NX".to_string(),
        ]
        .into_iter();
        response_set = handle_set(commands_conditional, &table).unwrap_or_else(|e| e.to_string());
        let response_get = table.get(&"BIKE".to_string());

        assert_eq!(response_set, RespReplies::NullString.to_resp());
        assert_eq!(response_get, RespReplies::NullString.to_resp());
    }

    #[test]
    fn test_nx_conditional_ok() {
        let table = Arc::new(Table::new());
        let commands_no_conditional =
            vec!["CAR".to_string(), "vroom vroom".to_string()].into_iter();

        let mut response_set =
            handle_set(commands_no_conditional, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, RespReplies::OKString.to_resp());

        let commands_conditional = vec![
            "BIKE".to_string(),
            "tring tring".to_string(),
            "NX".to_string(),
        ]
        .into_iter();
        response_set = handle_set(commands_conditional, &table).unwrap_or_else(|e| e.to_string());
        let response_get = table.get(&"BIKE".to_string());

        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("tring tring"));
    }
}

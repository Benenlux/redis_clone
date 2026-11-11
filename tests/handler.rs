mod common;

#[cfg(test)]
mod tests {
    use redis_clone::error::CommandError;
    use redis_clone::handler::handle_request;
    use redis_clone::utils::{RespReplies, encode_simple_string};
    use std::sync::Arc;

    use crate::common::init_test;
    use redis_clone::table::Table;

    fn execute_and_capture(request: Vec<String>, table: &Arc<Table>) -> String {
        let mut writer = Vec::new();
        match handle_request(request, table, &mut writer) {
            Ok(_) => String::from_utf8(writer).expect("Response was not valid UTF-8"),
            Err(e) => e,
        }
    }

    #[test]
    fn valid_set_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");

        let response = execute_and_capture(request, &table);
        assert_eq!(response, RespReplies::OKString.to_resp());
    }

    #[test]
    fn no_set_val() {
        let (table, request) = init_test("SET CAR");

        let response = execute_and_capture(request, &table);
        assert_eq!(
            response,
            CommandError::InsufficientArgument("SET".to_string(), "value".to_string()).to_resp()
        )
    }

    #[test]
    fn valid_set_get_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");
        let response_set = execute_and_capture(request, &table);

        let request_2 = vec!["GET".to_string(), "CAR".to_string()];
        let response_get = execute_and_capture(request_2, &table);

        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("vroom vroom"));
    }

    #[test]
    fn invalid_set_get_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");
        let response_set = execute_and_capture(request, &table);

        let request_2 = vec!["GET".to_string(), "Bike".to_string()];
        let response_get = execute_and_capture(request_2, &table);

        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, RespReplies::NullString.to_resp());
    }

    #[test]
    fn only_command() {
        let (table, request) = init_test("SET ");

        let response = execute_and_capture(request, &table);
        assert_eq!(
            response,
            CommandError::InsufficientArgument("SET".to_string(), "key".to_string()).to_resp()
        )
    }

    #[test]
    fn no_command() {
        let (table, request) = init_test("");

        let response = execute_and_capture(request, &table);
        assert_eq!(response, CommandError::NoCommand.to_resp())
    }

    #[test]
    fn invalid_command() {
        let (table, request) = init_test("SUPERCOOLCOMMAND");

        let response = execute_and_capture(request, &table);
        assert_eq!(
            response,
            CommandError::InvalidCommand("SUPERCOOLCOMMAND".to_string()).to_resp()
        )
    }
}

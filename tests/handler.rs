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
        handle_request(request, table, &mut writer).expect("Failed to write to test buffer");
        String::from_utf8(writer).expect("Response was not valid UTF-8")
    }

    fn error_to_string(err: CommandError) -> String {
        let mut buf = Vec::new();
        err.write_to(&mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn valid_set_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");

        let response = execute_and_capture(request, &table);
        assert_eq!(response.as_bytes(), RespReplies::OKString.to_resp());
    }

    #[test]
    fn no_set_val() {
        let (table, request) = init_test("SET CAR");

        let response = execute_and_capture(request, &table);
        assert_eq!(
            response,
            error_to_string(CommandError::InsufficientArgument(
                "SET".to_string(),
                "value".to_string()
            ))
        );
    }

    #[test]
    fn valid_set_get_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");
        let response_set = execute_and_capture(request, &table);

        let request_2 = vec!["GET".to_string(), "CAR".to_string()];
        let response_get = execute_and_capture(request_2, &table);

        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("vroom vroom"));
    }

    #[test]
    fn invalid_set_get_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");
        let response_set = execute_and_capture(request, &table);

        let request_2 = vec!["GET".to_string(), "Bike".to_string()];
        let response_get = execute_and_capture(request_2, &table);

        assert_eq!(response_set.as_bytes(), RespReplies::OKString.to_resp());
        assert_eq!(response_get.as_bytes(), RespReplies::NullString.to_resp());
    }

    #[test]
    fn only_command() {
        let (table, request) = init_test("SET ");

        let response = execute_and_capture(request, &table);
        assert_eq!(
            response,
            error_to_string(CommandError::InsufficientArgument(
                "SET".to_string(),
                "key".to_string()
            ))
        );
    }

    #[test]
    fn no_command() {
        let (table, request) = init_test("");

        let response = execute_and_capture(request, &table);
        assert_eq!(response, error_to_string(CommandError::NoCommand));
    }

    #[test]
    fn invalid_command() {
        let (table, request) = init_test("SUPERCOOLCOMMAND");

        let response = execute_and_capture(request, &table);
        assert_eq!(
            response,
            error_to_string(CommandError::InvalidCommand("SUPERCOOLCOMMAND".to_string()))
        );
    }
}

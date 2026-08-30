mod common;

#[cfg(test)]
mod tests {

    use redis_clone::error::CommandError;
    use redis_clone::handler::handle_request;
    use redis_clone::utils::{RespReplies, encode_simple_string};

    use crate::common::init_test;

    #[test]
    fn valid_set_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");

        let response = handle_request(request, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response, RespReplies::OKString.to_resp());
    }

    #[test]
    fn no_set_val() {
        let (table, request) = init_test("SET CAR");
        let response = handle_request(request, &table).unwrap_or_else(|e| e.to_string());

        assert_eq!(
            response,
            CommandError::InsufficientArgument("SET".to_string(), "value".to_string()).to_resp()
        )
    }

    #[test]
    fn valid_set_get_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");
        let response_set = handle_request(request, &table).unwrap_or_else(|e| e.to_string());

        let request_2 = vec!["GET".to_string(), "CAR".to_string()];
        let response_get = handle_request(request_2, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, encode_simple_string("vroom vroom"));
    }
    #[test]
    fn invalid_set_get_command() {
        let (table, request) = init_test("SET CAR 'vroom vroom'");
        let response_set = handle_request(request, &table).unwrap_or_else(|e| e.to_string());

        let request_2 = vec!["GET".to_string(), "Bike".to_string()];
        let response_get = handle_request(request_2, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(response_set, RespReplies::OKString.to_resp());
        assert_eq!(response_get, RespReplies::NullString.to_resp());
    }

    #[test]
    fn only_command() {
        let (table, request) = init_test("SET ");
        let response = handle_request(request, &table).unwrap_or_else(|e| e.to_string());
        assert_eq!(
            response,
            CommandError::InsufficientArgument("SET".to_string(), "key".to_string()).to_resp()
        )
    }

    #[test]
    fn no_command() {
        let (table, request) = init_test("");

        let response = handle_request(request, &table).unwrap_or_else(|e| e);

        assert_eq!(response, CommandError::NoCommand.to_resp())
    }

    #[test]
    fn invalid_command() {
        let (table, request) = init_test("SUPERCOOLCOMMAND");
        let response = handle_request(request, &table).unwrap_or_else(|e| e);

        assert_eq!(
            response,
            CommandError::InvalidCommand("SUPERCOOLCOMMAND".to_string()).to_resp()
        )
    }
}

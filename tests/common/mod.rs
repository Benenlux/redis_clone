use redis_clone::table::Table;
use std::sync::Arc;

pub fn init_test(request: &str) -> (Arc<Table>, Vec<String>) {
    let table = Arc::new(Table::new());

    let mut request_vector = Vec::new();
    let mut current_word = String::new();
    let mut in_quotes = false;

    for c in request.chars() {
        match c {
            '\'' => {
                in_quotes = !in_quotes;
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current_word.is_empty() {
                    request_vector.push(current_word.clone());
                    current_word.clear();
                }
            }
            _ => {
                current_word.push(c);
            }
        }
    }

    if !current_word.is_empty() {
        request_vector.push(current_word);
    }

    (table, request_vector)
}

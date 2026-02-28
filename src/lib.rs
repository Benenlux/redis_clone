pub fn encode_error(message: impl Into<String>) -> String {
    format!("-ERR {}\r\n", message.into())
}

pub fn encode_simple_string(message: impl Into<String>) -> String {
    format!("+{}\r\n", message.into())
}

pub fn encode_as_array(message: impl Into<String>) -> String {
    let string_message = message.into();

    let words: Vec<&str> = string_message.split_ascii_whitespace().collect();

    let mut resp = format!("*{}\r\n", words.len());

    for word in words {
        resp.push_str(&format!("${}\r\n{}\r\n", word.len(), word));
    }

    resp
}

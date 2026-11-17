use std::io::{IoSlice, Write};

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

pub enum RespReplies {
    NullString,
    OKString,
}

impl RespReplies {
    pub fn to_resp(&self) -> &'static [u8] {
        match self {
            RespReplies::NullString => b"$-1\r\n",
            RespReplies::OKString => b"+OK\r\n",
        }
    }
}

pub struct SimpleString<'a>(pub &'a str);

impl<'a> SimpleString<'a> {
    pub fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
        let slices = [
            IoSlice::new(b"+"),
            IoSlice::new(self.0.as_bytes()),
            IoSlice::new(b"\r\n"),
        ];
        writer.write_vectored(&slices)
    }
}

pub trait AsResp {
    fn as_resp(&self) -> SimpleString<'_>;
}

impl AsResp for str {
    fn as_resp(&self) -> SimpleString<'_> {
        SimpleString(self)
    }
}

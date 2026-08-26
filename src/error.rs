use std::{
    fmt::{self, format},
    io::{self},
    string::FromUtf8Error,
};

#[derive(Debug)]
pub enum RespError {
    ConnectionClosed,
    Io(io::Error),
    InvalidProtocol(String),
    Utf8(FromUtf8Error),
}

impl fmt::Display for RespError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RespError::ConnectionClosed => write!(f, "Connection closed by peer"),
            RespError::Io(e) => write!(f, "IO Error: {}", e),
            RespError::InvalidProtocol(msg) => write!(f, "Protocol Error: {}", msg),
            RespError::Utf8(e) => write!(f, "UTF-8 Error: {}", e),
        }
    }
}

impl std::error::Error for RespError {}

impl From<io::Error> for RespError {
    fn from(error: io::Error) -> Self {
        RespError::Io(error)
    }
}

impl From<FromUtf8Error> for RespError {
    fn from(error: FromUtf8Error) -> Self {
        RespError::Utf8(error)
    }
}

#[derive(Debug)]
pub enum CommandError {
    //TODO: might be worth it to consider using a &str instead of an owned String?
    InvalidConditional,
    CommandParseError(String),
    ParseError,
    InsufficientArgument(String, String),
    InvalidCommand(String),
    NoCommand,
}

impl CommandError {
    pub fn to_resp(&self) -> String {
        match self {
            CommandError::InvalidConditional => "-ERR Invalid conditional provided\r\n".to_string(),
            CommandError::CommandParseError(command) => {
                format!("-ERR Unable to parse command: {}\r\n", command)
            }
            CommandError::ParseError => "-ERR unable to parse next command\r\n".to_string(),
            CommandError::InsufficientArgument(command, args) => {
                format!(
                    "-ERR Insufficient arguments received for '{}' command: missing {}\r\n",
                    command, args
                )
            }
            CommandError::InvalidCommand(command) => {
                format!("-ERR Invalid Command: '{}'\r\n", command)
            }
            CommandError::NoCommand => "-ERR Expected command\r\n".to_string(),
        }
    }
}
impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            //TODO: Add more conditionals as they are implemented!
            CommandError::InvalidConditional => write!(
                f,
                "Invalid conditional, valid conditionals are: XX, NX, IFEQ, IFNQ"
            ),
            CommandError::CommandParseError(command) => {
                write!(f, "Unable to parse command: {}", command)
            }
            CommandError::ParseError => {
                write!(f, "Unable to parse next command")
            }
            CommandError::InsufficientArgument(command, arg) => write!(
                f,
                "Insufficient arguments received for '{}' command: missing {}",
                command, arg
            ),
            CommandError::InvalidCommand(command) => {
                write!(f, "Received invalid command: {}", command)
            }
            CommandError::NoCommand => {
                write!(f, "Received no command, command expected")
            }
        }
    }
}

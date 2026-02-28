
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

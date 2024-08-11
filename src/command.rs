use std::str::FromStr;

pub enum Command {
    Get(String),
    Put(String, String),
    Invalid(String),
}

impl FromStr for Command {
    type Err = ();

    fn from_str(input: &str) -> Result<Command, Self::Err> {
        let parts: Vec<&str> = input.trim().splitn(3, ' ').collect();
        match parts.as_slice() {
            ["get", key] => Ok(Command::Get(key.to_string())),
            ["pub", key, value] => Ok(Command::Put(key.to_string(), value.to_string())),
            _ => Ok(Command::Invalid(input.to_string())),
        }
    }
}

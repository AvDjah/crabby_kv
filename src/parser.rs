use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Set(String, String), // key, value
    Get(String),         // key
    Delete(String),      // key
}

#[derive(Debug)]
pub struct Command {
    pub command_type: CommandType,
}

impl Command {
    pub fn new(command_type: CommandType) -> Self {
        Self { command_type }
    }
}

impl FromStr for Command {
    type Err = String;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let trimmed = line.trim();

        // Skip empty lines
        if trimmed.is_empty() {
            return Err("Empty line".to_string());
        }

        let parts: Vec<&str> = trimmed.split_whitespace().collect();

        match parts.as_slice() {
            ["SET", key, value @ ..] if !value.is_empty() => {
                // Join remaining parts as the value (handles values with spaces)
                let value_str = value.join(" ");
                Ok(Command::new(CommandType::Set(key.to_string(), value_str)))
            }
            ["GET", key] => Ok(Command::new(CommandType::Get(key.to_string()))),
            ["DELETE", key] => Ok(Command::new(CommandType::Delete(key.to_string()))),
            _ => Err(format!("Invalid command: {}", trimmed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_set_command() {
        let cmd: Command = "SET user:1001 John".parse().unwrap();
        match cmd.command_type {
            CommandType::Set(key, value) => {
                assert_eq!(key, "user:1001");
                assert_eq!(value, "John");
            }
            _ => panic!("Expected Set command"),
        }
    }

    #[test]
    fn test_parse_get_command() {
        let cmd: Command = "GET user:1001".parse().unwrap();
        match cmd.command_type {
            CommandType::Get(key) => assert_eq!(key, "user:1001"),
            _ => panic!("Expected Get command"),
        }
    }

    #[test]
    fn test_parse_delete_command() {
        let cmd: Command = "DELETE user:1001".parse().unwrap();
        match cmd.command_type {
            CommandType::Delete(key) => assert_eq!(key, "user:1001"),
            _ => panic!("Expected Delete command"),
        }
    }

    #[test]
    fn test_parse_empty_line() {
        let result: Result<Command, String> = "".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_command() {
        let result: Result<Command, String> = "INVALID command".parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_set_with_multi_word_value() {
        let cmd: Command = "SET user:1001 John Doe".parse().unwrap();
        match cmd.command_type {
            CommandType::Set(key, value) => {
                assert_eq!(key, "user:1001");
                assert_eq!(value, "John Doe");
            }
            _ => panic!("Expected Set command"),
        }
    }
}

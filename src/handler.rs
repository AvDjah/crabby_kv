use crate::parser::{Command, CommandType};
use std::collections::HashMap;

pub struct CommandHandler {
    store: HashMap<String, String>,
}

impl CommandHandler {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn process_command(&mut self, command: Command) -> Result<String, String> {
        match command.command_type {
            CommandType::Set(key, value) => self.handle_set(key, value),
            CommandType::Get(key) => self.handle_get(&key),
            CommandType::Delete(key) => self.handle_delete(key),
        }
    }

    fn handle_set(&mut self, key: String, value: String) -> Result<String, String> {
        self.store.insert(key.clone(), value.clone());
        Ok(format!("SET {} = {}", key, value))
    }

    fn handle_get(&self, key: &str) -> Result<String, String> {
        match self.store.get(key) {
            Some(value) => Ok(format!("GET {} = {}", key, value)),
            None => Err(format!("Key '{}' not found", key)),
        }
    }

    fn handle_delete(&mut self, key: String) -> Result<String, String> {
        match self.store.remove(&key) {
            Some(value) => Ok(format!("DELETED {} (was: {})", key, value)),
            None => Err(format!("Key '{}' not found", key)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Command;

    #[test]
    fn test_set_and_get() {
        let mut handler = CommandHandler::new();

        let set_cmd: Command = "SET user:1001 John".parse().unwrap();
        let result = handler.process_command(set_cmd);
        assert!(result.is_ok());

        let get_cmd: Command = "GET user:1001".parse().unwrap();
        let result = handler.process_command(get_cmd);
        assert!(result.is_ok());
        assert!(result.unwrap().contains("John"));
    }

    #[test]
    fn test_get_nonexistent_key() {
        let handler = CommandHandler::new();
        let get_cmd: Command = "GET nonexistent".parse().unwrap();
        let result = handler.process_command(get_cmd);
        assert!(result.is_err());
    }

    #[test]
    fn test_delete() {
        let mut handler = CommandHandler::new();

        let set_cmd: Command = "SET user:1001 John".parse().unwrap();
        handler.process_command(set_cmd).unwrap();

        let delete_cmd: Command = "DELETE user:1001".parse().unwrap();
        let result = handler.process_command(delete_cmd);
        assert!(result.is_ok());

        // Verify key is deleted
        let get_cmd: Command = "GET user:1001".parse().unwrap();
        let result = handler.process_command(get_cmd);
        assert!(result.is_err());
    }
}

// This file is part of ts_dynalene.
//
// Developed for the Vera C. Rubin Observatory Systems.
// This product includes software developed by the LSST Project
// (https://www.lsst.org).
// See the COPYRIGHT file at the top-level directory of this distribution
// for details of code ownership.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use log::error;
use serde_json::Value;

use crate::daq::data_acquisition::DataAcquisition;
use crate::enums::CommandStatus;
use crate::utility::get_message_name;
use crate::utility::{acknowledge_command, get_message_sequence_id};

pub trait Command {
    /// Get the name of the command.
    ///
    /// # Returns
    /// Command name.
    fn name(&self) -> &str;

    /// Execute the command.
    ///
    /// # Arguments
    /// * `message` - Command message to execute.
    /// * `data_acquisition` - Data acquisition to execute the command.
    ///
    /// # Returns
    /// Command execution result. If failed, returns the failure reason.
    fn execute(
        &self,
        message: Value,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String>;
}

#[derive(Default)]
pub struct CommandSchema {
    // List of commands.
    pub commands: Vec<Box<dyn Command + Send>>,
}

impl CommandSchema {
    /// Create a new command schema.
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Add a command to the schema.
    pub fn add_command(&mut self, command: Box<dyn Command + Send>) {
        self.commands.push(command);
    }

    /// Execute a command.
    ///
    /// # Arguments
    /// * `message` - Command message to execute.
    /// * `data_acquisition` - Data acquisition to execute the command.
    ///
    /// # Returns
    /// Command execution result.
    pub fn execute(&self, message: Value, data_acquisition: Option<&mut DataAcquisition>) -> Value {
        let name = get_message_name(&message);
        let sequence_id = get_message_sequence_id(&message);
        for cmd in &self.commands {
            if cmd.name() == name {
                match cmd.execute(message, data_acquisition) {
                    Ok(_) => {
                        return acknowledge_command(
                            Some(&name),
                            CommandStatus::Success,
                            sequence_id,
                            None,
                        );
                    }
                    Err(fail_reason) => {
                        error!("Command execution failed: {fail_reason}");

                        return acknowledge_command(
                            Some(&name),
                            CommandStatus::Fail,
                            sequence_id,
                            Some(&fail_reason),
                        );
                    }
                }
            }
        }

        error!("Unknown command: {message}.");

        acknowledge_command(
            Some(&name),
            CommandStatus::Fail,
            sequence_id,
            Some("Unknown command."),
        )
    }

    /// Get the number of commands.
    ///
    /// # Returns
    /// Number of commands.
    pub fn number_of_commands(&self) -> usize {
        self.commands.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    struct CommandTest;
    impl Command for CommandTest {
        fn name(&self) -> &str {
            "cmd_test"
        }

        fn execute(
            &self,
            message: Value,
            _data_acquisition: Option<&mut DataAcquisition>,
        ) -> Result<(), String> {
            if message["parameters"]["status"].as_bool().unwrap_or(false) {
                return Ok(());
            }

            Err(String::from("Test command failed."))
        }
    }

    fn create_command_schema() -> CommandSchema {
        let mut schema = CommandSchema::new();
        schema.add_command(Box::new(CommandTest));

        schema
    }

    #[test]
    fn test_new() {
        let schema = create_command_schema();

        assert_eq!(schema.commands.len(), 1);
    }

    #[test]
    fn test_execute_fail() {
        let schema = create_command_schema();

        // Not a valid command.
        let result = schema.execute(json!({"id": "cmd_fail", "sequence_id": 1}), None);

        assert_eq!(
            result.to_string(),
            r#"{"fail_reason":"Unknown command.","id":"cmd_fail","sequence_id":1,"state":"fail"}"#
        );

        // No parameters field.
        let result = schema.execute(json!({"id": "cmd_test", "sequence_id": 2}), None);

        assert_eq!(
            result.to_string(),
            r#"{"fail_reason":"Test command failed.","id":"cmd_test","sequence_id":2,"state":"fail"}"#
        );

        // No status field within parameters.
        let result = schema.execute(
            json!({"id": "cmd_test", "sequence_id": 3, "parameters": {}}),
            None,
        );

        assert_eq!(
            result.to_string(),
            r#"{"fail_reason":"Test command failed.","id":"cmd_test","sequence_id":3,"state":"fail"}"#
        );

        // Status is false.
        let result = schema.execute(
            json!({"id": "cmd_test", "sequence_id": 4, "parameters": {"status": false}}),
            None,
        );

        assert_eq!(
            result.to_string(),
            r#"{"fail_reason":"Test command failed.","id":"cmd_test","sequence_id":4,"state":"fail"}"#
        );
    }

    #[test]
    fn test_execute_success() {
        let schema = create_command_schema();

        // Has the sequence_id field.
        let result = schema.execute(
            json!({"id": "cmd_test", "sequence_id": 1, "parameters": {"status": true}}),
            None,
        );

        assert_eq!(
            result.to_string(),
            r#"{"id":"cmd_test","sequence_id":1,"state":"success"}"#
        );

        // No sequence_id field.
        let result = schema.execute(
            json!({"id": "cmd_test", "parameters": {"status": true}}),
            None,
        );

        assert_eq!(
            result.to_string(),
            r#"{"id":"cmd_test","sequence_id":-1,"state":"success"}"#
        );
    }

    #[test]
    fn test_number_of_commands() {
        let schema = create_command_schema();

        assert_eq!(schema.number_of_commands(), 1);
    }
}

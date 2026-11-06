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
use std::collections::HashMap;

use crate::command::command_format::{CommandFormat, CommandId};
use crate::daq::data_acquisition::DataAcquisition;
use crate::enums::CommandStatus;
use crate::utility::acknowledge_command;

pub trait Command {
    /// Get the ID of the command.
    ///
    /// # Returns
    /// Command ID.
    fn id(&self) -> CommandId;

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
        message: &CommandFormat,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String>;
}

#[derive(Default)]
pub struct CommandSchema {
    // HashMap of commands.
    pub commands: HashMap<CommandId, Box<dyn Command + Send>>,
}

impl CommandSchema {
    /// Create a new command schema.
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Add a command to the schema.
    pub fn add_command(&mut self, command: Box<dyn Command + Send>) {
        self.commands.insert(command.id(), command);
    }

    /// Execute a command.
    ///
    /// # Arguments
    /// * `message` - Command message to execute.
    /// * `data_acquisition` - Data acquisition to execute the command.
    ///
    /// # Returns
    /// Command execution result.
    pub fn execute(
        &self,
        message: &CommandFormat,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Value {
        let tag_id = message.payload.get_tag_id();
        let sequence_id = message.sequence_id;
        if let Some(command) = self.commands.get(&CommandId::from(&message.payload)) {
            return match command.execute(message, data_acquisition) {
                Ok(_) => {
                    acknowledge_command(Some(tag_id), CommandStatus::Success, sequence_id, None)
                }
                Err(fail_reason) => {
                    error!("Command execution failed: {fail_reason}");

                    acknowledge_command(
                        Some(tag_id),
                        CommandStatus::Fail,
                        sequence_id,
                        Some(&fail_reason),
                    )
                }
            };
        }

        error!("Unknown command: {tag_id}.");

        acknowledge_command(
            Some(tag_id),
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
    use serde_json::{from_value, json};

    use crate::command::command_data_acquisition::{
        CommandResetPierFan, CommandSwitchDigitalOutput,
    };
    use crate::daq::data_acquisition::DataAcquisition;

    fn create_command_schema() -> CommandSchema {
        let mut schema = CommandSchema::new();
        schema.add_command(Box::new(CommandSwitchDigitalOutput));
        schema.add_command(Box::new(CommandResetPierFan));

        schema
    }

    #[test]
    fn test_new() {
        let schema = create_command_schema();

        assert_eq!(schema.commands.len(), 2);
    }

    #[test]
    fn test_execute_fail() {
        let schema = create_command_schema();

        // Not a valid command.
        let result = schema.execute(
            &from_value(json!({
                "id": "cmd_openPressureControlValve",
                "sequence_id": 1,
                "parameters": {"idx": 0, "percentage": 50.0}
            }))
            .unwrap(),
            None,
        );

        assert_eq!(
            result.to_string(),
            r#"{"fail_reason":"Unknown command.","id":"cmd_openPressureControlValve","sequence_id":1,"state":"fail"}"#
        );

        // No subsystem to execute the command.
        let result = schema.execute(
            &from_value(json!({
                "id": "cmd_switchDigitalOutput",
                "sequence_id": 2,
                "parameters": {"bit": 0b100000000, "status": true}
            }))
            .unwrap(),
            None,
        );

        assert_eq!(
            result.to_string(),
            r#"{"fail_reason":"Missing data acquisition.","id":"cmd_switchDigitalOutput","sequence_id":2,"state":"fail"}"#
        );

        // Failed execution in the subsystem.
        let mut data_acquisition = DataAcquisition::new(true);

        let result = schema.execute(
            &from_value(json!({
                "id": "cmd_resetPierFan",
                "sequence_id": 3,
                "parameters": {"idx": 3}
            }))
            .unwrap(),
            Some(&mut data_acquisition),
        );

        assert_eq!(
            result.to_string(),
            r#"{"fail_reason":"Invalid pier fan index.","id":"cmd_resetPierFan","sequence_id":3,"state":"fail"}"#
        );
    }

    #[test]
    fn test_execute_success() {
        let schema = create_command_schema();
        let mut data_acquisition = DataAcquisition::new(true);

        // Has the sequence_id field.
        let result = schema.execute(
            &from_value(json!({
                "id": "cmd_switchDigitalOutput",
                "sequence_id": 1,
                "parameters": {"bit": 0b100000000, "status": true}
            }))
            .unwrap(),
            Some(&mut data_acquisition),
        );

        assert_eq!(
            result.to_string(),
            r#"{"id":"cmd_switchDigitalOutput","sequence_id":1,"state":"success"}"#
        );

        // Internal command (sequence_id = -1).
        let result = schema.execute(
            &from_value(json!({
                "id": "cmd_switchDigitalOutput",
                "sequence_id": -1,
                "parameters": {"bit": 0b100000000, "status": true}
            }))
            .unwrap(),
            Some(&mut data_acquisition),
        );

        assert_eq!(
            result.to_string(),
            r#"{"id":"cmd_switchDigitalOutput","sequence_id":-1,"state":"success"}"#
        );
    }

    #[test]
    fn test_number_of_commands() {
        let schema = create_command_schema();

        assert_eq!(schema.number_of_commands(), 2);
    }
}

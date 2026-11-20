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

use crate::command::{
    command_format::{CommandFormat, CommandId, CommandPayload},
    command_schema::Command,
};
use crate::controller::Controller;
use crate::daq::data_acquisition::DataAcquisition;
use crate::enums::Commander;

/// Command to switch the command source: GUI or CSC.
pub struct CommandSwitchCommandSource;
impl Command for CommandSwitchCommandSource {
    fn id(&self) -> CommandId {
        CommandId::SwitchCommandSource
    }

    fn execute(
        &self,
        message: &CommandFormat,
        _data_acquisition: Option<&mut DataAcquisition>,
        controller: Option<&mut Controller>,
    ) -> Result<(), String> {
        if let CommandPayload::SwitchCommandSource(parameters) = &message.payload {
            let commander = if parameters.remote {
                Commander::CSC
            } else {
                Commander::GUI
            };
            let system_controller = controller.ok_or("Missing controller.")?;
            system_controller.set_commander(commander)
        } else {
            Err(String::from("Unexpected command payload."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json};

    #[test]
    fn test_command_switch_command_source() {
        let mut controller = Controller::new();

        let command = CommandSwitchCommandSource;

        assert_eq!(command.id(), CommandId::SwitchCommandSource);
        assert!(
            command
                .execute(
                    &from_value(json!({
                        "id": "cmd_switchCommandSource",
                        "sequence_id": 1,
                        "parameters": {"remote": false}
                    }))
                    .unwrap(),
                    None,
                    Some(&mut controller),
                )
                .is_ok()
        );

        assert!(controller.commander == Commander::GUI);
    }
}

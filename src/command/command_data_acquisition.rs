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
use crate::daq::data_acquisition::DataAcquisition;
use crate::enums::DigitalOutput;

/// Command to switch the digital output.
pub struct CommandSwitchDigitalOutput;
impl Command for CommandSwitchDigitalOutput {
    fn id(&self) -> CommandId {
        CommandId::SwitchDigitalOutput
    }

    fn execute(
        &self,
        message: &CommandFormat,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        if let CommandPayload::SwitchDigitalOutput(parameters) = &message.payload {
            let digital_output =
                DigitalOutput::from_repr(parameters.bit).ok_or("Invalid digital output.")?;

            let system = data_acquisition.ok_or("Missing data acquisition.")?;
            system.switch_digital_output(digital_output, parameters.status)
        } else {
            Err(String::from("Unexpected command payload."))
        }
    }
}

/// Command to open the pressure control valve (PCV).
pub struct CommandOpenPressureControlValve;
impl Command for CommandOpenPressureControlValve {
    fn id(&self) -> CommandId {
        CommandId::OpenPressureControlValve
    }

    fn execute(
        &self,
        message: &CommandFormat,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        if let CommandPayload::OpenPressureControlValve(parameters) = &message.payload {
            let system = data_acquisition.ok_or("Missing data acquisition.")?;
            system.open_pressure_control_valve(parameters.idx, parameters.percentage)
        } else {
            Err(String::from("Unexpected command payload."))
        }
    }
}

/// Command to open the control mixing valve (CMV).
pub struct CommandOpenControlMixingValve;
impl Command for CommandOpenControlMixingValve {
    fn id(&self) -> CommandId {
        CommandId::OpenControlMixingValve
    }

    fn execute(
        &self,
        message: &CommandFormat,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        if let CommandPayload::OpenControlMixingValve(parameters) = &message.payload {
            let system = data_acquisition.ok_or("Missing data acquisition.")?;
            system.open_control_mixing_valve(parameters.idx, parameters.percentage)
        } else {
            Err(String::from("Unexpected command payload."))
        }
    }
}

/// Command to reset the pier fan.
pub struct CommandResetPierFan;
impl Command for CommandResetPierFan {
    fn id(&self) -> CommandId {
        CommandId::ResetPierFan
    }

    fn execute(
        &self,
        message: &CommandFormat,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        if let CommandPayload::ResetPierFan(parameters) = &message.payload {
            let system = data_acquisition.ok_or("Missing data acquisition.")?;
            system.reset_pier_fan(parameters.idx)
        } else {
            Err(String::from("Unexpected command payload."))
        }
    }
}

/// Command to set the chiller temperature.
pub struct CommandSetChillerTemperature;
impl Command for CommandSetChillerTemperature {
    fn id(&self) -> CommandId {
        CommandId::SetChillerTemperature
    }

    fn execute(
        &self,
        message: &CommandFormat,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        if let CommandPayload::SetChillerTemperature(parameters) = &message.payload {
            let system = data_acquisition.ok_or("Missing data acquisition.")?;
            system.set_chiller_temperature(parameters.idx, parameters.temperature)
        } else {
            Err(String::from("Unexpected command payload."))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_value, json};

    use ts_control_utils::enums::BitEnum;

    #[test]
    fn test_command_switch_digital_output() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandSwitchDigitalOutput;

        assert_eq!(command.id(), CommandId::SwitchDigitalOutput);
        assert!(
            command
                .execute(
                    &from_value(json!({
                        "id": "cmd_switchDigitalOutput",
                        "sequence_id": 1,
                        "parameters": {"bit": 0b100000000, "status": true}
                    }))
                    .unwrap(),
                    Some(&mut data_acquisition),
                )
                .is_ok()
        );

        assert!(
            data_acquisition.plant.unwrap().digital_outputs
                & DigitalOutput::PowerPierFan1.bit_value()
                != 0
        );
    }

    #[test]
    fn test_command_open_pressure_control_valve() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandOpenPressureControlValve;

        assert_eq!(command.id(), CommandId::OpenPressureControlValve);
        assert!(
            command
                .execute(
                    &from_value(json!({
                        "id": "cmd_openPressureControlValve",
                        "sequence_id": 1,
                        "parameters": {"idx": 0, "percentage": 50.0}
                    }))
                    .unwrap(),
                    Some(&mut data_acquisition),
                )
                .is_ok()
        );
    }

    #[test]
    fn test_command_open_control_mixing_valve() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandOpenControlMixingValve;

        assert_eq!(command.id(), CommandId::OpenControlMixingValve);
        assert!(
            command
                .execute(
                    &from_value(json!({
                        "id": "cmd_openControlMixingValve",
                        "sequence_id": 1,
                        "parameters": {"idx": 0, "percentage": 50.0}
                    }))
                    .unwrap(),
                    Some(&mut data_acquisition),
                )
                .is_ok()
        );
    }

    #[test]
    fn test_command_reset_pier_fan() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandResetPierFan;

        assert_eq!(command.id(), CommandId::ResetPierFan);
        assert!(
            command
                .execute(
                    &from_value(json!({
                        "id": "cmd_resetPierFan",
                        "sequence_id": 1,
                        "parameters": {"idx": 0}
                    }))
                    .unwrap(),
                    Some(&mut data_acquisition),
                )
                .is_ok()
        );
    }

    #[test]
    fn test_command_set_chiller_temperature() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandSetChillerTemperature;

        assert_eq!(command.id(), CommandId::SetChillerTemperature);
        assert_eq!(
            command.execute(
                &from_value(json!({
                    "id": "cmd_setChillerTemperature",
                    "sequence_id": 1,
                    "parameters": {"idx": 0, "temperature": 25.0}
                }))
                .unwrap(),
                Some(&mut data_acquisition),
            ),
            Err(String::from("Chiller is not enabled.")),
        );
    }
}

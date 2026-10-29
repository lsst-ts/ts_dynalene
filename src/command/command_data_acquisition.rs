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

use serde::Deserialize;
use serde_json::{Value, from_value};

use crate::command::command_schema::Command;
use crate::daq::data_acquisition::DataAcquisition;
use crate::enums::DigitalOutput;

#[derive(Deserialize, PartialEq)]
#[serde(tag = "id", content = "parameters")]
pub enum CommandPayload {
    #[serde(rename = "cmd_switchDigitalOutput")]
    SwitchDigitalOutput(ParametersSwitchDigitalOutput),
    #[serde(rename = "cmd_openPressureControlValve")]
    OpenPressureControlValve(ParametersOpenPressureControlValve),
    #[serde(rename = "cmd_openControlMixingValve")]
    OpenControlMixingValve(ParametersOpenControlMixingValve),
    #[serde(rename = "cmd_resetPierFan")]
    ResetPierFan(ParametersResetPierFan),
    #[serde(rename = "cmd_setChillerTemperature")]
    SetChillerTemperature(ParametersSetChillerTemperature),
}

#[derive(Deserialize, PartialEq)]
pub struct ParametersSwitchDigitalOutput {
    // Bit value. For example, 0b1000 (= 8) represents the third bit.
    pub bit: u32,
    // True if the digital output should be switched on, false otherwise.
    pub status: bool,
}

#[derive(Deserialize, PartialEq)]
pub struct ParametersOpenPressureControlValve {
    // The index of the pressure control valve (0-1)
    pub idx: usize,
    // The percentage to open the valve (0.0 to 100.0). 0.0 means fully closed.
    // 100.0 means fully open to allow all coolant goes through it.
    pub percentage: f64,
}

#[derive(Deserialize, PartialEq)]
pub struct ParametersOpenControlMixingValve {
    // The index of the control mixing valve (0-2)
    pub idx: usize,
    // The percentage to open the valve (0.0 to 100.0). 0.0 means fully closed
    // (recirculating for the upper loop). 100.0 means fully open to the
    // return.
    pub percentage: f64,
}

#[derive(Deserialize, PartialEq)]
pub struct ParametersResetPierFan {
    // The index of the pier fan (0-1)
    pub idx: usize,
}

#[derive(Deserialize, PartialEq)]
pub struct ParametersSetChillerTemperature {
    // The index of the chiller (0-1)
    pub idx: usize,
    // The temperature to set for the chiller (-17.78 to 121.11) in Celsius.
    // The range is 0 to 250 Fahrenheit after conversion in the instrument.
    pub temperature: f32,
}

#[derive(Deserialize, PartialEq)]
pub struct CommandFormat {
    // Sequence ID for the command.
    pub sequence_id: i64,

    // The payload of the command.
    #[serde(flatten)]
    pub payload: CommandPayload,
}

/// Command to switch the digital output.
pub struct CommandSwitchDigitalOutput;
impl Command for CommandSwitchDigitalOutput {
    fn name(&self) -> &str {
        "cmd_switchDigitalOutput"
    }

    fn execute(
        &self,
        message: Value,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        let command: CommandFormat = from_value(message).map_err(|error| error.to_string())?;

        if let CommandPayload::SwitchDigitalOutput(parameters) = command.payload {
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
    fn name(&self) -> &str {
        "cmd_openPressureControlValve"
    }

    fn execute(
        &self,
        message: Value,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        let command: CommandFormat = from_value(message).map_err(|error| error.to_string())?;

        if let CommandPayload::OpenPressureControlValve(parameters) = command.payload {
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
    fn name(&self) -> &str {
        "cmd_openControlMixingValve"
    }

    fn execute(
        &self,
        message: Value,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        let command: CommandFormat = from_value(message).map_err(|error| error.to_string())?;

        if let CommandPayload::OpenControlMixingValve(parameters) = command.payload {
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
    fn name(&self) -> &str {
        "cmd_resetPierFan"
    }

    fn execute(
        &self,
        message: Value,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        let command: CommandFormat = from_value(message).map_err(|error| error.to_string())?;

        if let CommandPayload::ResetPierFan(parameters) = command.payload {
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
    fn name(&self) -> &str {
        "cmd_setChillerTemperature"
    }

    fn execute(
        &self,
        message: Value,
        data_acquisition: Option<&mut DataAcquisition>,
    ) -> Result<(), String> {
        let command: CommandFormat = from_value(message).map_err(|error| error.to_string())?;

        if let CommandPayload::SetChillerTemperature(parameters) = command.payload {
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
    use serde_json::json;

    use ts_control_utils::enums::BitEnum;

    #[test]
    fn test_command_switch_digital_output() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandSwitchDigitalOutput;

        assert_eq!(command.name(), "cmd_switchDigitalOutput");
        assert!(
            command
                .execute(
                    json!({
                        "id": "cmd_switchDigitalOutput",
                        "sequence_id": 1,
                        "parameters": {"bit": 0b100000000, "status": true}
                    }),
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

        assert_eq!(command.name(), "cmd_openPressureControlValve");
        assert!(
            command
                .execute(
                    json!({
                        "id": "cmd_openPressureControlValve",
                        "sequence_id": 1,
                        "parameters": {"idx": 0, "percentage": 50.0}
                    }),
                    Some(&mut data_acquisition),
                )
                .is_ok()
        );
    }

    #[test]
    fn test_command_open_control_mixing_valve() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandOpenControlMixingValve;

        assert_eq!(command.name(), "cmd_openControlMixingValve");
        assert!(
            command
                .execute(
                    json!({
                        "id": "cmd_openControlMixingValve",
                        "sequence_id": 1,
                        "parameters": {"idx": 0, "percentage": 50.0}
                    }),
                    Some(&mut data_acquisition),
                )
                .is_ok()
        );
    }

    #[test]
    fn test_command_reset_pier_fan() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandResetPierFan;

        assert_eq!(command.name(), "cmd_resetPierFan");
        assert!(
            command
                .execute(
                    json!({
                        "id": "cmd_resetPierFan",
                        "sequence_id": 1,
                        "parameters": {"idx": 0}
                    }),
                    Some(&mut data_acquisition),
                )
                .is_ok()
        );
    }

    #[test]
    fn test_command_set_chiller_temperature() {
        let mut data_acquisition = DataAcquisition::new(true);

        let command = CommandSetChillerTemperature;

        assert_eq!(command.name(), "cmd_setChillerTemperature");
        assert_eq!(
            command.execute(
                json!({
                    "id": "cmd_setChillerTemperature",
                    "sequence_id": 1,
                    "parameters": {"idx": 0, "temperature": 25.0}
                }),
                Some(&mut data_acquisition),
            ),
            Err(String::from("Chiller is not enabled.")),
        );
    }
}

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
use strum_macros::{EnumDiscriminants, IntoStaticStr};

#[derive(Deserialize, PartialEq)]
pub struct CommandFormat {
    // Sequence ID for the command.
    pub sequence_id: i64,

    // The payload of the command.
    #[serde(flatten)]
    pub payload: CommandPayload,
}

#[derive(Deserialize, PartialEq, EnumDiscriminants, IntoStaticStr)]
#[serde(tag = "id", content = "parameters")]
#[strum_discriminants(name(CommandId), derive(Hash))]
pub enum CommandPayload {
    // We have the repeated "cmd_xxx" string here because the serde and strum
    // macros require the exact literal string.
    #[serde(rename = "cmd_switchDigitalOutput")]
    #[strum(serialize = "cmd_switchDigitalOutput")]
    SwitchDigitalOutput(ParametersSwitchDigitalOutput),
    #[serde(rename = "cmd_openPressureControlValve")]
    #[strum(serialize = "cmd_openPressureControlValve")]
    OpenPressureControlValve(ParametersOpenPressureControlValve),
    #[serde(rename = "cmd_openControlMixingValve")]
    #[strum(serialize = "cmd_openControlMixingValve")]
    OpenControlMixingValve(ParametersOpenControlMixingValve),
    #[serde(rename = "cmd_resetPierFan")]
    #[strum(serialize = "cmd_resetPierFan")]
    ResetPierFan(ParametersResetPierFan),
    #[serde(rename = "cmd_setChillerTemperature")]
    #[strum(serialize = "cmd_setChillerTemperature")]
    SetChillerTemperature(ParametersSetChillerTemperature),
}

impl CommandPayload {
    /// Get the tag ID of the command payload.
    ///
    /// # Notes
    /// The deserialization of `CommandPayload` consumes the "id" field to
    /// determine the variant. Since we need this information later, we provide
    /// a method to retrieve it by the strum macro.
    ///
    /// # Returns
    /// The tag ID of the command payload as a string slice.
    pub fn get_tag_id(&self) -> &'static str {
        self.into()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tag_id() {
        let payload = CommandPayload::SwitchDigitalOutput(ParametersSwitchDigitalOutput {
            bit: 8,
            status: true,
        });
        assert_eq!(payload.get_tag_id(), "cmd_switchDigitalOutput");

        let payload =
            CommandPayload::OpenPressureControlValve(ParametersOpenPressureControlValve {
                idx: 0,
                percentage: 50.0,
            });
        assert_eq!(payload.get_tag_id(), "cmd_openPressureControlValve");

        let payload = CommandPayload::OpenControlMixingValve(ParametersOpenControlMixingValve {
            idx: 1,
            percentage: 75.0,
        });
        assert_eq!(payload.get_tag_id(), "cmd_openControlMixingValve");

        let payload = CommandPayload::ResetPierFan(ParametersResetPierFan { idx: 0 });
        assert_eq!(payload.get_tag_id(), "cmd_resetPierFan");

        let payload = CommandPayload::SetChillerTemperature(ParametersSetChillerTemperature {
            idx: 1,
            temperature: 25.0,
        });
        assert_eq!(payload.get_tag_id(), "cmd_setChillerTemperature");
    }
}

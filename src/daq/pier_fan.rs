// This file is part of ts_dynalene.
//
// Developed for the Vera Rubin Observatory Systems.
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
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::constants::{
    MAX_VALUE_PIER_FAN_ACTUAL_SPEED, MAX_VALUE_PIER_FAN_DC_LINK_VOLTAGE_CURRENT,
    NUM_REGISTER_PIER_FAN_ACTUAL_SPEED,
};
use crate::utility::get_values_from_u8_array;

#[derive(Debug, PartialEq)]
pub struct PierFan {
    // Address of the pier fan.
    pub address: u8,
    // The unit is rpm.
    pub actual_speed: f32,
    // See the `MotorStatusPierFan` enum in enums.rs.
    pub motor_status: u16,
    // See the `WarningPierFan` enum in enums.rs.
    pub warning: u16,
    // DC-link voltage in volts.
    pub dc_link_voltage: f32,
    // DC-link current in amperes.
    pub dc_link_current: f32,
    // The unit is degrees Celsius.
    pub module_temperature: u16,
    // The unit is degrees Celsius.
    pub motor_temperature: i16,
    // The unit is degrees Celsius.
    pub electronics_temperature: u16,
    // 0: anti-clockwise, 1: clockwise.
    pub current_direction_of_rotation: u16,
    // The unit is percentage.
    pub current_modulation_level: f32,
    // The set value denotes a speed: the unit is rpm.
    pub current_set_value: f32,
    // 0: Enable off (motor stop), 1: Enable on (motor start allowed).
    pub enable_input_status: u16,
    // This parameter specifies which parameter set (1 or 2) is currently in
    // use.
    //
    // If the "Source parameter set" parameter has the value "Digital input
    // Din2" (0) or "Digital input Din3" (2), the status of the external input
    // "parameter set" is shown here.
    // If the "Source parameter set" parameter has the value "internal" (1),
    // the value of the parameter "Internal parameter set" is shown here.
    //
    // 0: Parameter set 1, 1: Parameter set 2.
    pub current_parameter_set: u16,
    // If the "Source of controller function" parameter has the value "Digital
    // input Din3" (0) or "Digital input Din2" (2), the status of the external
    // "Controller function" input is shown here.
    // If the "Source of controller function" parameter has the value
    // "internal" (1), the value of the "Controller function" parameter is
    // shown here.
    //
    // For closed-loop sensor control via temperature sensor, a positive
    // controller function is synonymous with "heat" and a negative controller
    // function is synonymous with "cool".
    //
    // 0: Positive -> Control variable = set value - actual value
    // 1: Negative -> Control variable = actual value - set value
    pub current_controller_function: u16,
    // The unit is watts.
    pub current_power: f32,
}

impl PierFan {
    /// Pier fan to have the measured operational values.
    ///
    /// # Arguments
    /// * `address` - The address of the pier fan.
    ///
    /// # Returns
    /// A new instance of `PierFan`.
    pub fn new(address: u8) -> Self {
        Self {
            address,

            actual_speed: 0.0,
            motor_status: 0,
            warning: 0,
            dc_link_voltage: 0.0,
            dc_link_current: 0.0,
            module_temperature: 0,
            motor_temperature: 0,
            electronics_temperature: 0,
            current_direction_of_rotation: 0,
            current_modulation_level: 0.0,
            current_set_value: 0.0,
            enable_input_status: 0,
            current_parameter_set: 0,
            current_controller_function: 0,
            current_power: 0.0,
        }
    }

    /// Create a `PierFan` instance from a Modbus frame.
    ///
    /// # Arguments
    /// * `max_speed` - The maximum speed of the pier fan in rpm.
    /// * `ref_dc_link_voltage` - The reference DC link voltage in volts. To
    ///   keep the resolution variable, all values for the DC-link voltage are
    ///   based on this reference value. Note the raw register value is in mV.
    /// * `ref_dc_link_current` - The reference DC link current in amperes. To
    ///   keep the resolution variable, all values for the DC-link current are
    ///   based on this reference value. Note the raw register value is in mA.
    /// * `frame` - The Modbus frame containing the pier fan data.
    ///
    /// # Returns
    /// An `Option` containing the `PierFan` if the frame is valid, or
    /// `None` otherwise.
    pub fn from_frame(
        max_speed: f32,
        ref_dc_link_voltage: f32,
        ref_dc_link_current: f32,
        frame: &[u8],
    ) -> Option<PierFan> {
        const DATA_BYTES_PIER_FAN: usize = 2 * (NUM_REGISTER_PIER_FAN_ACTUAL_SPEED as usize);
        const FRAME_LENGTH_PIER_FAN: usize = 5 + DATA_BYTES_PIER_FAN;
        if (frame.len() != FRAME_LENGTH_PIER_FAN) || (frame[2] != (DATA_BYTES_PIER_FAN as u8)) {
            return None;
        }

        let address = frame[0];

        const NUM_VALUE_PIER_FAN: usize = NUM_REGISTER_PIER_FAN_ACTUAL_SPEED as usize;
        let values = get_values_from_u8_array::<u16, NUM_VALUE_PIER_FAN>(
            &frame[3..3 + DATA_BYTES_PIER_FAN],
        )?;

        let actual_speed = (values[0] as f32 / MAX_VALUE_PIER_FAN_ACTUAL_SPEED as f32) * max_speed;

        let dc_link_voltage = (values[3] as f32
            / MAX_VALUE_PIER_FAN_DC_LINK_VOLTAGE_CURRENT as f32)
            * ref_dc_link_voltage;
        let dc_link_current = (values[4] as f32
            / MAX_VALUE_PIER_FAN_DC_LINK_VOLTAGE_CURRENT as f32)
            * ref_dc_link_current;

        // This value is a signed 16-bit integer.
        let motor_temperature = values[6] as i16;

        let current_modulation_level = (values[9] as f32 / u16::MAX as f32) * 100.0;

        // For the current set value, we are in closed-loop speed control.
        // The value zero means motor standstill.
        let current_set_value =
            (values[10] as f32 / MAX_VALUE_PIER_FAN_ACTUAL_SPEED as f32) * max_speed;

        let current_power =
            (values[17] as f32 / u16::MAX as f32) * ref_dc_link_voltage * ref_dc_link_current;

        Some(PierFan {
            address,
            actual_speed,
            motor_status: values[1],
            warning: values[2],
            dc_link_voltage,
            dc_link_current,
            module_temperature: values[5],
            motor_temperature,
            electronics_temperature: values[7],
            current_direction_of_rotation: values[8],
            current_modulation_level,
            current_set_value,
            enable_input_status: values[12],
            current_parameter_set: values[13],
            current_controller_function: values[14],
            current_power,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_frame_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 40] = [0; 40];
        assert!(PierFan::from_frame(0.0, 0.0, 0.0, &frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 41] = [0; 41];
        frame_wrong_data_bytes[2] = 35;
        assert!(PierFan::from_frame(0.0, 0.0, 0.0, &frame_wrong_data_bytes).is_none());
    }
}

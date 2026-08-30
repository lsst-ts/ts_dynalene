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

use crc::{CRC_16_MODBUS, Crc};
use ts_control_utils::enums::BitEnum;

use crate::constants::{
    CODE_READ_HOLDING_REGISTERS, FACTOR_PIER_FAN_DC_LINK_REFERENCE_CURRENT,
    FACTOR_PIER_FAN_DC_LINK_REFERENCE_VOLTAGE, MAX_VALUE_PIER_FAN_ACTUAL_SPEED,
    MAX_VALUE_PIER_FAN_DC_LINK_VOLTAGE_CURRENT, REGISTER_ADDRESS_PIER_FAN_ACTUAL_SPEED,
    REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED,
    REGISTER_ADDRESS_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE,
};
use crate::daq::pier_fan::PierFan;
use crate::enums::{MotorStatusPierFan, WarningPierFan};
use crate::mock::mock_constants::{
    PLANT_PIER_FAN_ACTUAL_SPEED, PLANT_PIER_FAN_CURRENT_CONTROLLER_FUNCTION,
    PLANT_PIER_FAN_CURRENT_DIRECTION_OF_ROTATION, PLANT_PIER_FAN_CURRENT_MODULATION_LEVEL,
    PLANT_PIER_FAN_CURRENT_PARAMETER_SET, PLANT_PIER_FAN_CURRENT_POWER,
    PLANT_PIER_FAN_CURRENT_SET_VALUE, PLANT_PIER_FAN_DC_LINK_CURRENT,
    PLANT_PIER_FAN_DC_LINK_VOLTAGE, PLANT_PIER_FAN_ELECTRONICS_TEMPERATURE,
    PLANT_PIER_FAN_ENABLE_INPUT_STATUS, PLANT_PIER_FAN_MAX_SPEED,
    PLANT_PIER_FAN_MODULE_TEMPERATURE, PLANT_PIER_FAN_MOTOR_TEMPERATURE,
    PLANT_PIER_FAN_REF_DC_LINK_CURRENT, PLANT_PIER_FAN_REF_DC_LINK_VOLTAGE,
};
use crate::utility::calculate_modbus_crc_and_update_frame;

pub struct MockPierFan {
    // Cyclic redundancy check (CRC) calculator for the Modbus communication.
    _crc: Crc<u16>,
    // The maximum speed of the pier fan in rpm.
    pub max_speed: u16,
    // The reference DC link voltage in mV.
    pub ref_dc_link_voltage: f32,
    // The reference DC link current in mA.
    pub ref_dc_link_current: f32,
    // Pier fan.
    pub pier_fan: PierFan,
}

impl MockPierFan {
    /// Mock pier fan to simulate the pier fan behavior.
    ///
    /// # Arguments
    /// * `address` - The address of the pier fan.
    ///
    /// # Returns
    /// A new instance of `MockPierFan`.
    pub fn new(address: u8) -> Self {
        let mut pier_fan = PierFan::new(address);
        pier_fan.actual_speed = PLANT_PIER_FAN_ACTUAL_SPEED;
        pier_fan.dc_link_voltage = PLANT_PIER_FAN_DC_LINK_VOLTAGE;
        pier_fan.dc_link_current = PLANT_PIER_FAN_DC_LINK_CURRENT;
        pier_fan.module_temperature = PLANT_PIER_FAN_MODULE_TEMPERATURE;
        pier_fan.motor_temperature = PLANT_PIER_FAN_MOTOR_TEMPERATURE;
        pier_fan.electronics_temperature = PLANT_PIER_FAN_ELECTRONICS_TEMPERATURE;
        pier_fan.current_direction_of_rotation = PLANT_PIER_FAN_CURRENT_DIRECTION_OF_ROTATION;
        pier_fan.current_modulation_level = PLANT_PIER_FAN_CURRENT_MODULATION_LEVEL;
        pier_fan.current_set_value = PLANT_PIER_FAN_CURRENT_SET_VALUE;
        pier_fan.enable_input_status = PLANT_PIER_FAN_ENABLE_INPUT_STATUS;
        pier_fan.current_parameter_set = PLANT_PIER_FAN_CURRENT_PARAMETER_SET;
        pier_fan.current_controller_function = PLANT_PIER_FAN_CURRENT_CONTROLLER_FUNCTION;
        pier_fan.current_power = PLANT_PIER_FAN_CURRENT_POWER;

        Self {
            _crc: Crc::<u16>::new(&CRC_16_MODBUS),

            max_speed: PLANT_PIER_FAN_MAX_SPEED,
            ref_dc_link_voltage: PLANT_PIER_FAN_REF_DC_LINK_VOLTAGE,
            ref_dc_link_current: PLANT_PIER_FAN_REF_DC_LINK_CURRENT,

            pier_fan,
        }
    }

    /// Add the motor status of the pier fan.
    ///
    /// # Arguments
    /// * `status` - The motor status to add.
    pub fn add_motor_status(&mut self, status: MotorStatusPierFan) {
        self.pier_fan.motor_status |= status.bit_value();

        // General error. This is set for every error.
        self.pier_fan.motor_status |= MotorStatusPierFan::FanBad.bit_value();
    }

    /// Add the warning status of the pier fan.
    ///
    /// # Arguments
    /// * `warning` - The warning status to set.
    pub fn add_warning(&mut self, warning: WarningPierFan) {
        self.pier_fan.warning |= warning.bit_value()
    }

    /// Reset the error.
    pub fn reset(&mut self) {
        self.pier_fan.motor_status = 0;
        self.pier_fan.warning = 0;
    }

    /// Request the specified number of registers from the pier fan.
    ///
    /// # Arguments
    /// * `data_address` - The starting address of the data to request.
    /// * `num` - The number of registers to request.
    ///
    /// # Returns
    /// A vector containing the Modbus frame response.
    pub fn request(&self, data_address: u16, num: u16) -> Vec<u8> {
        // Each register consists of 2 bytes.
        let data_bytes = num * 2;

        let mut frame_response = vec![0; 5 + (data_bytes as usize)];
        frame_response[0] = self.pier_fan.address;
        frame_response[1] = CODE_READ_HOLDING_REGISTERS;
        frame_response[2] = data_bytes as u8;

        let frame_data = match data_address {
            REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED => self.get_frame_max_speed(num),
            REGISTER_ADDRESS_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE => {
                self.get_frame_ref_dc_link_voltage(num)
            }
            REGISTER_ADDRESS_PIER_FAN_ACTUAL_SPEED => self.get_frame_actual_speed(num),
            _ => Vec::new(),
        };

        if !frame_data.is_empty() {
            frame_response[3..3 + (data_bytes as usize)].copy_from_slice(&frame_data);
        }

        calculate_modbus_crc_and_update_frame(&self._crc, &mut frame_response);

        frame_response
    }

    /// Get the frame for the maximum speed register.
    ///
    /// # Arguments
    /// * `num` - The number of registers to include in the frame.
    ///
    /// # Returns
    /// A vector containing the frame data for the maximum speed register.
    fn get_frame_max_speed(&self, num: u16) -> Vec<u8> {
        let data_bytes = num * 2;

        let mut frame = vec![0; data_bytes as usize];
        if num >= 1 {
            frame[0..2].copy_from_slice(&self.max_speed.to_be_bytes());
        }

        frame
    }

    /// Get the frame for the reference value of the DC link voltage register.
    ///
    /// # Arguments
    /// * `num` - The number of registers to include in the frame.
    ///
    /// # Returns
    /// A vector containing the frame data for the reference value of the DC
    /// link voltage register.
    fn get_frame_ref_dc_link_voltage(&self, num: u16) -> Vec<u8> {
        let data_bytes = num * 2;

        let mut frame = vec![0; data_bytes as usize];
        if num >= 1 {
            let ref_dc_link_voltage =
                (self.ref_dc_link_voltage / FACTOR_PIER_FAN_DC_LINK_REFERENCE_VOLTAGE) as u16;
            frame[0..2].copy_from_slice(&ref_dc_link_voltage.to_be_bytes());
        }

        if num >= 2 {
            let ref_dc_link_current =
                (self.ref_dc_link_current / FACTOR_PIER_FAN_DC_LINK_REFERENCE_CURRENT) as u16;
            frame[2..4].copy_from_slice(&ref_dc_link_current.to_be_bytes());
        }

        frame
    }

    /// Get the frame for the actual speed register.
    ///
    /// # Arguments
    /// * `num` - The number of registers to include in the frame.
    ///
    /// # Returns
    /// A vector containing the frame data for the actual speed register.
    fn get_frame_actual_speed(&self, num: u16) -> Vec<u8> {
        let data_bytes = num * 2;

        // The following encoding is the reverse of PierFan.from_frame().
        let pier_fan = &self.pier_fan;
        let mut frame = vec![0; data_bytes as usize];
        if num >= 1 {
            let actual_speed = (pier_fan.actual_speed * (MAX_VALUE_PIER_FAN_ACTUAL_SPEED as f32)
                / (self.max_speed as f32)) as u16;
            frame[0..2].copy_from_slice(&actual_speed.to_be_bytes());
        }

        if num >= 2 {
            frame[2..4].copy_from_slice(&pier_fan.motor_status.to_be_bytes());
        }

        if num >= 3 {
            frame[4..6].copy_from_slice(&pier_fan.warning.to_be_bytes());
        }

        if num >= 4 {
            // Change the unit from V to mV
            let dc_link_voltage = (pier_fan.dc_link_voltage * 1000.0 / self.ref_dc_link_voltage
                * (MAX_VALUE_PIER_FAN_DC_LINK_VOLTAGE_CURRENT as f32))
                as u16;
            frame[6..8].copy_from_slice(&dc_link_voltage.to_be_bytes());
        }

        if num >= 5 {
            // Change the unit from A to mA
            let dc_link_current = (pier_fan.dc_link_current * 1000.0 / self.ref_dc_link_current
                * (MAX_VALUE_PIER_FAN_DC_LINK_VOLTAGE_CURRENT as f32))
                as u16;
            frame[8..10].copy_from_slice(&dc_link_current.to_be_bytes());
        }

        if num >= 6 {
            frame[10..12].copy_from_slice(&pier_fan.module_temperature.to_be_bytes());
        }

        if num >= 7 {
            let motor_temperature = pier_fan.motor_temperature as u16;
            frame[12..14].copy_from_slice(&motor_temperature.to_be_bytes());
        }

        if num >= 8 {
            frame[14..16].copy_from_slice(&pier_fan.electronics_temperature.to_be_bytes());
        }

        if num >= 9 {
            frame[16..18].copy_from_slice(&pier_fan.current_direction_of_rotation.to_be_bytes());
        }

        if num >= 10 {
            let current_modulation_level =
                (pier_fan.current_modulation_level / 100.0 * (u16::MAX as f32)) as u16;
            frame[18..20].copy_from_slice(&current_modulation_level.to_be_bytes());
        }

        if num >= 11 {
            let current_set_value = (pier_fan.current_set_value
                * (MAX_VALUE_PIER_FAN_ACTUAL_SPEED as f32)
                / (self.max_speed as f32)) as u16;
            frame[20..22].copy_from_slice(&current_set_value.to_be_bytes());
        }

        if num >= 13 {
            frame[24..26].copy_from_slice(&pier_fan.enable_input_status.to_be_bytes());
        }

        if num >= 14 {
            frame[26..28].copy_from_slice(&pier_fan.current_parameter_set.to_be_bytes());
        }

        if num >= 15 {
            frame[28..30].copy_from_slice(&pier_fan.current_controller_function.to_be_bytes());
        }

        if num >= 18 {
            // Change the unit from W to uW because the units of reference
            // voltage and current are mV and mA, respectively.
            let current_power = (pier_fan.current_power * 1_000_000.0
                / self.ref_dc_link_voltage
                / self.ref_dc_link_current
                * (u16::MAX as f32)) as u16;
            frame[34..36].copy_from_slice(&current_power.to_be_bytes());
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    const EPSILON: f32 = 1e-1;

    use crate::constants::{
        NUM_REGISTER_PIER_FAN_ACTUAL_SPEED,
        NUM_REGISTER_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE,
    };

    #[test]
    fn test_add_motor_status() {
        let mut mock_pier_fan = MockPierFan::new(12);

        mock_pier_fan.add_motor_status(MotorStatusPierFan::FanBlocked);

        assert_eq!(
            mock_pier_fan.pier_fan.motor_status,
            MotorStatusPierFan::FanBad.bit_value() + MotorStatusPierFan::FanBlocked.bit_value()
        );

        mock_pier_fan.add_motor_status(MotorStatusPierFan::HallFailure);

        let expected_motor_status = MotorStatusPierFan::FanBad.bit_value()
            + MotorStatusPierFan::FanBlocked.bit_value()
            + MotorStatusPierFan::HallFailure.bit_value();
        assert_eq!(mock_pier_fan.pier_fan.motor_status, expected_motor_status);

        // Repeated motor status should not change the overall motor status.
        mock_pier_fan.add_motor_status(MotorStatusPierFan::HallFailure);

        assert_eq!(mock_pier_fan.pier_fan.motor_status, expected_motor_status);
    }

    #[test]
    fn test_add_warning() {
        let mut mock_pier_fan = MockPierFan::new(12);

        mock_pier_fan.add_warning(WarningPierFan::DcLinkVoltageLow);

        assert_eq!(
            mock_pier_fan.pier_fan.warning,
            WarningPierFan::DcLinkVoltageLow.bit_value()
        );

        mock_pier_fan.add_warning(WarningPierFan::ElectronicsTemperatureHigh);

        let expected_warning = WarningPierFan::DcLinkVoltageLow.bit_value()
            + WarningPierFan::ElectronicsTemperatureHigh.bit_value();
        assert_eq!(mock_pier_fan.pier_fan.warning, expected_warning);

        // Repeated warning should not change the overall warning.
        mock_pier_fan.add_warning(WarningPierFan::ElectronicsTemperatureHigh);

        assert_eq!(mock_pier_fan.pier_fan.warning, expected_warning);
    }

    #[test]
    fn test_reset() {
        let mut mock_pier_fan = MockPierFan::new(12);
        mock_pier_fan.pier_fan.motor_status = 10;
        mock_pier_fan.pier_fan.warning = 10;

        mock_pier_fan.reset();

        assert_eq!(mock_pier_fan.pier_fan.motor_status, 0);
        assert_eq!(mock_pier_fan.pier_fan.warning, 0);
    }

    #[test]
    fn test_request_default_register() {
        let mock_pier_fan = MockPierFan::new(12);

        // No number of registers requested
        let response_0 = mock_pier_fan.request(0, 0);

        assert_eq!(response_0.len(), 5);
        assert_eq!(response_0[0], 12);
        assert_eq!(response_0[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response_0[2], 0);

        // One number of registers requested
        let response_1 = mock_pier_fan.request(0, 1);

        assert_eq!(response_1.len(), 7);
        assert_eq!(response_1[0], 12);
        assert_eq!(response_1[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response_1[2], 2);
    }

    #[test]
    fn test_request_max_speed() {
        let mock_pier_fan = MockPierFan::new(12);

        let response = mock_pier_fan.request(REGISTER_ADDRESS_PIER_FAN_MAXIMUM_SPEED, 1);

        assert_eq!(response.len(), 7);
        assert_eq!(response[0], 12);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 2);
        assert_eq!(
            u16::from_be_bytes([response[3], response[4]]),
            PLANT_PIER_FAN_MAX_SPEED
        );
    }

    #[test]
    fn test_request_ref_dc_link_voltage() {
        let mock_pier_fan = MockPierFan::new(12);

        let response = mock_pier_fan.request(
            REGISTER_ADDRESS_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE,
            NUM_REGISTER_PIER_FAN_REFERENCE_VALUE_OF_DC_LINK_VOLTAGE,
        );

        assert_eq!(response.len(), 9);
        assert_eq!(response[0], 12);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 4);
        assert_eq!(
            (u16::from_be_bytes([response[3], response[4]]) as f32)
                * FACTOR_PIER_FAN_DC_LINK_REFERENCE_VOLTAGE,
            PLANT_PIER_FAN_REF_DC_LINK_VOLTAGE,
        );
        assert_eq!(
            (u16::from_be_bytes([response[5], response[6]]) as f32)
                * FACTOR_PIER_FAN_DC_LINK_REFERENCE_CURRENT,
            PLANT_PIER_FAN_REF_DC_LINK_CURRENT,
        );
    }

    #[test]
    fn test_request_actual_speed() {
        let mut mock_pier_fan = MockPierFan::new(12);
        mock_pier_fan.pier_fan.motor_status = 10;
        mock_pier_fan.pier_fan.warning = 12;
        mock_pier_fan.pier_fan.motor_temperature = -15;

        let response = mock_pier_fan.request(
            REGISTER_ADDRESS_PIER_FAN_ACTUAL_SPEED,
            NUM_REGISTER_PIER_FAN_ACTUAL_SPEED,
        );

        assert_eq!(response.len(), 41);
        assert_eq!(response[0], 12);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 36);

        // Need to change the units of reference DC link voltage and current
        // from millivolts and milliamps to volts and amps
        let pier_fan = PierFan::from_frame(
            mock_pier_fan.max_speed as f32,
            mock_pier_fan.ref_dc_link_voltage * 0.001,
            mock_pier_fan.ref_dc_link_current * 0.001,
            &response,
        )
        .unwrap();

        assert_relative_eq!(
            pier_fan.actual_speed,
            PLANT_PIER_FAN_ACTUAL_SPEED,
            epsilon = EPSILON
        );
        assert_eq!(pier_fan.motor_status, mock_pier_fan.pier_fan.motor_status);
        assert_eq!(pier_fan.warning, mock_pier_fan.pier_fan.warning);
        assert_relative_eq!(
            pier_fan.dc_link_voltage,
            PLANT_PIER_FAN_DC_LINK_VOLTAGE,
            epsilon = EPSILON
        );
        assert_relative_eq!(
            pier_fan.dc_link_current,
            PLANT_PIER_FAN_DC_LINK_CURRENT,
            epsilon = EPSILON
        );
        assert_eq!(
            pier_fan.module_temperature,
            PLANT_PIER_FAN_MODULE_TEMPERATURE
        );
        assert_eq!(
            pier_fan.motor_temperature,
            mock_pier_fan.pier_fan.motor_temperature
        );
        assert_eq!(
            pier_fan.electronics_temperature,
            PLANT_PIER_FAN_ELECTRONICS_TEMPERATURE
        );
        assert_eq!(
            pier_fan.current_direction_of_rotation,
            PLANT_PIER_FAN_CURRENT_DIRECTION_OF_ROTATION
        );
        assert_relative_eq!(
            pier_fan.current_modulation_level,
            PLANT_PIER_FAN_CURRENT_MODULATION_LEVEL,
            epsilon = EPSILON
        );
        assert_eq!(pier_fan.current_set_value, PLANT_PIER_FAN_CURRENT_SET_VALUE);
        assert_eq!(
            pier_fan.enable_input_status,
            PLANT_PIER_FAN_ENABLE_INPUT_STATUS
        );
        assert_eq!(
            pier_fan.current_parameter_set,
            PLANT_PIER_FAN_CURRENT_PARAMETER_SET
        );
        assert_eq!(
            pier_fan.current_controller_function,
            PLANT_PIER_FAN_CURRENT_CONTROLLER_FUNCTION
        );
        assert_relative_eq!(
            pier_fan.current_power,
            PLANT_PIER_FAN_CURRENT_POWER,
            epsilon = EPSILON
        );
    }
}

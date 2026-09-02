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

use crate::constants::{
    CODE_READ_HOLDING_REGISTERS, HUNDRED, REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION,
    REGISTER_ADDRESS_RECIRCULATION_PUMP_CONTROL, REGISTER_ADDRESS_RECIRCULATION_PUMP_DATA,
    REGISTER_ADDRESS_RECIRCULATION_PUMP_STATUS, TEN,
};
use crate::daq::recirculation_pump::{
    RecirculationPump, RecirculationPumpCimConfiguration, RecirculationPumpControl,
    RecirculationPumpData, RecirculationPumpStatus,
};
use crate::enums::AlarmWarningRecirculationPump;
use crate::mock::mock_constants::{
    PLANT_PUMP_CONFIGURATION_AUTO_ACK_CONTROL_BITS, PLANT_PUMP_CONFIGURATION_GENIBUS_TX_RX_COUNT,
    PLANT_PUMP_CONFIGURATION_PRODUCT_SOFTWARE_DATE,
    PLANT_PUMP_CONFIGURATION_PRODUCT_SOFTWARE_VERSION,
    PLANT_PUMP_CONFIGURATION_SOFTWARE_DEFINED_BIT_RATE,
    PLANT_PUMP_CONFIGURATION_SOFTWARE_DEFINED_PARITY,
    PLANT_PUMP_CONFIGURATION_SOFTWARE_DEFINED_STOP_BIT, PLANT_PUMP_CONTROL_CONTROL_MODE,
    PLANT_PUMP_CONTROL_OPERATION_MODE, PLANT_PUMP_CONTROL_SETPOINT, PLANT_PUMP_DATA_ENERGY,
    PLANT_PUMP_DATA_MOTOR_CURRENT, PLANT_PUMP_DATA_OPERATION_TIME, PLANT_PUMP_DATA_POWER,
    PLANT_PUMP_DATA_RELATIVE_PERFORMANCE, PLANT_PUMP_STATUS_PUMPS_PRESENT,
    PLANT_PUMP_STATUS_SYSTEM_ACTIVE_FUNCTIONS,
};
use crate::utility::calculate_modbus_crc_and_update_frame;

pub struct MockRecirculationPump {
    // Cyclic redundancy check (CRC) calculator for the Modbus communication.
    _crc: Crc<u16>,
    // Recirculation pump.
    pub recirculation_pump: RecirculationPump,
}

impl MockRecirculationPump {
    /// Mock recirculation pump to simulate the recirculation pump behavior.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    ///
    /// # Returns
    /// A new instance of `MockRecirculationPump`.
    pub fn new(address: u8) -> Self {
        Self {
            _crc: Crc::<u16>::new(&CRC_16_MODBUS),

            recirculation_pump: Self::create_recirculation_pump(address),
        }
    }

    /// Create a new instance of the recirculation pump.
    ///
    /// # Arguments
    /// * `address` - The address of the recirculation pump.
    ///
    /// # Returns
    /// A new instance of `RecirculationPump`.
    fn create_recirculation_pump(address: u8) -> RecirculationPump {
        let mut recirculation_pump = RecirculationPump::new(address);

        recirculation_pump.cim_configuration = Some(RecirculationPumpCimConfiguration {
            slave_minimum_reply_delay: 0,
            software_defined_modbus_address: 0,
            software_defined_bit_rate: PLANT_PUMP_CONFIGURATION_SOFTWARE_DEFINED_BIT_RATE,
            auto_ack_control_bits: PLANT_PUMP_CONFIGURATION_AUTO_ACK_CONTROL_BITS,
            software_defined_parity: PLANT_PUMP_CONFIGURATION_SOFTWARE_DEFINED_PARITY,
            software_defined_stop_bit: PLANT_PUMP_CONFIGURATION_SOFTWARE_DEFINED_STOP_BIT,
            watchdog: 0,
            genibus_diode_off: 0,
            genibus_crc_error_cnt: 0,
            genibus_data_error_cnt: 0,
            version_number: 0,
            actual_modbus_address: address as u16,
            genibus_tx_count: PLANT_PUMP_CONFIGURATION_GENIBUS_TX_RX_COUNT,
            genibus_rx_count: PLANT_PUMP_CONFIGURATION_GENIBUS_TX_RX_COUNT,
            unit_family: 0,
            unit_type: 0,
            unit_version: 0,
            product_software_version: String::from(
                PLANT_PUMP_CONFIGURATION_PRODUCT_SOFTWARE_VERSION,
            ),
            product_software_date: String::from(PLANT_PUMP_CONFIGURATION_PRODUCT_SOFTWARE_DATE),
        });

        recirculation_pump.control = Some(RecirculationPumpControl {
            status: 0,
            control_mode: PLANT_PUMP_CONTROL_CONTROL_MODE,
            operation_mode: PLANT_PUMP_CONTROL_OPERATION_MODE,
            setpoint: PLANT_PUMP_CONTROL_SETPOINT,
            control_pump_1: 0,
        });

        recirculation_pump.status = Some(RecirculationPumpStatus {
            status: 0,
            process_feedback: PLANT_PUMP_CONTROL_SETPOINT,
            control_mode: PLANT_PUMP_CONTROL_CONTROL_MODE,
            operation_mode: PLANT_PUMP_CONTROL_OPERATION_MODE,
            alarm_code: AlarmWarningRecirculationPump::None,
            warning_code: AlarmWarningRecirculationPump::None,
            pumps_present: PLANT_PUMP_STATUS_PUMPS_PRESENT,
            pumps_running: PLANT_PUMP_STATUS_PUMPS_PRESENT,
            pumps_fault: 0,
            pumps_comm_fault: 0,
            system_active_functions: PLANT_PUMP_STATUS_SYSTEM_ACTIVE_FUNCTIONS,
        });

        recirculation_pump.data = Some(RecirculationPumpData {
            relative_performance: PLANT_PUMP_DATA_RELATIVE_PERFORMANCE,
            actual_setpoint: PLANT_PUMP_CONTROL_SETPOINT,
            motor_current: PLANT_PUMP_DATA_MOTOR_CURRENT,
            power: PLANT_PUMP_DATA_POWER,
            operation_time: PLANT_PUMP_DATA_OPERATION_TIME,
            energy: PLANT_PUMP_DATA_ENERGY,
            user_setpoint: PLANT_PUMP_CONTROL_SETPOINT,
        });

        recirculation_pump
    }

    /// Request the specified number of registers from the recirculation pump.
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
        frame_response[0] = self.recirculation_pump.address;
        frame_response[1] = CODE_READ_HOLDING_REGISTERS;
        frame_response[2] = data_bytes as u8;

        let frame_data = match data_address {
            REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION => {
                self.get_frame_cim_configuration(num)
            }
            REGISTER_ADDRESS_RECIRCULATION_PUMP_CONTROL => self.get_frame_control(num),
            REGISTER_ADDRESS_RECIRCULATION_PUMP_STATUS => self.get_frame_status(num),
            REGISTER_ADDRESS_RECIRCULATION_PUMP_DATA => self.get_frame_data(num),
            _ => Vec::new(),
        };

        if !frame_data.is_empty() {
            frame_response[3..3 + (data_bytes as usize)].copy_from_slice(&frame_data);
        }

        calculate_modbus_crc_and_update_frame(&self._crc, &mut frame_response);

        frame_response
    }

    /// Get the frame for the CIM configuration register.
    ///
    /// # Arguments
    /// * `num` - The number of registers to include in the frame.
    ///
    /// # Returns
    /// A vector containing the frame data for the CIM configuration register.
    fn get_frame_cim_configuration(&self, num: u16) -> Vec<u8> {
        let data_bytes = num * 2;

        // The following encoding is the reverse of
        // RecirculationPump.from_frame_cim_configuration().
        let mut frame = vec![0; data_bytes as usize];
        if let Some(configuration) = &self.recirculation_pump.cim_configuration {
            if num >= 1 {
                frame[0..2].copy_from_slice(&configuration.slave_minimum_reply_delay.to_be_bytes());
            }

            if num >= 3 {
                frame[4..6]
                    .copy_from_slice(&configuration.software_defined_modbus_address.to_be_bytes());
            }

            if num >= 4 {
                frame[6..8].copy_from_slice(&configuration.software_defined_bit_rate.to_be_bytes());
            }

            if num >= 5 {
                frame[8..10].copy_from_slice(&configuration.auto_ack_control_bits.to_be_bytes());
            }

            if num >= 9 {
                frame[16..18].copy_from_slice(&configuration.software_defined_parity.to_be_bytes());
            }

            if num >= 10 {
                frame[18..20]
                    .copy_from_slice(&configuration.software_defined_stop_bit.to_be_bytes());
            }

            if num >= 12 {
                frame[22..24].copy_from_slice(&configuration.watchdog.to_be_bytes());
            }

            if num >= 13 {
                frame[24..26].copy_from_slice(&configuration.genibus_diode_off.to_be_bytes());
            }

            if num >= 21 {
                frame[40..42].copy_from_slice(&configuration.genibus_crc_error_cnt.to_be_bytes());
            }

            if num >= 22 {
                frame[42..44].copy_from_slice(&configuration.genibus_data_error_cnt.to_be_bytes());
            }

            if num >= 23 {
                frame[44..46].copy_from_slice(&configuration.version_number.to_be_bytes());
            }

            if num >= 24 {
                frame[46..48].copy_from_slice(&configuration.actual_modbus_address.to_be_bytes());
            }

            if num >= 25 {
                let genibus_tx_count_high = (configuration.genibus_tx_count >> 16) as u16;
                frame[48..50].copy_from_slice(&genibus_tx_count_high.to_be_bytes());
            }

            if num >= 26 {
                let genibus_tx_count_low = (configuration.genibus_tx_count & 0xFFFF) as u16;
                frame[50..52].copy_from_slice(&genibus_tx_count_low.to_be_bytes());
            }

            if num >= 27 {
                let genibus_rx_count_high = (configuration.genibus_rx_count >> 16) as u16;
                frame[52..54].copy_from_slice(&genibus_rx_count_high.to_be_bytes());
            }

            if num >= 28 {
                let genibus_rx_count_low = (configuration.genibus_rx_count & 0xFFFF) as u16;
                frame[54..56].copy_from_slice(&genibus_rx_count_low.to_be_bytes());
            }

            if num >= 30 {
                frame[58..60].copy_from_slice(&configuration.unit_family.to_be_bytes());
            }

            if num >= 31 {
                frame[60..62].copy_from_slice(&configuration.unit_type.to_be_bytes());
            }

            if num >= 32 {
                frame[62..64].copy_from_slice(&configuration.unit_version.to_be_bytes());
            }

            if num >= 34 {
                let digits = Self::get_digits_from_string(&configuration.product_software_version);
                frame[66] = Self::combine_digits(digits[0], digits[1]);
                frame[67] = Self::combine_digits(digits[2], digits[3]);
            }

            if num >= 35 {
                let digits = Self::get_digits_from_string(&configuration.product_software_version);
                frame[68] = Self::combine_digits(digits[4], digits[5]);
                frame[69] = Self::combine_digits(digits[6], digits[7]);
            }

            if num >= 36 {
                let digits = Self::get_digits_from_string(&configuration.product_software_date);
                frame[70] = Self::combine_digits(digits[0], digits[1]);
                frame[71] = Self::combine_digits(digits[2], digits[3]);
            }

            if num >= 37 {
                let digits = Self::get_digits_from_string(&configuration.product_software_date);
                frame[72] = Self::combine_digits(digits[4], digits[5]);
                frame[73] = Self::combine_digits(digits[6], digits[7]);
            }
        }

        frame
    }

    /// Get the digits from a string as a vector of u8.
    ///
    /// # Arguments
    /// * `s` - The input string containing digits.
    ///
    /// # Returns
    /// A vector of u8 containing the digits extracted from the string.
    fn get_digits_from_string(s: &str) -> Vec<u8> {
        s.chars()
            .filter_map(|c| c.to_digit(10).map(|d| d as u8))
            .collect()
    }

    /// Combine two digits into a single u8 value.
    ///
    /// # Arguments
    /// * `high` - The high nibble (4 bits) of the resulting byte.
    /// * `low` - The low nibble (4 bits) of the resulting byte.
    ///
    /// # Returns
    /// A u8 value with the high and low nibbles combined.
    fn combine_digits(high: u8, low: u8) -> u8 {
        (high << 4) | low
    }

    /// Get the frame for the control register.
    ///
    /// # Arguments
    /// * `num` - The number of registers to include in the frame.
    ///
    /// # Returns
    /// A vector containing the frame data for the control register.
    fn get_frame_control(&self, num: u16) -> Vec<u8> {
        let data_bytes = num * 2;

        // The following encoding is the reverse of
        // RecirculationPump.from_frame_control().
        let mut frame = vec![0; data_bytes as usize];
        if let Some(control) = &self.recirculation_pump.control {
            if num >= 1 {
                frame[0..2].copy_from_slice(&control.status.to_be_bytes());
            }

            if num >= 2 {
                let control_mode = control.control_mode as u16;
                frame[2..4].copy_from_slice(&control_mode.to_be_bytes());
            }

            if num >= 3 {
                let operation_mode = control.operation_mode as u16;
                frame[4..6].copy_from_slice(&operation_mode.to_be_bytes());
            }

            if num >= 4 {
                let setpoint = (control.setpoint * HUNDRED) as u16;
                frame[6..8].copy_from_slice(&setpoint.to_be_bytes());
            }

            if num >= 5 {
                frame[8..10].copy_from_slice(&control.control_pump_1.to_be_bytes());
            }
        }

        frame
    }

    /// Get the frame for the status register.
    ///
    /// # Arguments
    /// * `num` - The number of registers to include in the frame.
    ///
    /// # Returns
    /// A vector containing the frame data for the status register.
    fn get_frame_status(&self, num: u16) -> Vec<u8> {
        let data_bytes = num * 2;

        // The following encoding is the reverse of
        // RecirculationPump.from_frame_status().
        let mut frame = vec![0; data_bytes as usize];
        if let Some(status) = &self.recirculation_pump.status {
            if num >= 1 {
                frame[0..2].copy_from_slice(&status.status.to_be_bytes());
            };

            if num >= 2 {
                let process_feedback = (status.process_feedback * HUNDRED) as u16;
                frame[2..4].copy_from_slice(&process_feedback.to_be_bytes());
            };

            if num >= 3 {
                let control_mode = status.control_mode as u16;
                frame[4..6].copy_from_slice(&control_mode.to_be_bytes());
            };

            if num >= 4 {
                let operation_mode = status.operation_mode as u16;
                frame[6..8].copy_from_slice(&operation_mode.to_be_bytes());
            };

            if num >= 5 {
                let alarm_code = status.alarm_code as u16;
                frame[8..10].copy_from_slice(&alarm_code.to_be_bytes());
            };

            if num >= 6 {
                let warning_code = status.warning_code as u16;
                frame[10..12].copy_from_slice(&warning_code.to_be_bytes());
            };

            if num >= 8 {
                let pumps_present = status.pumps_present as u16;
                frame[14..16].copy_from_slice(&pumps_present.to_be_bytes());
            };

            if num >= 9 {
                let pumps_running = status.pumps_running as u16;
                frame[16..18].copy_from_slice(&pumps_running.to_be_bytes());
            };

            if num >= 10 {
                let pumps_fault = status.pumps_fault as u16;
                frame[18..20].copy_from_slice(&pumps_fault.to_be_bytes());
            };

            if num >= 11 {
                let pumps_comm_fault = status.pumps_comm_fault as u16;
                frame[20..22].copy_from_slice(&pumps_comm_fault.to_be_bytes());
            };

            if num >= 12 {
                frame[22..24].copy_from_slice(&status.system_active_functions.to_be_bytes());
            };
        }

        frame
    }

    /// Get the frame for the data register.
    ///
    /// # Arguments
    /// * `num` - The number of registers to include in the frame.
    ///
    /// # Returns
    /// A vector containing the frame data for the data register.
    fn get_frame_data(&self, num: u16) -> Vec<u8> {
        let data_bytes = num * 2;

        // The following encoding is the reverse of
        // RecirculationPump.from_frame_data().
        let mut frame = vec![0; data_bytes as usize];
        if let Some(data) = &self.recirculation_pump.data {
            if num >= 3 {
                let relative_performance = (data.relative_performance * HUNDRED) as u16;
                frame[4..6].copy_from_slice(&relative_performance.to_be_bytes());
            }

            if num >= 8 {
                let actual_setpoint = (data.actual_setpoint * HUNDRED) as u16;
                frame[14..16].copy_from_slice(&actual_setpoint.to_be_bytes());
            }

            if num >= 9 {
                let motor_current = (data.motor_current * TEN) as u16;
                frame[16..18].copy_from_slice(&motor_current.to_be_bytes());
            }

            if num >= 12 {
                let power_high = (data.power >> 16) as u16;
                frame[22..24].copy_from_slice(&power_high.to_be_bytes());
            }

            if num >= 13 {
                let power_low = (data.power & 0xFFFF) as u16;
                frame[24..26].copy_from_slice(&power_low.to_be_bytes());
            }

            if num >= 27 {
                let operation_time_high = (data.operation_time >> 16) as u16;
                frame[52..54].copy_from_slice(&operation_time_high.to_be_bytes());
            }

            if num >= 28 {
                let operation_time_low = (data.operation_time & 0xFFFF) as u16;
                frame[54..56].copy_from_slice(&operation_time_low.to_be_bytes());
            }

            if num >= 32 {
                let energy_high = (data.energy >> 16) as u16;
                frame[62..64].copy_from_slice(&energy_high.to_be_bytes());
            }

            if num >= 33 {
                let energy_low = (data.energy & 0xFFFF) as u16;
                frame[64..66].copy_from_slice(&energy_low.to_be_bytes());
            }

            if num >= 43 {
                let user_setpoint = (data.user_setpoint * HUNDRED) as u16;
                frame[84..86].copy_from_slice(&user_setpoint.to_be_bytes());
            }
        }

        frame
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::{
        NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION, NUM_REGISTER_RECIRCULATION_PUMP_CONTROL,
        NUM_REGISTER_RECIRCULATION_PUMP_DATA, NUM_REGISTER_RECIRCULATION_PUMP_STATUS,
    };
    use crate::enums::{ControlModeRecirculationPump, OperationModeRecirculationPump};

    #[test]
    fn test_new() {
        let mock_pump = MockRecirculationPump::new(1);

        assert_eq!(mock_pump.recirculation_pump.address, 1);
        assert!(!mock_pump.recirculation_pump.cim_configuration.is_none());
        assert!(!mock_pump.recirculation_pump.control.is_none());
        assert!(!mock_pump.recirculation_pump.status.is_none());
        assert!(!mock_pump.recirculation_pump.data.is_none());
    }

    #[test]
    fn test_get_digits_from_string() {
        let digits = MockRecirculationPump::get_digits_from_string("02.30.06.78");

        assert_eq!(digits, vec![0, 2, 3, 0, 0, 6, 7, 8]);
    }

    #[test]
    fn test_combine_digits() {
        let combined = MockRecirculationPump::combine_digits(2, 3);

        assert_eq!(combined, 0x23);
    }

    #[test]
    fn test_request_default_register() {
        let mock_pump = MockRecirculationPump::new(1);

        // No number of registers requested
        let response_0 =
            mock_pump.request(REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION, 0);

        assert_eq!(response_0.len(), 5);
        assert_eq!(response_0[0], 1);
        assert_eq!(response_0[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response_0[2], 0);

        // One number of registers requested
        let response_1 =
            mock_pump.request(REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION, 1);

        assert_eq!(response_1.len(), 7);
        assert_eq!(response_1[0], 1);
        assert_eq!(response_1[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response_1[2], 2);
    }

    #[test]
    fn test_get_frame_cim_configuration() {
        let mut mock_pump = MockRecirculationPump::new(1);
        mock_pump.recirculation_pump.cim_configuration = Some(RecirculationPumpCimConfiguration {
            slave_minimum_reply_delay: 9123,
            software_defined_modbus_address: 3,
            software_defined_bit_rate: 4,
            auto_ack_control_bits: 1,
            software_defined_parity: 2,
            software_defined_stop_bit: 1,
            watchdog: 1,
            genibus_diode_off: 1,
            genibus_crc_error_cnt: 131,
            genibus_data_error_cnt: 2145,
            version_number: 321,
            actual_modbus_address: mock_pump.recirculation_pump.address as u16,
            genibus_tx_count: 812,
            genibus_rx_count: 793,
            unit_family: 1,
            unit_type: 2,
            unit_version: 31,
            product_software_version: String::from("12.34.56.78"),
            product_software_date: String::from("13/02/2013"),
        });

        let response = mock_pump.request(
            REGISTER_ADDRESS_RECIRCULATION_PUMP_CIM_CONFIGURATION,
            NUM_REGISTER_RECIRCULATION_PUMP_CIM_CONFIGURATION,
        );

        assert_eq!(response.len(), 79);
        assert_eq!(response[0], 1);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 74);

        let pump = RecirculationPump::from_frame(&response, &[], &[], &[]);
        let cim_configuration = pump.cim_configuration.unwrap();

        assert_eq!(
            cim_configuration,
            mock_pump.recirculation_pump.cim_configuration.unwrap()
        );
        assert_eq!(pump.address, mock_pump.recirculation_pump.address);
    }

    #[test]
    fn test_get_frame_control() {
        let mut mock_pump = MockRecirculationPump::new(1);
        mock_pump.recirculation_pump.control = Some(RecirculationPumpControl {
            status: 4,
            control_mode: ControlModeRecirculationPump::AutoAdapt,
            operation_mode: OperationModeRecirculationPump::OpenLoopMax,
            setpoint: 98.82,
            control_pump_1: 2,
        });

        let response = mock_pump.request(
            REGISTER_ADDRESS_RECIRCULATION_PUMP_CONTROL,
            NUM_REGISTER_RECIRCULATION_PUMP_CONTROL,
        );

        assert_eq!(response.len(), 15);
        assert_eq!(response[0], 1);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 10);

        let pump = RecirculationPump::from_frame(&[], &response, &[], &[]);
        let control = pump.control.unwrap();

        assert_eq!(control, mock_pump.recirculation_pump.control.unwrap());
    }

    #[test]
    fn test_get_frame_status() {
        let mut mock_pump = MockRecirculationPump::new(1);
        mock_pump.recirculation_pump.status = Some(RecirculationPumpStatus {
            status: 1234,
            process_feedback: 97.78,
            control_mode: ControlModeRecirculationPump::AutoAdapt,
            operation_mode: OperationModeRecirculationPump::OpenLoopMin,
            alarm_code: AlarmWarningRecirculationPump::TemperatureControlElectronics,
            warning_code: AlarmWarningRecirculationPump::VentValveDefective,
            pumps_present: 255,
            pumps_running: 127,
            pumps_fault: 128,
            pumps_comm_fault: 128,
            system_active_functions: 16382,
        });

        let response = mock_pump.request(
            REGISTER_ADDRESS_RECIRCULATION_PUMP_STATUS,
            NUM_REGISTER_RECIRCULATION_PUMP_STATUS,
        );

        assert_eq!(response.len(), 29);
        assert_eq!(response[0], 1);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 24);

        let pump = RecirculationPump::from_frame(&[], &[], &response, &[]);
        let status = pump.status.unwrap();

        assert_eq!(status, mock_pump.recirculation_pump.status.unwrap());
    }

    #[test]
    fn test_get_frame_data() {
        let mut mock_pump = MockRecirculationPump::new(1);
        mock_pump.recirculation_pump.data = Some(RecirculationPumpData {
            relative_performance: 93.12,
            actual_setpoint: 97.81,
            motor_current: 3.4,
            power: 34512,
            operation_time: 2316,
            energy: 3,
            user_setpoint: 98.82,
        });

        let response = mock_pump.request(
            REGISTER_ADDRESS_RECIRCULATION_PUMP_DATA,
            NUM_REGISTER_RECIRCULATION_PUMP_DATA,
        );

        assert_eq!(response.len(), 91);
        assert_eq!(response[0], 1);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 86);

        let pump = RecirculationPump::from_frame(&[], &[], &[], &response);
        let data = pump.data.unwrap();

        assert_eq!(data, mock_pump.recirculation_pump.data.unwrap());
    }
}

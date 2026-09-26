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

use crate::constants::CODE_READ_HOLDING_REGISTERS;
use crate::daq::chiller::Chiller;
use crate::enums::HeartbeatChiller;
use crate::mock::mock_constants::PLANT_TEMPERATURE;
use crate::utility::{calculate_modbus_crc_and_update_frame, celsius_to_fahrenheit};

pub struct MockChiller {
    // Cyclic redundancy check (CRC) calculator for the Modbus communication.
    _crc: Crc<u16>,
    // Chiller.
    pub chiller: Chiller,
    // Heartbeat counter to simulate the heartbeat signal of the chiller.
    _counter_heartbeat: u8,
}

impl MockChiller {
    /// Mock chiller to simulate the chiller behavior.
    ///
    /// # Arguments
    /// * `address` - The address of the chiller.
    ///
    /// # Returns
    /// A new instance of `MockChiller`.
    pub fn new(address: u8) -> Self {
        let mut chiller = Chiller::new(address);

        let temperature = celsius_to_fahrenheit(PLANT_TEMPERATURE);
        chiller.temperature_setpoint = temperature as u16;
        chiller.to_process_temperature = temperature as u16;
        chiller.from_process_temperature = temperature as u16;
        chiller.zone_1_temperature_evaporator_in = temperature as u8;
        chiller.zone_1_temperature_evaporator_out = temperature as u8;
        chiller.zone_2_temperature_evaporator_in = temperature as u8;
        chiller.zone_2_temperature_evaporator_out = temperature as u8;

        Self {
            _crc: Crc::<u16>::new(&CRC_16_MODBUS),

            chiller,

            _counter_heartbeat: 0,
        }
    }

    /// Set the temperature of the chiller.
    ///
    /// # Arguments
    /// * `temperature` - The temperature to set for the chiller. The unit is
    ///   the degree Fahrenheit.
    pub fn set_temperature(&mut self, temperature: u16) {
        self.chiller.temperature_setpoint = temperature;
    }

    /// Request the specified number of registers from the chiller.
    ///
    /// # Arguments
    /// * `num` - The number of registers to request.
    ///
    /// # Returns
    /// A vector containing the Modbus frame response.
    pub fn request(&self, num: u16) -> Vec<u8> {
        // Each register consists of 2 bytes.
        let data_bytes = num * 2;

        let mut frame_response = vec![0; 5 + (data_bytes as usize)];
        frame_response[0] = self.chiller.address;
        frame_response[1] = CODE_READ_HOLDING_REGISTERS;
        frame_response[2] = data_bytes as u8;

        // For the indices, see the Chiller.from_frame().
        if num >= 1 {
            frame_response[3..5].copy_from_slice(&self.chiller.temperature_setpoint.to_be_bytes());
        }

        if num >= 3 {
            frame_response[7..9]
                .copy_from_slice(&self.chiller.temperature_high_alarm.to_be_bytes());
        }

        if num >= 4 {
            frame_response[9..11]
                .copy_from_slice(&self.chiller.temperature_low_alarm.to_be_bytes());
        }

        if num >= 6 {
            frame_response[13..15]
                .copy_from_slice(&self.chiller.to_process_temperature.to_be_bytes());
        }

        if num >= 7 {
            frame_response[15..17]
                .copy_from_slice(&self.chiller.from_process_temperature.to_be_bytes());
        }

        if num >= 9 {
            frame_response[19..21].copy_from_slice(&self.chiller.process_status.to_be_bytes());
        }

        if num >= 10 {
            frame_response[21..23].copy_from_slice(&self.chiller.machine_status_1.to_be_bytes());
        }

        if num >= 11 {
            frame_response[23..25].copy_from_slice(&self.chiller.machine_status_2.to_be_bytes());
        }

        if num >= 12 {
            frame_response[25..27].copy_from_slice(&self.chiller.machine_status_3.to_be_bytes());
        }

        if num >= 15 {
            frame_response[31..33].copy_from_slice(&self.chiller.heartbeat.to_be_bytes());
        }

        if num >= 17 {
            frame_response[35] = self.chiller.zone_1_temperature_evaporator_in;
            frame_response[36] = self.chiller.zone_1_temperature_evaporator_out;
        }

        if num >= 18 {
            frame_response[37] = self.chiller.zone_2_temperature_evaporator_in;
            frame_response[38] = self.chiller.zone_2_temperature_evaporator_out;
        }

        if num >= 23 {
            frame_response[47..49].copy_from_slice(&self.chiller.zone_1_status.to_be_bytes());
        }

        if num >= 24 {
            frame_response[49..51].copy_from_slice(&self.chiller.zone_2_status.to_be_bytes());
        }

        calculate_modbus_crc_and_update_frame(&self._crc, &mut frame_response);

        frame_response
    }

    /// Trigger the heartbeat by toggling the corresponding bits in the
    /// chiller's heartbeat.
    ///
    /// # Notes
    /// Call this function every second to properly trigger the heartbeat.
    pub fn trigger_heartbeat(&mut self) {
        self._counter_heartbeat += 1;

        if self._counter_heartbeat >= 16 {
            self._counter_heartbeat = 0;

            self.chiller.heartbeat ^= HeartbeatChiller::SixteenSeconds.bit_value();
        }

        // Reverse the first bit of the self.chiller.heartbeat.
        self.chiller.heartbeat ^= HeartbeatChiller::OneSecond.bit_value();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::NUM_REGISTER_CHILLER;

    #[test]
    fn test_set_temperature() {
        let mut mock_chiller = MockChiller::new(3);

        mock_chiller.set_temperature(5);

        assert_eq!(mock_chiller.chiller.temperature_setpoint, 5);
    }

    #[test]
    fn test_request() {
        let mut mock_chiller = MockChiller::new(3);
        mock_chiller.chiller.temperature_high_alarm = 21;
        mock_chiller.chiller.temperature_low_alarm = 12;
        mock_chiller.chiller.to_process_temperature = 18;
        mock_chiller.chiller.from_process_temperature = 19;
        mock_chiller.chiller.process_status = 13;
        mock_chiller.chiller.machine_status_1 = 16;
        mock_chiller.chiller.machine_status_2 = 17;
        mock_chiller.chiller.machine_status_3 = 18;
        mock_chiller.chiller.heartbeat = 17;
        mock_chiller.chiller.zone_1_temperature_evaporator_in = 22;
        mock_chiller.chiller.zone_1_temperature_evaporator_out = 23;
        mock_chiller.chiller.zone_2_temperature_evaporator_in = 24;
        mock_chiller.chiller.zone_2_temperature_evaporator_out = 25;
        mock_chiller.chiller.zone_1_status = 14;
        mock_chiller.chiller.zone_2_status = 15;

        let response = mock_chiller.request(NUM_REGISTER_CHILLER);

        assert_eq!(response.len(), 53);
        assert_eq!(response[0], 3);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 48);

        let chiller = Chiller::from_frame(&response).unwrap();

        assert_eq!(chiller, mock_chiller.chiller);
    }

    #[test]
    fn test_trigger_heartbeat() {
        let mut mock_chiller = MockChiller::new(3);

        // First 31 heartbeat triggers
        for count in 0..31 {
            mock_chiller.trigger_heartbeat();

            if count % 2 == 0 {
                assert!(
                    mock_chiller.chiller.heartbeat & HeartbeatChiller::OneSecond.bit_value() != 0
                );
            } else {
                assert!(
                    mock_chiller.chiller.heartbeat & HeartbeatChiller::OneSecond.bit_value() == 0
                );
            }

            if count >= 15 {
                assert!(
                    mock_chiller.chiller.heartbeat & HeartbeatChiller::SixteenSeconds.bit_value()
                        != 0
                );
            } else {
                assert!(
                    mock_chiller.chiller.heartbeat & HeartbeatChiller::SixteenSeconds.bit_value()
                        == 0
                );
            }
        }

        // 32nd heartbeat trigger
        mock_chiller.trigger_heartbeat();

        assert!(mock_chiller.chiller.heartbeat & HeartbeatChiller::SixteenSeconds.bit_value() == 0);
    }
}

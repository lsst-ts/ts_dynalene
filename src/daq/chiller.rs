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

use crate::constants::NUM_REGISTER_CHILLER;
use crate::utility::get_values_from_u8_array;

#[derive(Debug, PartialEq)]
pub struct Chiller {
    // Address of the chiller.
    pub address: u8,
    // Temperature setpoint(0-250). The unit is degrees Fahrenheit.
    pub temperature_setpoint: u16,
    // Temperature high alarm (0-50, deviation value). The unit is degrees
    // Fahrenheit.
    pub temperature_high_alarm: u16,
    // Temperature low alarm (0-50, deviation value). The unit is degrees
    // Fahrenheit.
    pub temperature_low_alarm: u16,
    // To process temperature (0-255). The unit is degrees Fahrenheit.
    pub to_process_temperature: u16,
    // From process temperature (0-255). The unit is degrees Fahrenheit.
    pub from_process_temperature: u16,
    // Process status that each bit is defined in `ProcessStatusChiller` in
    // enums.rs.
    pub process_status: u16,
    // Machine status 1 that each bit is defined in `MachineStatus1Chiller` in
    // enums.rs.
    pub machine_status_1: u16,
    // Machine status 2 that each bit is defined in `MachineStatus2Chiller` in
    // enums.rs.
    pub machine_status_2: u16,
    // Machine status 3 that each bit is defined in `MachineStatus3Chiller` in
    // enums.rs.
    pub machine_status_3: u16,
    // Heartbeat that each bit is defined in `HeartbeatChiller` in enums.rs.
    pub heartbeat: u16,
    // Zone 1 temperature (evaporator inlet). The unit is degrees Fahrenheit.
    pub zone_1_temperature_evaporator_in: u8,
    // Zone 1 temperature (evaporator outlet). The unit is degrees Fahrenheit.
    pub zone_1_temperature_evaporator_out: u8,
    // Zone 2 temperature (evaporator inlet). The unit is degrees Fahrenheit.
    pub zone_2_temperature_evaporator_in: u8,
    // Zone 2 temperature (evaporator outlet). The unit is degrees Fahrenheit.
    pub zone_2_temperature_evaporator_out: u8,
    // Zone 1 status that each bit is defined in `ZoneStatusChiller` in
    // enums.rs.
    pub zone_1_status: u16,
    // Zone 2 status that each bit is defined in `ZoneStatusChiller` in
    // enums.rs.
    pub zone_2_status: u16,
}

impl Chiller {
    /// Chiller to have the measured temperatures and status values.
    ///
    /// # Arguments
    /// * `address` - The address of the chiller.
    ///
    /// # Returns
    /// A new instance of `Chiller`.
    pub fn new(address: u8) -> Self {
        Self {
            address,

            temperature_setpoint: 0,
            temperature_high_alarm: 0,
            temperature_low_alarm: 0,
            to_process_temperature: 0,
            from_process_temperature: 0,
            process_status: 0,
            machine_status_1: 0,
            machine_status_2: 0,
            machine_status_3: 0,
            heartbeat: 0,
            zone_1_temperature_evaporator_in: 0,
            zone_1_temperature_evaporator_out: 0,
            zone_2_temperature_evaporator_in: 0,
            zone_2_temperature_evaporator_out: 0,
            zone_1_status: 0,
            zone_2_status: 0,
        }
    }

    /// Create a `Chiller` instance from a Modbus frame.
    ///
    /// # Arguments
    /// * `frame` - The Modbus frame containing the chiller data.
    ///
    /// # Returns
    /// An `Option` containing the `Chiller` if the frame is valid, or
    /// `None` otherwise.
    pub fn from_frame(frame: &[u8]) -> Option<Chiller> {
        const DATA_BYTES_CHILLER: usize = 2 * (NUM_REGISTER_CHILLER as usize);
        const FRAME_LENGTH_CHILLER: usize = 5 + DATA_BYTES_CHILLER;
        if (frame.len() != FRAME_LENGTH_CHILLER) || (frame[2] != (DATA_BYTES_CHILLER as u8)) {
            return None;
        }

        let address = frame[0];

        let values = get_values_from_u8_array::<u16, { NUM_REGISTER_CHILLER as usize }>(
            &frame[3..(3 + DATA_BYTES_CHILLER)],
        )?;

        let (temperature_in_1, temperature_out_1) = Self::get_evaporator_temperatures(values[16]);
        let (temperature_in_2, temperature_out_2) = Self::get_evaporator_temperatures(values[17]);

        Some(Chiller {
            address,

            temperature_setpoint: values[0],

            temperature_high_alarm: values[2],
            temperature_low_alarm: values[3],

            to_process_temperature: values[5],
            from_process_temperature: values[6],

            process_status: values[8],
            machine_status_1: values[9],
            machine_status_2: values[10],
            machine_status_3: values[11],

            heartbeat: values[14],

            zone_1_temperature_evaporator_in: temperature_in_1,
            zone_1_temperature_evaporator_out: temperature_out_1,

            zone_2_temperature_evaporator_in: temperature_in_2,
            zone_2_temperature_evaporator_out: temperature_out_2,

            zone_1_status: values[22],
            zone_2_status: values[23],
        })
    }

    /// Get the evaporator temperatures from a combined u16 value.
    ///
    /// # Arguments
    /// * `temperatures` - The combined u16 value containing the evaporator
    ///   temperatures.
    ///
    /// # Returns
    /// A tuple containing the evaporator inlet and outlet temperatures as u8
    /// values.
    fn get_evaporator_temperatures(temperatures: u16) -> (u8, u8) {
        let evaporator_in = (temperatures >> 8) as u8;
        let evaporator_out = (temperatures & 0xFF) as u8;

        (evaporator_in, evaporator_out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_frame_invalid() {
        // Frame with incorrect length
        let frame_short: [u8; 52] = [0; 52];
        assert!(Chiller::from_frame(&frame_short).is_none());

        // Frame with incorrect data bytes
        let mut frame_wrong_data_bytes: [u8; 53] = [0; 53];
        frame_wrong_data_bytes[2] = 47;
        assert!(Chiller::from_frame(&frame_wrong_data_bytes).is_none());
    }

    #[test]
    fn test_get_evaporator_temperatures() {
        let combined: u16 = 0x1234;
        let (evaporator_in, evaporator_out) = Chiller::get_evaporator_temperatures(combined);

        assert_eq!(evaporator_in, 0x12);
        assert_eq!(evaporator_out, 0x34);
    }
}

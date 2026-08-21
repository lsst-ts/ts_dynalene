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

use crc::{CRC_16_MODBUS, Crc};

use crate::constants::CODE_READ_HOLDING_REGISTERS;
use crate::daq::flowmeter::Flowmeter;
use crate::mock::mock_constants::{
    PLANT_FLOWMETER_FLOW_RATE, PLANT_FLOWMETER_NEGATIVE_TOTALIZER, PLANT_FLOWMETER_NET_TOTALIZER,
    PLANT_FLOWMETER_POSITIVE_TOTALIZER, PLANT_FLOWMETER_SIGNAL_STRENGTH,
};
use crate::utility::calculate_modbus_crc_and_update_frame;

pub struct MockFlowmeterGroup {
    // Cyclic redundancy check (CRC) calculator for the Modbus communication.
    _crc: Crc<u16>,
    // The flowmeters connected to as a group with a daisy chain connection
    // bus.
    pub flowmeters: Vec<Flowmeter>,
}

impl MockFlowmeterGroup {
    /// Mock flowmeter group to simulate the measured flow rate and totalizer
    /// values.
    ///
    /// # Arguments
    /// * `addresses` - The addresses of the flowmeters connected to the hub.
    ///
    /// # Returns
    /// A new instance of `MockFlowmeterGroup`.
    pub fn new(addresses: &[u8]) -> Self {
        let mut flowmeters: Vec<Flowmeter> = addresses
            .iter()
            .map(|address| Flowmeter::new(*address))
            .collect();

        for flowmeter in &mut flowmeters {
            flowmeter.signal_strength = PLANT_FLOWMETER_SIGNAL_STRENGTH;
            flowmeter.flow_rate = PLANT_FLOWMETER_FLOW_RATE;
            flowmeter.positive_totalizer = PLANT_FLOWMETER_POSITIVE_TOTALIZER;
            flowmeter.negative_totalizer = PLANT_FLOWMETER_NEGATIVE_TOTALIZER;
            flowmeter.net_totalizer = PLANT_FLOWMETER_NET_TOTALIZER;
        }

        Self {
            _crc: Crc::<u16>::new(&CRC_16_MODBUS),

            flowmeters,
        }
    }

    /// Request the specified number of registers from the flowmeter.
    ///
    /// # Arguments
    /// * `idx` - The index of the flowmeter in the group.
    /// * `num` - The number of registers to request.
    ///
    /// # Returns
    /// A vector containing the Modbus frame response.
    pub fn request(&self, idx: usize, num: u16) -> Vec<u8> {
        let flowmeter = &self.flowmeters[idx];

        // Each register consists of 2 bytes.
        let data_bytes = num * 2;

        let mut frame_response = vec![0; 5 + (data_bytes as usize)];
        frame_response[0] = flowmeter.address;
        frame_response[1] = CODE_READ_HOLDING_REGISTERS;
        frame_response[2] = data_bytes as u8;

        if num >= 2 {
            frame_response[3..7].copy_from_slice(&flowmeter.signal_strength.to_be_bytes());
        }

        if num >= 4 {
            frame_response[7..11].copy_from_slice(&flowmeter.flow_rate.to_be_bytes());
        }

        if num >= 6 {
            frame_response[11..15].copy_from_slice(&flowmeter.net_totalizer.to_be_bytes());
        }

        if num >= 8 {
            frame_response[15..19].copy_from_slice(&flowmeter.positive_totalizer.to_be_bytes());
        }

        if num >= 10 {
            frame_response[19..23].copy_from_slice(&flowmeter.negative_totalizer.to_be_bytes());
        }

        calculate_modbus_crc_and_update_frame(&self._crc, &mut frame_response);

        frame_response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::NUM_REGISTER_FLOWMETER;

    #[test]
    fn test_request() {
        let group = MockFlowmeterGroup::new(&[1, 3, 6, 20]);

        let response: Vec<u8> = group.request(0, NUM_REGISTER_FLOWMETER);

        assert_eq!(response.len(), 25);
        assert_eq!(response[0], 1);
        assert_eq!(response[1], CODE_READ_HOLDING_REGISTERS);
        assert_eq!(response[2], 20);

        let flowmeter = Flowmeter::from_frame(&response).unwrap();

        assert_eq!(flowmeter, group.flowmeters[0]);
    }
}

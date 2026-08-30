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

use crate::mock::mock_constants::PLANT_PRESSURE;

pub struct MockPressureTransducer {
    // Address of the pressure transducer.
    _address: u8,
    // Absolute pressure in PSI (0-100 PSI).
    pub pressure: f32,
}

impl MockPressureTransducer {
    /// Mock pressure transducer to simulate the measured pressure.
    ///
    /// # Arguments
    /// * `address` - The address of the pressure transducer.
    ///
    /// # Returns
    /// A new instance of `MockPressureTransducer`.
    pub fn new(address: u8) -> Self {
        Self {
            _address: address,
            pressure: PLANT_PRESSURE,
        }
    }

    /// Request the ModBus payload of the measured pressure.
    ///
    /// # Notes
    /// The format of the response is "@<address> <pressure> PSI G\r\n".
    ///
    /// See the svi_PS_Bus_Test.vi in dynalene_system LabVIEW project.
    ///
    /// # Returns
    /// Payload of the measured pressure.
    pub fn request(&self) -> Vec<u8> {
        format!("@{:03} {:07.3} PSI G\r\n", self._address, self.pressure)
            .as_bytes()
            .to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::BYTES_RESPONSE_PRESSURE;
    use std::str::from_utf8;

    #[test]
    fn test_request() {
        let mut pressure_transducer = MockPressureTransducer::new(12);

        // Check the positive pressure.
        let response = pressure_transducer.request();

        assert_eq!(response.len(), BYTES_RESPONSE_PRESSURE);
        assert_eq!(from_utf8(&response).unwrap(), "@012 014.700 PSI G\r\n");

        // Check the negative pressure.
        pressure_transducer.pressure = -4.7123456;
        let response = pressure_transducer.request();

        assert_eq!(response.len(), BYTES_RESPONSE_PRESSURE);
        assert_eq!(from_utf8(&response).unwrap(), "@012 -04.712 PSI G\r\n");
    }
}

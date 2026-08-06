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

use crate::constants::NUM_TEMPERATURE_CHANNEL;
use crate::mock::mock_constants::PLANT_TEMPERATURE;

#[derive(Default, Copy, Clone)]
pub struct MockTemperatureHub {
    // Temperatures in degree Celsius.
    _temperatures: [f64; NUM_TEMPERATURE_CHANNEL],
}

impl MockTemperatureHub {
    /// Mock temperature hub to simulate the measured temperatures.
    ///
    /// # Returns
    /// A new instance of `MockTemperatureHub`.
    pub fn new() -> Self {
        Self {
            _temperatures: [PLANT_TEMPERATURE; NUM_TEMPERATURE_CHANNEL],
        }
    }

    /// Set the temperatures.
    ///
    /// # Arguments
    /// * `temperatures` - An array of temperatures in degree Celsius to set in
    ///   the mock temperature hub. The length of the array must be equal to
    ///   `NUM_TEMPERATURE_CHANNEL`.
    pub fn set_temperatures(&mut self, temperatures: &[f64; NUM_TEMPERATURE_CHANNEL]) {
        self._temperatures = *temperatures;
    }

    /// Get the temperatures.
    ///
    /// # Returns
    /// An array of temperatures in degree Celsius.
    pub fn get_temperatures(&self) -> [f64; NUM_TEMPERATURE_CHANNEL] {
        self._temperatures
    }

    /// Request the ModBus payload of the measured temperatures.
    ///
    /// # Notes
    /// The format of the returned string is:
    /// C01=%f,C02=%f,C03=%f,C04=%f,C05=%f,C06=%f,C07=%f,C08=%f
    ///
    /// See the svi_SEL1403-08PT100_noOpenCloseVisa.vi in dynalene_system
    /// LabVIEW project.
    ///
    /// # Returns
    /// Payload of the measured temperatures.
    pub fn request(&self) -> Vec<u8> {
        let mut temperatures = String::new();
        for (idx, temperature) in self._temperatures.iter().enumerate() {
            if idx > 0 {
                temperatures.push(',');
            }

            // Format the temperature with 9 bytes only.
            temperatures.push_str(&format!("C{:02}={:09.4}", idx + 1, temperature));
        }

        temperatures.push('\n');

        temperatures.as_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::BYTES_RESPONSE_TEMPERATURE;
    use std::str::from_utf8;

    #[test]
    fn test_set_temperatures_and_get_temperatures() {
        let mut mock_temperature_hub = MockTemperatureHub::new();

        let new_temperatures = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        mock_temperature_hub.set_temperatures(&new_temperatures);

        assert_eq!(mock_temperature_hub.get_temperatures(), new_temperatures);
    }

    #[test]
    fn test_request() {
        let mut mock_temperature_hub = MockTemperatureHub::new();

        // Default temperatures.
        let response = mock_temperature_hub.request();

        assert_eq!(response.len(), BYTES_RESPONSE_TEMPERATURE);
        assert_eq!(
            from_utf8(&response).unwrap(),
            "C01=0000.5100,C02=0000.5100,C03=0000.5100,C04=0000.5100,C05=0000.5100,C06=0000.5100,C07=0000.5100,C08=0000.5100\n"
        );

        // Update the temperatures and check the returned string.
        let new_temperatures = [
            123.03,
            12.04,
            3.123456789123,
            -321.03,
            -132.01234567,
            0.0,
            1234.023,
            -1.0,
        ];
        mock_temperature_hub.set_temperatures(&new_temperatures);

        let response = mock_temperature_hub.request();

        assert_eq!(response.len(), BYTES_RESPONSE_TEMPERATURE);
        assert_eq!(
            from_utf8(&response).unwrap(),
            "C01=0123.0300,C02=0012.0400,C03=0003.1235,C04=-321.0300,C05=-132.0123,C06=0000.0000,C07=1234.0230,C08=-001.0000\n"
        );
    }
}

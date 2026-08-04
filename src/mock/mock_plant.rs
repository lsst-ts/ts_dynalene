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

use std::str::from_utf8;

use crate::constants::{
    ADDRESSES_PRESSURE_TRANSDUCER_BUS_0, ADDRESSES_PRESSURE_TRANSDUCER_BUS_1,
    ADDRESSES_PRESSURE_TRANSDUCER_BUS_2, NUM_TEMPERATURE_CHANNEL, NUM_TEMPERATURE_HUB,
};
use crate::mock::{
    mock_pressure_transducer::MockPressureTransducer, mock_temperature_hub::MockTemperatureHub,
};

#[derive(Default)]
pub struct MockPlant {
    // Sensor of the temperature hubs.
    _sensor_temperature_hubs: [MockTemperatureHub; NUM_TEMPERATURE_HUB],
    // Sensor of the pressure transducers.
    _sensor_pressure_transducer_bus_0:
        [MockPressureTransducer; ADDRESSES_PRESSURE_TRANSDUCER_BUS_0.len()],
    _sensor_pressure_transducer_bus_1:
        [MockPressureTransducer; ADDRESSES_PRESSURE_TRANSDUCER_BUS_1.len()],
    _sensor_pressure_transducer_bus_2:
        [MockPressureTransducer; ADDRESSES_PRESSURE_TRANSDUCER_BUS_2.len()],
}

impl MockPlant {
    /// Mock plant model to support the simulation mode of the dynalene control
    /// system.
    ///
    /// # Returns
    /// A new instance of `MockPlant`.
    pub fn new() -> Self {
        Self {
            _sensor_temperature_hubs: [MockTemperatureHub::new(); NUM_TEMPERATURE_HUB],

            _sensor_pressure_transducer_bus_0: ADDRESSES_PRESSURE_TRANSDUCER_BUS_0
                .map(MockPressureTransducer::new),
            _sensor_pressure_transducer_bus_1: ADDRESSES_PRESSURE_TRANSDUCER_BUS_1
                .map(MockPressureTransducer::new),
            _sensor_pressure_transducer_bus_2: ADDRESSES_PRESSURE_TRANSDUCER_BUS_2
                .map(MockPressureTransducer::new),
        }
    }

    /// Set the sensor of temperatures in the mock plant.
    ///
    /// # Arguments
    /// * `idx` - The index of the temperature hub. This index must be less
    ///   than `NUM_TEMPERATURE_HUB`.
    /// * `temperatures` - An array of temperatures in degree Celsius. The
    ///   length of the array must be equal to `NUM_TEMPERATURE_CHANNEL`.
    pub fn set_sensor_temperatures(
        &mut self,
        idx: usize,
        temperatures: &[f64; NUM_TEMPERATURE_CHANNEL],
    ) {
        if idx < NUM_TEMPERATURE_HUB {
            self._sensor_temperature_hubs[idx].set_temperatures(temperatures);
        }
    }

    /// Request to read the measured temperatures from sensor.
    ///
    /// # Notes
    /// The temperatures in index 0 are:
    /// [TS01, TS02, TS03, TS04, TS05, TS06, TS07, TS08]
    ///
    /// The temperatures in index 1 are:
    /// [TS20, TS21, TS22, TS23, TS24, TS25, TS26, TS27]
    ///
    /// The temperatures in index 2 are:
    /// [TS30, TS31, TS32, TS33, None, TS40, TS41, TS42]
    ///
    /// See the svi_HMI.vi in dynalene_system LabVIEW project.
    ///
    /// # Arguments
    /// * `idx` - The index of the temperature hub. This index must be less
    ///   than `NUM_TEMPERATURE_HUB`.
    ///
    /// # Returns
    /// Payload containing the temperature measurements if the index is valid.
    /// Otherwise, `None` is returned.
    pub fn request_sensor_temperatures(&self, idx: usize) -> Option<Vec<u8>> {
        if idx < NUM_TEMPERATURE_HUB {
            Some(self._sensor_temperature_hubs[idx].request())
        } else {
            None
        }
    }

    /// Request to read the measured pressure from sensor.
    ///
    /// # Notes
    /// The format of the command is "#<address>P\r\n". The format of address
    /// is "%03d".
    ///
    /// See the svi_PS_Bus_Test.vi in dynalene_system LabVIEW project.
    ///
    /// # Arguments
    /// * `idx` - The index of the pressure transducer bus. This index must be
    ///   less than `NUM_BUS_PRESSURE_TRANSDUCER`.
    /// * `command` - The command to read the measured pressure.
    ///
    /// # Returns
    /// Payload containing the pressure measurement if the idx and command are
    /// valid. Otherwise, `None` is returned.
    pub fn request_sensor_pressure(&self, idx: usize, command: &[u8]) -> Option<Vec<u8>> {
        let address = from_utf8(command)
            .ok()?
            .trim_start_matches('#')
            .trim_end_matches("P\r\n")
            .parse::<u8>()
            .ok()?;

        match idx {
            0 => {
                let idx_sensor = ADDRESSES_PRESSURE_TRANSDUCER_BUS_0
                    .iter()
                    .position(|x| *x == address)?;

                Some(
                    self._sensor_pressure_transducer_bus_0
                        .get(idx_sensor)?
                        .request(),
                )
            }
            1 => {
                let idx_sensor = ADDRESSES_PRESSURE_TRANSDUCER_BUS_1
                    .iter()
                    .position(|x| *x == address)?;

                Some(
                    self._sensor_pressure_transducer_bus_1
                        .get(idx_sensor)?
                        .request(),
                )
            }
            2 => {
                let idx_sensor = ADDRESSES_PRESSURE_TRANSDUCER_BUS_2
                    .iter()
                    .position(|x| *x == address)?;

                Some(
                    self._sensor_pressure_transducer_bus_2
                        .get(idx_sensor)?
                        .request(),
                )
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::constants::{BYTES_RESPONSE_TEMPERATURE, NUM_BUS_PRESSURE_TRANSDUCER};

    #[test]
    fn test_set_sensor_temperatures() {
        let mut plant = MockPlant::new();

        let temperatures = [20.03; NUM_TEMPERATURE_CHANNEL];
        for idx in 0..NUM_TEMPERATURE_HUB {
            plant.set_sensor_temperatures(idx, &temperatures);

            assert_eq!(
                plant._sensor_temperature_hubs[idx].get_temperatures(),
                temperatures
            );
        }
    }

    #[test]
    fn test_request_sensor_temperatures() {
        let plant = MockPlant::new();

        // Valid
        for idx in 0..NUM_TEMPERATURE_HUB {
            assert_eq!(
                plant.request_sensor_temperatures(idx).unwrap().len(),
                BYTES_RESPONSE_TEMPERATURE
            );
        }

        // Invalid
        assert!(
            plant
                .request_sensor_temperatures(NUM_TEMPERATURE_HUB)
                .is_none()
        )
    }

    #[test]
    fn test_request_sensor_pressure() {
        let plant = MockPlant::new();

        // Valid
        assert_eq!(
            from_utf8(
                &plant
                    .request_sensor_pressure(0, &"#005P\r\n".as_bytes())
                    .unwrap()
            )
            .unwrap(),
            "@005 014.700 PSI G\r\n"
        );
        assert_eq!(
            from_utf8(
                &plant
                    .request_sensor_pressure(0, &"#020P\r\n".as_bytes())
                    .unwrap()
            )
            .unwrap(),
            "@020 014.700 PSI G\r\n"
        );
        assert_eq!(
            from_utf8(
                &plant
                    .request_sensor_pressure(2, &"#004P\r\n".as_bytes())
                    .unwrap()
            )
            .unwrap(),
            "@004 014.700 PSI G\r\n"
        );

        // Invalid
        assert!(
            plant
                .request_sensor_pressure(NUM_BUS_PRESSURE_TRANSDUCER, &"005P\r\n".as_bytes())
                .is_none()
        );
        assert!(
            plant
                .request_sensor_pressure(NUM_BUS_PRESSURE_TRANSDUCER, &"#005\r\n".as_bytes())
                .is_none()
        );

        assert!(
            plant
                .request_sensor_pressure(NUM_BUS_PRESSURE_TRANSDUCER, &"#000P\r\n".as_bytes())
                .is_none()
        );
        assert!(
            plant
                .request_sensor_pressure(0, &"#999P\r\n".as_bytes())
                .is_none()
        );
    }
}

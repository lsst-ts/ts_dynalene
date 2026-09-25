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

pub struct TankLevel {
    // The maximum value of the sensor range.
    _sensor_max_range: f64,
    // The distance for the full level of the tank.
    _distance_full: f64,
    // The distance for the empty level of the tank.
    _distance_empty: f64,
}

impl TankLevel {
    /// Creates a new `TankLevel` with the sensor range and distances to full
    /// and empty levels.
    ///
    /// # Arguments
    /// * `sensor_max_range` - The maximum value of the sensor range.
    /// * `distance_full` - The distance for the full level of the tank.
    /// * `distance_empty` - The distance for the empty level of the tank.
    ///
    /// # Returns
    /// A new instance of `TankLevel`.
    pub fn new(sensor_max_range: f64, distance_full: f64, distance_empty: f64) -> Self {
        Self {
            _sensor_max_range: sensor_max_range,

            _distance_full: distance_full,
            _distance_empty: distance_empty,
        }
    }

    /// Calculates the distance and percentage of the tank level based on the
    /// given ratio.
    ///
    /// # Notes
    /// See the svi_LevelSensorSignalConvertion.vi in dynalene_system LabVIEW
    /// project.
    ///
    /// # Arguments
    /// * `ratio` - The ratio (between 0.0 and 1.0) of the sensor reading to
    ///   the maximum sensor range.
    ///
    /// # Returns
    /// A tuple containing the distance and remaining percentage of the tank
    /// level.
    pub fn calculate_distance_and_percentage(&self, ratio: f64) -> (f64, f64) {
        let distance_to_full = self._sensor_max_range * ratio - self._distance_full;
        let remaining_percentage = (1.0 - distance_to_full / self._distance_empty) * 100.0;

        (distance_to_full, remaining_percentage)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    use crate::constants::MAX_ANALOG_INPUT_OUTPUT;
    use crate::mock::mock_constants::PLANT_TANK_LEVEL_VOLTAGE;

    const EPSILON: f64 = 1e-6;

    #[test]
    fn test_calculate_distance_and_percentage() {
        let tank = TankLevel::new(4300.0, 120.0, 980.0);

        // Zero
        let (distance_to_full, remaining_percentage) = tank.calculate_distance_and_percentage(
            (tank._distance_empty + tank._distance_full) / tank._sensor_max_range,
        );

        assert_eq!(distance_to_full, 980.0);
        assert_eq!(remaining_percentage, 0.0);

        // Full
        let (distance_to_full, remaining_percentage) =
            tank.calculate_distance_and_percentage(tank._distance_full / tank._sensor_max_range);

        assert_eq!(distance_to_full, 0.0);
        assert_eq!(remaining_percentage, 100.0);

        // Random value
        let (distance_to_full, remaining_percentage) = tank
            .calculate_distance_and_percentage(PLANT_TANK_LEVEL_VOLTAGE / MAX_ANALOG_INPUT_OUTPUT);

        assert_eq!(distance_to_full, 263.1773);
        assert_relative_eq!(remaining_percentage, 73.145173, epsilon = EPSILON);
    }
}

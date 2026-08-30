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

use std::collections::HashMap;
use std::path::Path;

use ts_control_utils::utility::get_parameter_array;

#[derive(Debug, Default)]
pub struct Config {
    // Addresses for various devices on different buses.
    pub addresses: HashMap<String, Vec<u8>>,
}

impl Config {
    /// Create a new config object.
    ///
    /// # Returns
    /// A new config object.
    pub fn new() -> Self {
        let filepath = Path::new("config/parameters_app.yaml");
        Self {
            addresses: Config::read_addresses(filepath),
        }
    }

    /// Read addresses from the configuration file.
    ///
    /// # Arguments
    /// * `filepath` - Path to the configuration file.
    ///
    /// # Returns
    /// A HashMap containing addresses for various devices.
    fn read_addresses(filepath: &Path) -> HashMap<String, Vec<u8>> {
        let mut addresses = HashMap::new();
        addresses.insert(
            "pressure_transducer_bus_0".to_string(),
            get_parameter_array(filepath, "addresses_pressure_transducer_bus_0"),
        );
        addresses.insert(
            "pressure_transducer_bus_1".to_string(),
            get_parameter_array(filepath, "addresses_pressure_transducer_bus_1"),
        );
        addresses.insert(
            "pressure_transducer_bus_2".to_string(),
            get_parameter_array(filepath, "addresses_pressure_transducer_bus_2"),
        );

        addresses.insert(
            "flowmeter_bus_0".to_string(),
            get_parameter_array(filepath, "addresses_flowmeter_bus_0"),
        );
        addresses.insert(
            "flowmeter_bus_1".to_string(),
            get_parameter_array(filepath, "addresses_flowmeter_bus_1"),
        );
        addresses.insert(
            "flowmeter_bus_2".to_string(),
            get_parameter_array(filepath, "addresses_flowmeter_bus_2"),
        );

        addresses.insert(
            "power_grid_monitor".to_string(),
            get_parameter_array(filepath, "addresses_power_grid_monitor"),
        );

        addresses.insert(
            "pier_fan".to_string(),
            get_parameter_array(filepath, "addresses_pier_fan"),
        );

        addresses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_addresses() {
        let addresses = Config::read_addresses(Path::new("config/parameters_app.yaml"));

        assert_eq!(addresses.len(), 8);
    }
}

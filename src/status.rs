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

use crate::enums::Commander;

#[derive(Default)]
pub struct Status {
    // Connection status. True if connected. Otherwise, false.
    connections: HashMap<Commander, bool>,
    // Digital inputs (Mod4, NI-9425).
    pub digital_inputs_mod4: u32,
    // Digital inputs (Mod7, NI-9425).
    pub digital_inputs_mod7: u32,
    // Digital outputs (Mod6, NI-9476).
    pub digital_outputs: u32,
}

impl Status {
    /// Create a new status.
    ///
    /// # Returns
    /// A new status.
    pub fn new() -> Self {
        let mut connections = HashMap::new();
        [Commander::CSC, Commander::GUI].iter().for_each(|key| {
            connections.insert(*key, false);
        });

        Self {
            connections,

            digital_inputs_mod4: 0,
            digital_inputs_mod7: 0,

            digital_outputs: 0,
        }
    }

    /// Is the source connected?
    ///
    /// # Arguments
    /// * `source` - Source of the connection.
    ///
    /// # Returns
    /// True if the source is connected. Otherwise, false.
    pub fn is_connected(&self, source: Commander) -> bool {
        self.connections[&source]
    }

    /// Update the connection status.
    ///
    /// # Arguments
    /// * `source` - Source of the connection.
    /// * `is_connected` - Is connected or not.
    pub fn update_connection_status(&mut self, source: Commander, is_connected: bool) {
        if let Some(connection) = self.connections.get_mut(&source) {
            *connection = is_connected;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_status() -> Status {
        Status::new()
    }

    #[test]
    fn test_update_connection_status() {
        let mut status = create_status();

        // Connected.
        status.update_connection_status(Commander::CSC, true);
        assert!(status.is_connected(Commander::CSC));

        status.update_connection_status(Commander::GUI, true);
        assert!(status.is_connected(Commander::GUI));

        // Disconnected.
        status.update_connection_status(Commander::CSC, false);

        assert!(!status.is_connected(Commander::CSC));
    }
}

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

use crate::mock::mock_pressure_transducer::MockPressureTransducer;

pub struct MockPressureTransducerGroup {
    // The pressure transducers connected to as a group with a daisy chain
    // connection bus.
    pub pressure_transducers: Vec<MockPressureTransducer>,
}

impl MockPressureTransducerGroup {
    /// Mock pressure transducer group to simulate the measured pressure
    /// values.
    ///
    /// # Arguments
    /// * `addresses` - The addresses of the pressure transducers connected to
    ///   the group.
    ///
    /// # Returns
    /// A new instance of `MockPressureTransducerGroup`.
    pub fn new(addresses: &[u8]) -> Self {
        Self {
            pressure_transducers: addresses
                .iter()
                .map(|address| MockPressureTransducer::new(*address))
                .collect(),
        }
    }

    /// Requests the pressure measurement from a specific pressure transducer
    /// in the group.
    ///
    /// # Arguments
    /// * `idx` - The index of the pressure transducer to request the
    ///   measurement from.
    ///
    /// # Returns
    /// Payload of the measured pressure.
    pub fn request(&self, idx: usize) -> Vec<u8> {
        self.pressure_transducers[idx].request()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_pressure_transducer_group_new() {
        let addresses = vec![1, 2, 3];
        let group = MockPressureTransducerGroup::new(&addresses);

        assert_eq!(group.pressure_transducers.len(), addresses.len());
    }

    #[test]
    fn test_mock_pressure_transducer_group_request() {
        let addresses = vec![1, 2, 3];
        let group = MockPressureTransducerGroup::new(&addresses);

        for idx in 0..addresses.len() {
            assert!(!group.request(idx).is_empty());
        }
    }
}

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

use serde_json::{Value, json};

use crate::enums::Commander;

pub struct Event;
impl Event {
    /// Get the message that the TCP/IP connection is on or not.
    ///
    /// # Arguments
    /// * `is_connected` - TCP/IP connection is on or not.
    ///
    /// # Returns
    /// The message that the TCP/IP connection is on or not.
    pub fn get_message_tcp_ip_connected(is_connected: bool) -> Value {
        json!({
            "id": "tcpIpConnected",
            "isConnected": is_connected,
        })
    }

    /// Get the message of the current commander.
    ///
    /// # Arguments
    /// * `commander` - The current commander.
    ///
    /// # Returns
    /// The message of the current commander.
    pub fn get_message_commander(commander: Commander) -> Value {
        json!({
            "id": "commander",
            "name": commander.as_ref().to_lowercase(),
        })
    }

    /// Get the message of the digital inputs of Mod4, NI-9425.
    ///
    /// # Arguments
    /// * `digital_inputs` - Digital inputs.
    ///
    /// # Returns
    /// The message of the digital inputs.
    pub fn get_message_digital_inputs_mod4(digital_inputs: u32) -> Value {
        json!({
            "id": "digitalInputsMod4",
            "value": digital_inputs,
        })
    }

    /// Get the message of the digital inputs of Mod7, NI-9425.
    ///
    /// # Arguments
    /// * `digital_inputs` - Digital inputs.
    ///
    /// # Returns
    /// The message of the digital inputs.
    pub fn get_message_digital_inputs_mod7(digital_inputs: u32) -> Value {
        json!({
            "id": "digitalInputsMod7",
            "value": digital_inputs,
        })
    }

    /// Get the message of the digital outputs (Mod6, NI-9476).
    ///
    /// # Arguments
    /// * `digital_outputs` - Digital outputs.
    ///
    /// # Returns
    /// The message of the digital outputs.
    pub fn get_message_digital_outputs(digital_outputs: u32) -> Value {
        json!({
            "id": "digitalOutputs",
            "value": digital_outputs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_message_tcp_ip_connected() {
        assert_eq!(
            Event::get_message_tcp_ip_connected(true),
            json!({
                "id": "tcpIpConnected",
                "isConnected": true,
            })
        );
    }

    #[test]
    fn test_get_message_commander() {
        assert_eq!(
            Event::get_message_commander(Commander::CSC),
            json!({
                "id": "commander",
                "name": "csc",
            })
        );
    }

    #[test]
    fn test_get_message_digital_inputs_mod4() {
        assert_eq!(
            Event::get_message_digital_inputs_mod4(1),
            json!({
                "id": "digitalInputsMod4",
                "value": 1,
            })
        );
    }

    #[test]
    fn test_get_message_digital_inputs_mod7() {
        assert_eq!(
            Event::get_message_digital_inputs_mod7(1),
            json!({
                "id": "digitalInputsMod7",
                "value": 1,
            })
        );
    }

    #[test]
    fn test_get_message_digital_outputs() {
        assert_eq!(
            Event::get_message_digital_outputs(1),
            json!({
                "id": "digitalOutputs",
                "value": 1,
            })
        );
    }
}

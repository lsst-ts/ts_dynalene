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
}

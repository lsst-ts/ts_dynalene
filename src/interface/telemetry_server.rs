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

use serde_json::Value;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;

use crate::constants::BOUND_SYNC_CHANNEL;
use ts_control_utils::tcp_server::TcpServer;

pub struct TelemetryServer {
    // Sender of the messages to the TCP/IP.
    _sender_to_tcp: SyncSender<Vec<Value>>,
    // Receiver of the messages to the TCP/IP.
    pub receiver_to_tcp: Receiver<Vec<Value>>,
}

impl Default for TelemetryServer {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryServer {
    /// Create a new telemetry server instance.
    ///
    /// # Returns
    /// Telemetry server.
    pub fn new() -> Self {
        let (sender_to_tcp, receiver_to_tcp) = sync_channel(BOUND_SYNC_CHANNEL);

        Self {
            _sender_to_tcp: sender_to_tcp,
            receiver_to_tcp,
        }
    }

    /// Get the sender to the TCP/IP.
    ///
    /// # Returns
    /// Sender.
    pub fn get_sender_to_tcp(&self) -> SyncSender<Vec<Value>> {
        self._sender_to_tcp.clone()
    }

    /// Process the telemetry.
    ///
    /// # Arguments
    /// * `tcp_server` - TCP server.
    /// * `telemetry_server` - Telemetry server.
    pub fn process_telemetry(tcp_server: &mut TcpServer, telemetry_server: &mut TelemetryServer) {
        // Check the internal telemetry and send to the TCP/IP.
        if let Ok(telemetry_send) = telemetry_server
            .receiver_to_tcp
            .recv_timeout(Duration::from_millis(tcp_server.timeout))
        {
            tcp_server.write_jsons(&telemetry_send);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;
    use std::net::TcpStream;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    use std::thread::{sleep, spawn};

    use ts_control_utils::{
        constants::{LOCAL_HOST, TERMINATOR},
        utility::client_read_and_assert,
    };

    const SLEEP_TIME: u64 = 50;
    const MAX_TIMEOUT: u64 = 200;

    fn create_tcp_server() -> (TcpServer, Arc<AtomicBool>) {
        let stop = Arc::new(AtomicBool::new(false));
        let server = TcpServer::new(
            "telemetry",
            LOCAL_HOST,
            0,
            SLEEP_TIME,
            &TERMINATOR.to_vec(),
            &stop,
        );

        (server, stop)
    }

    fn create_tcp_client(port: i32) -> TcpStream {
        TcpStream::connect(format!("{}:{}", LOCAL_HOST, port)).expect("Tcp stream should connect.")
    }

    #[test]
    fn test_process_telemetry() {
        let mut telemetry_server = TelemetryServer::new();
        let sender = telemetry_server.get_sender_to_tcp();

        let (mut tcp_server, stop) = create_tcp_server();
        let port = tcp_server.get_port();

        let handle = spawn(move || {
            tcp_server.run(
                TelemetryServer::process_telemetry,
                None::<fn(&mut TcpServer, &mut TelemetryServer)>,
                &mut telemetry_server,
            );
        });

        // Create a client to connect to the server.
        let mut client = create_tcp_client(port);
        sleep(Duration::from_millis(MAX_TIMEOUT));

        // Write the message to the TCP/IP.
        for value in 0..3 {
            sender
                .try_send(vec![
                    json!({"id": "tel_tlm1", "value": value}),
                    json!({"id": "tel_tlm2", "value": value}),
                ])
                .unwrap();

            sleep(Duration::from_millis(SLEEP_TIME));

            client_read_and_assert(
                &mut client,
                &format!(
                    "{{\"id\":\"tel_tlm1\",\"value\":{value}}}\r\n{{\"id\":\"tel_tlm2\",\"value\":{value}}}\r\n"
                ),
            );
        }

        // close the server.
        stop.store(true, Ordering::Relaxed);

        assert!(handle.join().is_ok());
    }
}

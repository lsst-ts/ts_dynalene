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

use log::{debug, info};
use serde_json::Value;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, SyncSender, sync_channel},
};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::command::{
    command_data_acquisition::{
        CommandOpenControlMixingValve, CommandOpenPressureControlValve, CommandResetPierFan,
        CommandSetChillerTemperature, CommandSwitchDigitalOutput,
    },
    command_schema::CommandSchema,
};
use crate::constants::BOUND_SYNC_CHANNEL;
use crate::daq::data_acquisition::DataAcquisition;
use crate::telemetry::telemetry::Telemetry;
use crate::utility::get_message_sequence_id;

pub struct DataAcquisitionProcess {
    // Data acquisition (DAQ) system
    pub daq: DataAcquisition,
    // Command schema
    _command_schema: CommandSchema,
    // Sender of the message to the model
    _sender_to_model: SyncSender<Telemetry>,
    // Sender of the message to DAQ
    _sender_to_daq: SyncSender<Value>,
    // Receiver of the message to DAQ
    _receiver_to_daq: Receiver<Value>,
    // Stop the loop.
    _stop: Arc<AtomicBool>,
}

impl DataAcquisitionProcess {
    /// Create a new data acquisition (DAQ) process.
    ///
    /// # Arguments
    /// * `is_simulation_mode` - Whether the system is in simulation mode.
    /// * `sender_to_model` - The sender of the message to the model.
    /// * `sender_to_daq` - The sender of the message to DAQ.
    /// * `receiver_to_daq` - The receiver of the message to DAQ.
    /// * `stop` - The atomic boolean to stop the loop.
    ///
    /// # Returns
    /// A new data acquisition process.
    pub fn new(
        is_simulation_mode: bool,
        sender_to_model: &SyncSender<Telemetry>,
        sender_to_daq: SyncSender<Value>,
        receiver_to_daq: Receiver<Value>,
        stop: &Arc<AtomicBool>,
    ) -> Self {
        Self {
            daq: DataAcquisition::new(is_simulation_mode),

            _command_schema: Self::create_command_schema(),

            _sender_to_model: sender_to_model.clone(),

            _sender_to_daq: sender_to_daq,
            _receiver_to_daq: receiver_to_daq,

            _stop: stop.clone(),
        }
    }

    /// Create the command schema.
    ///
    /// # Returns
    /// Command schema.
    fn create_command_schema() -> CommandSchema {
        let mut command_schema = CommandSchema::new();
        command_schema.add_command(Box::new(CommandSwitchDigitalOutput));
        command_schema.add_command(Box::new(CommandOpenPressureControlValve));
        command_schema.add_command(Box::new(CommandOpenControlMixingValve));
        command_schema.add_command(Box::new(CommandResetPierFan));
        command_schema.add_command(Box::new(CommandSetChillerTemperature));

        command_schema
    }

    /// Create a sender and receiver to the data acquisition.
    ///
    /// # Notes
    /// This function is to avoid the circular dependency when working with
    /// multiple processes.
    ///
    /// # Returns
    /// A tuple of the sender and receiver to the data acquisition.
    pub fn create_sender_and_receiver_to_data_acquisition() -> (SyncSender<Value>, Receiver<Value>)
    {
        sync_channel(BOUND_SYNC_CHANNEL)
    }

    /// Get the sender to the data acquisition.
    ///
    /// # Returns
    /// The sender to the data acquisition.
    pub fn get_sender_to_data_acquisition(&self) -> SyncSender<Value> {
        self._sender_to_daq.clone()
    }

    /// Run the data acquisition loop.
    pub fn run(&mut self) {
        info!("Data acquisition loop is running.");

        // Counter of the debug message to avoid flooding the log.
        let frequency_loop = self.daq.config.frequency_loop;
        let max_counter_debug = (frequency_loop as i32) - 1;
        let mut counter_debug = 0;

        // Period of the data acquisition loop in milliseconds.
        let period_loop = (1000.0 / frequency_loop) as u64;
        while !self._stop.load(Ordering::Relaxed) {
            // Time the data acquisition loop.
            let now = Instant::now();

            // Process the messages. Use the loop here because the control loop
            // might send multiple messages in a single loop iteration to
            // adjust the valves and others.
            let mut command_result = None;
            while let Ok(message) = self._receiver_to_daq.try_recv() {
                command_result = self.execute_and_check_internal_command(message);

                // Stop processing further messages if a command result is
                // obtained. It should be an external command and we need to
                // report its result to the client.
                if command_result.is_some() {
                    break;
                }
            }

            // Read the telemetry data from the data acquisition.
            let telemetry_daq = self.daq.get_telemetry();

            // Get the events
            let events = if self.daq.event_queue.has_event() {
                Some(self.daq.event_queue.get_events_and_clear())
            } else {
                None
            };

            // Send the telemetry, command result, and events to the model.
            // Ignore the error.
            let _ = self._sender_to_model.try_send(Telemetry::new(
                Some(telemetry_daq),
                command_result,
                events,
            ));

            // Sleep with the remaining time.
            let cycle_time = now.elapsed().as_millis() as u64;

            if counter_debug >= max_counter_debug {
                debug!("Data acquisition process loop time: {} ms.", cycle_time);

                counter_debug = 0;
            } else {
                counter_debug += 1;
            }

            if period_loop > cycle_time {
                sleep(Duration::from_millis(period_loop - cycle_time));
            }
        }

        info!("Data acquisition loop is stopped.");
    }

    /// Execute a command and check if it's an internal command.
    ///
    /// # Arguments
    /// * `message` - The message to execute.
    ///
    /// # Returns
    /// The command result, or None if it's an internal command.
    fn execute_and_check_internal_command(&mut self, message: Value) -> Option<Value> {
        let is_internal_command = get_message_sequence_id(&message) == -1;

        let result = self._command_schema.execute(message, Some(&mut self.daq));

        // For the internal command, no need to send the result.
        if is_internal_command {
            None
        } else {
            Some(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;
    use std::thread::spawn;

    fn create_data_acquisition_process() -> (DataAcquisitionProcess, Receiver<Telemetry>) {
        let (sender_to_model, receiver_to_model) = sync_channel(BOUND_SYNC_CHANNEL);
        let (sender_to_daq, receiver_to_daq) =
            DataAcquisitionProcess::create_sender_and_receiver_to_data_acquisition();

        let stop = Arc::new(AtomicBool::new(false));

        (
            DataAcquisitionProcess::new(
                true,
                &sender_to_model,
                sender_to_daq,
                receiver_to_daq,
                &stop,
            ),
            receiver_to_model,
        )
    }

    #[test]
    fn test_new() {
        let control_loop_process = create_data_acquisition_process().0;

        assert_eq!(control_loop_process._command_schema.number_of_commands(), 5);
    }

    #[test]
    fn test_run() {
        let (mut data_acquisition, receiver_to_model) = create_data_acquisition_process();
        let stop = data_acquisition._stop.clone();

        let sender_to_data_acquisition = data_acquisition.get_sender_to_data_acquisition();
        let handle = spawn(move || {
            data_acquisition.run();
        });

        // Check to get the DAQ telemetry data.
        let received_telemetry_daq;
        loop {
            if let Ok(telemetry_daq) = receiver_to_model.recv_timeout(Duration::from_millis(50)) {
                received_telemetry_daq = telemetry_daq.daq.unwrap();
                break;
            }
        }

        assert_eq!(received_telemetry_daq.digital_outputs, 0);

        // Send multiple internal commands to the data acquisition process.
        for percentage in 0..=20 {
            let _ = sender_to_data_acquisition.try_send(json!({
                "id": "cmd_openControlMixingValve",
                "sequence_id": -1,
                "parameters": {
                    "idx": 0,
                    "percentage": percentage as f64,
                }
            }));
        }

        let mut counter = 0;
        loop {
            if let Ok(telemetry) = receiver_to_model.recv_timeout(Duration::from_millis(50)) {
                assert!(telemetry.command_result.is_none());

                if let Some(telemetry_daq) = &telemetry.daq {
                    if telemetry_daq.percentages_cmv[0] == 20.0 as f64 {
                        break;
                    }
                }

                counter += 1;
            }

            // In case the loop runs too many times without receiving the
            // expected telemetry, break to avoid infinite loop.
            if counter >= 10 {
                break;
            }
        }

        assert!(counter < 10);

        // Turn on the pressure control valve as an external command.
        let _ = sender_to_data_acquisition.try_send(json!({
            "id": "cmd_openPressureControlValve",
            "sequence_id": 2,
            "parameters": {
                "idx": 0,
                "percentage": 10.0,
            },
        }));

        // Check the telemetry data.
        let latest_telemetry_success;
        loop {
            if let Ok(telemetry) = receiver_to_model.recv_timeout(Duration::from_millis(50)) {
                if let Some(_) = &telemetry.command_result {
                    latest_telemetry_success = telemetry;
                    break;
                }
            }
        }

        assert_eq!(
            latest_telemetry_success.command_result.unwrap(),
            json!({
                "id": "cmd_openPressureControlValve",
                "state": "success",
                "sequence_id": 2,
            })
        );
        assert_eq!(
            latest_telemetry_success.daq.unwrap().percentages_pcv,
            vec![10.0, 0.0]
        );

        // Should fail for the wrong index.
        let _ = sender_to_data_acquisition.try_send(json!({
            "id": "cmd_openPressureControlValve",
            "sequence_id": 3,
            "parameters": {
                "idx": 2,
                "percentage": 10.0,
            },
        }));

        let latest_telemetry_fail;
        loop {
            if let Ok(telemetry) = receiver_to_model.recv_timeout(Duration::from_millis(50)) {
                if let Some(_) = &telemetry.command_result {
                    latest_telemetry_fail = telemetry;
                    break;
                }
            }
        }

        assert_eq!(
            latest_telemetry_fail.command_result.unwrap(),
            json!({
                "id": "cmd_openPressureControlValve",
                "state": "fail",
                "sequence_id": 3,
                "fail_reason": "Invalid valve index.",
            })
        );

        // Close the server.
        stop.store(true, Ordering::Relaxed);

        assert!(handle.join().is_ok());
    }

    #[test]
    fn test_execute_and_check_internal_command() {
        let mut data_acquisition_process = create_data_acquisition_process().0;

        // Test an internal command.
        let command_internal = json!({
            "id": "cmd_openPressureControlValve",
            "parameters": {
                "idx": 0,
                "percentage": 10.0,
            },
        });
        let result_internal =
            data_acquisition_process.execute_and_check_internal_command(command_internal);

        assert!(result_internal.is_none());

        // Test a non-internal command.
        let command_external = json!({
            "id": "cmd_openPressureControlValve",
            "sequence_id": 2,
            "parameters": {
                "idx": 0,
                "percentage": 10.0,
            },
        });
        let result_external =
            data_acquisition_process.execute_and_check_internal_command(command_external);

        assert_eq!(
            result_external.unwrap(),
            json!({
                "id": "cmd_openPressureControlValve",
                "state": "success",
                "sequence_id": 2,
            })
        );
    }
}

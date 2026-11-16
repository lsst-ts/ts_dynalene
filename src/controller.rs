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

use log::info;
use std::sync::mpsc::SyncSender;

use crate::command::command_format::CommandFormat;
use crate::enums::Commander;
use crate::event_queue::EventQueue;
use crate::status::Status;
use crate::telemetry::{event::Event, telemetry::Telemetry};

pub struct Controller {
    // System status
    pub status: Status,
    // The commander: CSC or GUI.
    pub commander: Commander,
    // Events to publish
    pub event_queue: EventQueue,
    // Last effective telemetry
    pub last_effective_telemetry: Telemetry,
    // Sender to the data acquisition
    pub sender_to_daq: Option<SyncSender<CommandFormat>>,
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

impl Controller {
    /// Create a new controller.
    ///
    /// # Returns
    /// A new controller.
    pub fn new() -> Self {
        Self {
            status: Status::new(),

            // By default, the commander is CSC.
            commander: Commander::CSC,

            event_queue: EventQueue::new(),

            last_effective_telemetry: Telemetry::new(None, None, None),

            sender_to_daq: None,
        }
    }

    /// Set the commander.
    ///
    /// # Arguments
    /// * `commander` - The commander: CSC or GUI.
    ///
    /// # Returns
    /// Ok(()) if the commander is set successfully. Otherwise, Err with
    /// a failure reason.
    pub fn set_commander(&mut self, commander: Commander) -> Result<(), String> {
        self.commander = commander;
        self.event_queue
            .add_event(Event::get_message_commander(commander));

        info!("Commander set to {:?}.", commander);

        Ok(())
    }

    /// Update the internal status of digital inputs of Mod4, NI-9425.
    ///
    /// # Arguments
    /// * `digital_inputs` - Digital inputs.
    ///
    /// # Returns
    /// True if the status is updated. Otherwise, false.
    pub fn update_internal_status_digital_inputs_mod4(&mut self, digital_inputs: u32) -> bool {
        if self.status.digital_inputs_mod4 != digital_inputs {
            self.status.digital_inputs_mod4 = digital_inputs;

            info!(
                "Digital inputs (Mod4, NI-9425) updated to {:#X}.",
                digital_inputs
            );

            return true;
        }

        false
    }

    /// Update the internal status of digital inputs of Mod7, NI-9425.
    ///
    /// # Arguments
    /// * `digital_inputs` - Digital inputs.
    ///
    /// # Returns
    /// True if the status is updated. Otherwise, false.
    pub fn update_internal_status_digital_inputs_mod7(&mut self, digital_inputs: u32) -> bool {
        if self.status.digital_inputs_mod7 != digital_inputs {
            self.status.digital_inputs_mod7 = digital_inputs;

            info!(
                "Digital inputs (Mod7, NI-9425) updated to {:#X}.",
                digital_inputs
            );

            return true;
        }

        false
    }

    /// Update the internal status of digital outputs (Mod6, NI-9476).
    ///
    /// # Arguments
    /// * `digital_outputs` - Digital outputs.
    ///
    /// # Returns
    /// True if the status is updated. Otherwise, false.
    pub fn update_internal_status_digital_outputs(&mut self, digital_outputs: u32) -> bool {
        if self.status.digital_outputs != digital_outputs {
            self.status.digital_outputs = digital_outputs;

            info!(
                "Digital outputs (Mod6, NI-9476) updated to {:#X}.",
                digital_outputs
            );

            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_commander() {
        let mut controller = Controller::new();

        assert!(controller.set_commander(Commander::GUI).is_ok());

        assert_eq!(controller.commander, Commander::GUI);
        assert_eq!(
            controller.event_queue.get_events_and_clear(),
            vec![Event::get_message_commander(Commander::GUI)],
        );
    }

    #[test]
    fn test_update_internal_status_digital_inputs_mod4() {
        let mut controller = Controller::new();

        assert!(controller.update_internal_status_digital_inputs_mod4(1));
        assert_eq!(controller.status.digital_inputs_mod4, 1);

        assert!(!controller.update_internal_status_digital_inputs_mod4(1));
    }

    #[test]
    fn test_update_internal_status_digital_inputs_mod7() {
        let mut controller = Controller::new();

        assert!(controller.update_internal_status_digital_inputs_mod7(1));
        assert_eq!(controller.status.digital_inputs_mod7, 1);

        assert!(!controller.update_internal_status_digital_inputs_mod7(1));
    }

    #[test]
    fn test_update_internal_status_digital_outputs() {
        let mut controller = Controller::new();

        assert!(controller.update_internal_status_digital_outputs(1));
        assert_eq!(controller.status.digital_outputs, 1);

        assert!(!controller.update_internal_status_digital_outputs(1));
    }
}

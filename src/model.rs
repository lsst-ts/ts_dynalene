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

use log::{debug, error, info};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    thread::{JoinHandle, spawn},
};

#[cfg(feature = "realtime")]
use libc::{SCHED_FIFO, pthread_self, pthread_setschedparam, sched_get_priority_max, sched_param};

use crate::command::{
    command_controller::CommandSwitchCommandSource,
    command_data_acquisition::{
        CommandOpenControlMixingValve, CommandOpenPressureControlValve, CommandResetPierFan,
        CommandSetChillerTemperature, CommandSwitchDigitalOutput,
    },
    command_format::{CommandFormat, CommandId},
    command_schema::{Command, CommandSchema},
};
use crate::constants::BOUND_SYNC_CHANNEL;
use crate::controller::Controller;
use crate::daq::data_acquisition_process::DataAcquisitionProcess;
use crate::enums::{CommandStatus, Commander};
use crate::interface::command_telemetry_server::CommandTelemetryServer;
use crate::telemetry::{
    event::Event, telemetry::Telemetry, telemetry_data_acquisition::TelemetryDataAcquisition,
    telemetry_default::TelemetryDefault,
};
use crate::utility::{acknowledge_command, get_message_name};
use ts_control_utils::utility::get_parameter;

#[allow(clippy::type_complexity)]
pub struct Model {
    // Controller
    controller: Controller,
    // Controller's command schema
    controller_command_schema: CommandSchema,
    // Commands
    commands: HashMap<String, Vec<CommandId>>,
    // Is the simulation mode or not.
    is_simulation_mode: bool,
    // The digit of the telemetry.
    telemetry_digit: i32,
    // Servers.
    host: String,
    ports: HashMap<String, i32>,
    // Receiver of the events from the command-telemetry server.
    receivers_from_command_telemetry_server: HashMap<Commander, Option<Receiver<Value>>>,
    // Receivers from the TCP/IP command servers.
    receivers_from_tcp_command: HashMap<Commander, Option<Receiver<CommandFormat>>>,
    // Receiver of the telemetry to the model.
    receiver_to_model: Receiver<Telemetry>,
    // Senders of the messages to the TCP/IP command-telemetry servers.
    // The value is a tuple of the command (0) and telemetry (1) senders.
    senders_to_tcp: HashMap<
        Commander,
        (
            Option<SyncSender<Vec<Value>>>,
            Option<SyncSender<Vec<Value>>>,
        ),
    >,
    // Sender of the telemetry to the model.
    sender_to_model: Option<SyncSender<Telemetry>>,
    // An Arc instance that holds the AtomicBool instance to stop the threads.
    pub stop: Arc<AtomicBool>,
    // Handles of the threads.
    handles: Vec<JoinHandle<()>>,
    // Flag to stop publishing telemetry. This is for the test purpose only.
    // For the OS such as MacOS, the TCP/IP connection will be closed when the
    // pipeis full, which will cause the test to fail. This flag is used to
    // prevent publishing telemetry in such cases.
    stop_publish_telemetry: bool,
}

impl Model {
    /// Create a new model.
    ///
    /// # Arguments
    /// * `is_simulation_mode` - Is the simulation mode or not.
    /// * `host` - Host.
    /// * `port_command_gui` - Command port for the GUI.
    /// * `port_telemetry_gui` - Telemetry port for the GUI.
    /// * `port_command_csc` - Command port for the CSC.
    /// * `port_telemetry_csc` - Telemetry port for the CSC.
    ///
    /// # Returns
    /// A new model.
    pub fn new(
        is_simulation_mode: bool,
        host: &str,
        port_command_gui: i32,
        port_telemetry_gui: i32,
        port_command_csc: i32,
        port_telemetry_csc: i32,
    ) -> Self {
        let config_file = Path::new("config/parameters_app.yaml");
        let stop = Arc::new(AtomicBool::new(false));

        let commands = Self::create_commands();

        let mut ports = HashMap::new();
        ports.insert(String::from("gui_command"), port_command_gui);
        ports.insert(String::from("gui_telemetry"), port_telemetry_gui);
        ports.insert(String::from("csc_command"), port_command_csc);
        ports.insert(String::from("csc_telemetry"), port_telemetry_csc);

        let mut receivers_from_command_telemetry_server = HashMap::new();
        receivers_from_command_telemetry_server.insert(Commander::GUI, None);
        receivers_from_command_telemetry_server.insert(Commander::CSC, None);

        let mut receivers_from_tcp_command = HashMap::new();
        receivers_from_tcp_command.insert(Commander::GUI, None);
        receivers_from_tcp_command.insert(Commander::CSC, None);

        let mut senders_to_tcp = HashMap::new();
        senders_to_tcp.insert(Commander::GUI, (None, None));
        senders_to_tcp.insert(Commander::CSC, (None, None));

        let (sender_to_model, receiver_to_model) = sync_channel(BOUND_SYNC_CHANNEL);

        Self {
            controller: Controller::new(),
            controller_command_schema: Self::create_controller_command_schema(),
            commands,

            is_simulation_mode,
            telemetry_digit: get_parameter(config_file, "telemetry_digit"),

            host: host.to_string(),
            ports,

            receivers_from_command_telemetry_server,
            receivers_from_tcp_command,

            receiver_to_model,

            senders_to_tcp,

            sender_to_model: Some(sender_to_model),

            stop,

            handles: Vec::new(),

            stop_publish_telemetry: false,
        }
    }

    /// Create commands.
    ///
    /// # Returns
    /// Commands.
    fn create_commands() -> HashMap<String, Vec<CommandId>> {
        // The commands here need to be in the
        // DataAcquisitionProcess.create_command_schema()
        let commands_data_acquisition = vec![
            CommandSwitchDigitalOutput.id(),
            CommandOpenPressureControlValve.id(),
            CommandOpenControlMixingValve.id(),
            CommandResetPierFan.id(),
            CommandSetChillerTemperature.id(),
        ];

        let controller_command_schema = Self::create_controller_command_schema();
        let commands_controller = controller_command_schema.commands.keys().cloned().collect();

        let mut commands = HashMap::new();
        commands.insert(String::from("data_acquisition"), commands_data_acquisition);
        commands.insert(String::from("controller"), commands_controller);

        commands
    }

    /// Create the controller's command schema.
    ///
    /// # Returns
    /// Command schema.
    fn create_controller_command_schema() -> CommandSchema {
        let mut command_schema = CommandSchema::new();
        command_schema.add_command(Box::new(CommandSwitchCommandSource));

        command_schema
    }

    /// Stop the threads.
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);

        for handle in self.handles.drain(..) {
            handle.join().expect("Process handle should join.");
        }
    }

    /// Run the processes.
    pub fn run_processes(&mut self) {
        self.run_command_telemetry_server();

        let (sender_to_daq, receiver_to_daq) =
            DataAcquisitionProcess::create_sender_and_receiver_to_data_acquisition();
        self.controller.sender_to_daq = Some(sender_to_daq.clone());

        self.run_control_loop(&sender_to_daq);

        self.run_data_acquisition(sender_to_daq, receiver_to_daq);

        // Drop the internal sender to the model. This is to let the self.step()
        // wakes up when all the senders are dropped once we stop the
        // application.
        self.sender_to_model = None;
    }

    /// Run the command and telemetry servers.
    fn run_command_telemetry_server(&mut self) {
        // Run the servers
        let (
            mut server_gui,
            mut server_csc,
            receiver_command_gui,
            receiver_command_csc,
            receiver_command_telemetry_server_gui,
            receiver_command_telemetry_server_csc,
        ) = Model::create_servers(
            Path::new("config/parameters_app.yaml"),
            &self.host,
            self.ports["gui_command"],
            self.ports["gui_telemetry"],
            self.ports["csc_command"],
            self.ports["csc_telemetry"],
            &self.stop,
        );

        self.receivers_from_tcp_command
            .insert(Commander::GUI, Some(receiver_command_gui));
        self.receivers_from_tcp_command
            .insert(Commander::CSC, Some(receiver_command_csc));

        self.receivers_from_command_telemetry_server
            .insert(Commander::GUI, Some(receiver_command_telemetry_server_gui));
        self.receivers_from_command_telemetry_server
            .insert(Commander::CSC, Some(receiver_command_telemetry_server_csc));

        let (tcp_port_command_gui, tcp_port_telemetry_gui) = server_gui.run_servers();
        let (tcp_port_command_csc, tcp_port_telemetry_csc) = server_csc.run_servers();

        self.senders_to_tcp.insert(
            Commander::GUI,
            (
                server_gui.senders_to_tcp["command"].clone(),
                server_gui.senders_to_tcp["telemetry"].clone(),
            ),
        );
        self.senders_to_tcp.insert(
            Commander::CSC,
            (
                server_csc.senders_to_tcp["command"].clone(),
                server_csc.senders_to_tcp["telemetry"].clone(),
            ),
        );

        // Update the ports
        self.ports
            .insert(String::from("gui_command"), tcp_port_command_gui);
        self.ports
            .insert(String::from("gui_telemetry"), tcp_port_telemetry_gui);
        self.ports
            .insert(String::from("csc_command"), tcp_port_command_csc);
        self.ports
            .insert(String::from("csc_telemetry"), tcp_port_telemetry_csc);

        info!(
            "GUI command port: {tcp_port_command_gui}, GUI telemetry port: {tcp_port_telemetry_gui}."
        );
        info!(
            "CSC command port: {tcp_port_command_csc}, CSC telemetry port: {tcp_port_telemetry_csc}."
        );

        // Run the monitor loops
        self.run_monitor_loop(server_gui);
        self.run_monitor_loop(server_csc);
    }

    /// Create servers.
    ///
    /// # Arguments
    /// * `config_file` - Configuration file.
    /// * `host` - Host.
    /// * `port_command_gui` - Command port for the GUI.
    /// * `port_telemetry_gui` - Telemetry port for the GUI.
    /// * `port_command_csc` - Command port for the CSC.
    /// * `port_telemetry_csc` - Telemetry port for the CSC.
    /// * `stop` - Stop flag.
    ///
    /// # Returns
    /// * `CommandTelemetryServer` - GUI server.
    /// * `CommandTelemetryServer` - CSC server.
    /// * `Receiver<CommandFormat>` - Receiver of the command from GUI.
    /// * `Receiver<CommandFormat>` - Receiver of the command from CSC.
    /// * `Receiver<Value>` - Receiver from the GUI command-telemetry server.
    /// * `Receiver<Value>` - Receiver from the CSC command-telemetry server.
    fn create_servers(
        config_file: &Path,
        host: &str,
        port_command_gui: i32,
        port_telemetry_gui: i32,
        port_command_csc: i32,
        port_telemetry_csc: i32,
        stop: &Arc<AtomicBool>,
    ) -> (
        CommandTelemetryServer,
        CommandTelemetryServer,
        Receiver<CommandFormat>,
        Receiver<CommandFormat>,
        Receiver<Value>,
        Receiver<Value>,
    ) {
        let timeout = get_parameter(config_file, "timeout");

        let (sender_command_gui, receiver_command_gui) = sync_channel(BOUND_SYNC_CHANNEL);
        let (sender_command_csc, receiver_command_csc) = sync_channel(BOUND_SYNC_CHANNEL);
        let (sender_command_telemetry_server_gui, receiver_command_telemetry_server_gui) =
            sync_channel(BOUND_SYNC_CHANNEL);
        let (sender_command_telemetry_server_csc, receiver_command_telemetry_server_csc) =
            sync_channel(BOUND_SYNC_CHANNEL);

        let server_gui = CommandTelemetryServer::new(
            Commander::GUI,
            host,
            port_command_gui,
            port_telemetry_gui,
            timeout,
            stop,
            sender_command_gui,
            sender_command_telemetry_server_gui,
        );
        let server_csc = CommandTelemetryServer::new(
            Commander::CSC,
            host,
            port_command_csc,
            port_telemetry_csc,
            timeout,
            stop,
            sender_command_csc,
            sender_command_telemetry_server_csc,
        );

        (
            server_gui,
            server_csc,
            receiver_command_gui,
            receiver_command_csc,
            receiver_command_telemetry_server_gui,
            receiver_command_telemetry_server_csc,
        )
    }

    /// Run the monitor loop.
    ///
    /// # Arguments
    /// * `server` - Command telemetry server.
    fn run_monitor_loop(&mut self, mut server: CommandTelemetryServer) {
        let handle = spawn(move || {
            server.run_monitor_loop();
        });

        self.handles.push(handle);
    }

    /// Run the control loop.
    ///
    /// # Arguments
    /// * `sender_to_daq` - The sender to the data acquisition.
    fn run_control_loop(&mut self, _sender_to_daq: &SyncSender<CommandFormat>) {
        // TODO: Implement the control loop logic.
    }

    /// Run the data acquisition process.
    ///
    /// # Arguments
    /// * `sender_to_daq` - Sender to the data acquisition process.
    /// * `receiver_to_daq` - Receiver to the data acquisition process.
    fn run_data_acquisition(
        &mut self,
        sender_to_daq: SyncSender<CommandFormat>,
        receiver_to_daq: Receiver<CommandFormat>,
    ) {
        let mut daq = DataAcquisitionProcess::new(
            self.is_simulation_mode,
            self.sender_to_model
                .as_ref()
                .expect("Should have a message sender to the model."),
            sender_to_daq,
            receiver_to_daq,
            &self.stop,
        );

        #[cfg(feature = "realtime")]
        {
            unsafe {
                let param = sched_param {
                    sched_priority: sched_get_priority_max(SCHED_FIFO),
                };

                let handle = spawn(move || {
                    // Configure the spawned thread itself as realtime before
                    // starting the data acquisition process.
                    if pthread_setschedparam(pthread_self(), SCHED_FIFO, &param) != 0 {
                        let message =
                            "Failed to set the realtime scheduler for data acquisition thread.";
                        error!("{}", message);
                        panic!("{}", message);
                    }
                    daq.run();
                });

                self.handles.push(handle);
            }
        }
        #[cfg(not(feature = "realtime"))]
        {
            let handle = spawn(move || {
                daq.run();
            });

            self.handles.push(handle);
        }
    }

    /// Step the model. This function has a blocking call to wait for the
    /// new telemetry.
    pub fn step(&mut self) {
        // Check the messages from the command-telemetry servers
        for commander in [Commander::GUI, Commander::CSC] {
            if let Some(receiver) = &self.receivers_from_command_telemetry_server[&commander]
                && let Ok(message) = receiver.try_recv()
            {
                Self::process_message(
                    &mut self.controller,
                    message,
                    commander,
                    self.senders_to_tcp[&commander]
                        .0
                        .as_ref()
                        .unwrap_or_else(|| {
                            panic!("Should have a command/event sender to the {:?}.", commander)
                        }),
                );
            }
        }

        // Check the command from the GUI
        let commander_before_gui_command = self.controller.commander;
        let mut gui_has_valid_command = false;
        if let Some(receiver) = &self.receivers_from_tcp_command[&Commander::GUI]
            && let Ok(command) = receiver.try_recv()
        {
            gui_has_valid_command = Self::process_command(
                &mut self.controller,
                &self.commands,
                &self.controller_command_schema,
                command,
                Commander::GUI,
                self.senders_to_tcp[&Commander::GUI]
                    .0
                    .as_ref()
                    .expect("Should have a command/event sender to the GUI."),
            );
        };

        let change_commander_from_gui_to_csc = (commander_before_gui_command == Commander::GUI)
            && (self.controller.commander == Commander::CSC);

        // Change the commander if the GUI has a valid command. But do not do
        // this if this is to switch the commander back to CSC.
        if gui_has_valid_command
            && (!change_commander_from_gui_to_csc)
            && (self.controller.commander != Commander::GUI)
        {
            let _ = self.controller.set_commander(Commander::GUI);
        }

        // Check the message from the CSC
        if let Some(receiver) = &self.receivers_from_tcp_command[&Commander::CSC]
            && let Ok(command) = receiver.try_recv()
        {
            Self::process_command(
                &mut self.controller,
                &self.commands,
                &self.controller_command_schema,
                command,
                Commander::CSC,
                self.senders_to_tcp[&Commander::CSC]
                    .0
                    .as_ref()
                    .expect("Should have a command/event sender to the CSC."),
            );
        };

        // Check the telemetry message. Note the blocking call of the recv() is
        // used here. This is because the model should always get the telemetry
        // message regualarly if the data acquisition process is running.
        match self.receiver_to_model.recv() {
            Ok(telemetry) => {
                let mut all_events: Vec<Value> = Vec::new();
                if let Some(mut events) = telemetry.events {
                    all_events.append(&mut events);
                }

                if let Some(telemetry_data_acquisition) = telemetry.daq {
                    // Update the internal status.
                    let mut events_data_acquisition =
                        self.update_status_telemetry_data_acquisition(&telemetry_data_acquisition);
                    all_events.append(&mut events_data_acquisition);

                    // Publish the telemetry
                    let messages = telemetry_data_acquisition.get_messages(self.telemetry_digit);
                    self.publish_telemetry(messages);

                    // Update the last effective telemetry.
                    self.controller.last_effective_telemetry.daq = Some(telemetry_data_acquisition);
                }

                // Publish the controller events.
                if self.controller.event_queue.has_event() {
                    all_events.append(&mut self.controller.event_queue.get_events_and_clear());
                }

                if !all_events.is_empty() {
                    self.publish_events(all_events);
                }

                if let Some(command_result) = telemetry.command_result {
                    if let Some(sender) = &self.senders_to_tcp[&self.controller.commander].0 {
                        let _ = sender.try_send(vec![command_result.clone()]);
                    }

                    // Update the last effective telemetry.
                    self.controller.last_effective_telemetry.command_result = Some(command_result);
                }
            }

            Err(_) => {
                info!("All senders to the model are dropped.")
            }
        }
    }

    /// Process the received message.
    ///
    /// # Arguments
    /// * `controller` - Controller.
    /// * `message` - Received message.
    /// * `source` - Source of the message.
    /// * `sender_to_tcp` - Sender to the TCP/IP.
    fn process_message(
        controller: &mut Controller,
        message: Value,
        source: Commander,
        sender_to_tcp: &SyncSender<Vec<Value>>,
    ) {
        debug!("Process the received message: {message}.");

        let name = get_message_name(&message);
        if name == "tcpIpConnected"
            && let Some(is_connected) = message["isConnected"].as_bool()
        {
            controller
                .status
                .update_connection_status(source, is_connected);
            if is_connected {
                let _ = sender_to_tcp.try_send(Self::get_welcome_messages(controller));
            }
        }
    }

    /// Get the welcome messages.
    ///
    /// # Arguments
    /// * `controller` - Controller.
    ///
    /// # Returns
    /// Welcome messages.
    fn get_welcome_messages(controller: &Controller) -> Vec<Value> {
        let status = &controller.status;

        vec![
            Event::get_message_commander(controller.commander),
            Event::get_message_digital_inputs_mod4(status.digital_inputs_mod4),
            Event::get_message_digital_inputs_mod7(status.digital_inputs_mod7),
            Event::get_message_digital_outputs(status.digital_outputs),
        ]
    }

    /// Process the received command.
    ///
    /// # Arguments
    /// * `controller` - Controller.
    /// * `all_commands` - All available commands.
    /// * `controller_command_schema` - Controller's command schema.
    /// * `command` - Received command.
    /// * `source` - Source of the command.
    /// * `sender_to_tcp` - Sender to the TCP/IP.
    ///
    /// # Returns
    /// Is the valid command or not.
    fn process_command(
        controller: &mut Controller,
        all_commands: &HashMap<String, Vec<CommandId>>,
        controller_command_schema: &CommandSchema,
        command: CommandFormat,
        source: Commander,
        sender_to_tcp: &SyncSender<Vec<Value>>,
    ) -> bool {
        let tag_id = command.payload.get_tag_id();
        debug!("Process the received command: {tag_id}.",);

        // Fail the command if the source is CSC but the commander is GUI
        if (controller.commander == Commander::GUI) && (source == Commander::CSC) {
            error!(
                "The command {tag_id} from CSC is rejected because the current commander is GUI."
            );
            let _ = sender_to_tcp.try_send(vec![acknowledge_command(
                Some(tag_id),
                CommandStatus::Fail,
                command.sequence_id,
                Some("Current commander is GUI instead of CSC."),
            )]);

            return false;
        }

        // Process the command
        let command_id = CommandId::from(&command.payload);

        // Pass the data acquisition command to the data acquisition process
        if all_commands["data_acquisition"].contains(&command_id) {
            if let Some(sender) = controller.sender_to_daq.as_ref() {
                let _ = sender.try_send(command);
            }

            return true;
        }

        // Command for the controller
        if all_commands["controller"].contains(&command_id) {
            let result = controller_command_schema.execute(&command, None, Some(controller));
            let _ = sender_to_tcp.try_send(vec![result]);

            return true;
        }

        false
    }

    /// Update the status according to the new telemetry of data acquisition
    /// system.
    ///
    /// # Arguments
    /// * `telemetry` - Telemetry of the data acquisition system.
    ///
    /// # Returns
    /// The event messages.
    fn update_status_telemetry_data_acquisition(
        &mut self,
        telemetry: &TelemetryDataAcquisition,
    ) -> Vec<Value> {
        let mut events = Vec::new();

        // Update the digital inputs
        let digital_inputs_mod4 = telemetry.digital_inputs_mod4;
        if self
            .controller
            .update_internal_status_digital_inputs_mod4(digital_inputs_mod4)
        {
            events.push(Event::get_message_digital_inputs_mod4(digital_inputs_mod4));
        }

        let digital_inputs_mod7 = telemetry.digital_inputs_mod7;
        if self
            .controller
            .update_internal_status_digital_inputs_mod7(digital_inputs_mod7)
        {
            events.push(Event::get_message_digital_inputs_mod7(digital_inputs_mod7));
        }

        // Update the digital outputs
        let digital_outputs = telemetry.digital_outputs;
        if self
            .controller
            .update_internal_status_digital_outputs(digital_outputs)
        {
            events.push(Event::get_message_digital_outputs(digital_outputs));
        }

        events
    }

    /// Publish the telemetry.
    ///
    /// # Arguments
    /// * `messages` - Messages.
    ///
    /// # Returns
    /// Did the publishment or not.
    fn publish_telemetry(&self, messages: Vec<Value>) -> bool {
        // No connection or stop publishing telemetry, return immediately
        if (!self.is_connected()) || self.stop_publish_telemetry {
            return false;
        }

        // Publish the messages
        let (is_connected_gui, is_connected_csc) = self.get_connection_status();
        if is_connected_gui {
            let _ = self.senders_to_tcp[&Commander::GUI]
                .1
                .as_ref()
                .expect("Should have a telemetry sender to the GUI.")
                .try_send(messages.clone());
        }
        if is_connected_csc {
            let _ = self.senders_to_tcp[&Commander::CSC]
                .1
                .as_ref()
                .expect("Should have a telemetry sender to the CSC.")
                .try_send(messages);
        }

        true
    }

    /// Check there is a connection or not.
    ///
    /// # Returns
    /// Is connected or not.
    fn is_connected(&self) -> bool {
        let (is_connected_gui, is_connected_csc) = self.get_connection_status();
        is_connected_gui || is_connected_csc
    }

    /// Get the connection status.
    ///
    /// # Returns
    /// Connection status. The first element is the GUI and the second element
    /// is the CSC.
    fn get_connection_status(&self) -> (bool, bool) {
        (
            self.controller.status.is_connected(Commander::GUI),
            self.controller.status.is_connected(Commander::CSC),
        )
    }

    /// Publish the events.
    ///
    /// # Arguments
    /// * `messages` - Messages.
    ///
    /// # Returns
    /// Did the publishment or not.
    fn publish_events(&self, events: Vec<Value>) -> bool {
        // No connection, return immediately
        if !self.is_connected() {
            return false;
        }

        // Publish the events
        let (is_connected_gui, is_connected_csc) = self.get_connection_status();
        if is_connected_gui {
            let _ = self.senders_to_tcp[&Commander::GUI]
                .0
                .as_ref()
                .expect("Should have a command sender to the GUI.")
                .try_send(events.clone());
        }
        if is_connected_csc {
            let _ = self.senders_to_tcp[&Commander::CSC]
                .0
                .as_ref()
                .expect("Should have a command sender to the CSC.")
                .try_send(events);
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::net::{Shutdown, TcpStream};
    use std::sync::atomic::Ordering;
    use std::thread::sleep;
    use std::time::Duration;

    use ts_control_utils::{
        constants::{LOCAL_HOST, TERMINATOR},
        utility::{client_read_and_assert, client_read_json, client_write_and_sleep},
    };

    const MAX_TIME_WAIT_FOR_COMMAND_RESULT: i32 = 500;
    const SLEEP_TIME: u64 = 100;
    const MAX_TIMEOUT: u64 = 200;

    fn create_model() -> Model {
        Model::new(true, LOCAL_HOST, 0, 0, 0, 0)
    }

    fn create_tcp_clients(port_command: i32, port_telemetry: i32) -> (TcpStream, TcpStream) {
        (
            TcpStream::connect(format!("{}:{}", LOCAL_HOST, port_command))
                .expect("Tcp command stream should connect."),
            TcpStream::connect(format!("{}:{}", LOCAL_HOST, port_telemetry))
                .expect("Tcp telemetry stream should connect."),
        )
    }

    fn wait_for_connection(model: &mut Model) {
        for _ in 0..MAX_TIME_WAIT_FOR_COMMAND_RESULT {
            model.step();
            if model.is_connected() {
                break;
            }
        }
    }

    fn consume_welcome_messages(model: &Model, client: &mut TcpStream) -> Vec<Value> {
        let welcome_messages = Model::get_welcome_messages(&model.controller);
        let num_welcome_messages = welcome_messages.len();

        let mut messages = Vec::new();
        for _ in 0..num_welcome_messages {
            messages.push(client_read_json(client, TERMINATOR));
        }

        messages
    }

    fn wait_for_command_result_and_reset(model: &mut Model) {
        for _ in 0..MAX_TIME_WAIT_FOR_COMMAND_RESULT {
            model.step();
            if model
                .controller
                .last_effective_telemetry
                .command_result
                .is_some()
            {
                model.controller.last_effective_telemetry.command_result = None;
                break;
            }
        }
    }

    #[test]
    fn test_new() {
        let model = create_model();

        assert_eq!(model.controller_command_schema.number_of_commands(), 1);
        assert_eq!(model.commands["data_acquisition"].len(), 5);
        assert_eq!(model.commands["controller"].len(), 1);
    }

    #[test]
    fn test_stop() {
        let mut model = create_model();

        model.stop();

        assert_eq!(model.stop.load(Ordering::Relaxed), true);
    }

    #[test]
    fn test_run_command_telemetry_server() {
        let mut model = create_model();

        model.run_command_telemetry_server();

        for (_, value) in &model.ports {
            assert_ne!(*value, 0);
        }

        model.stop();
    }

    #[test]
    fn test_run_data_acquisition() {
        let mut model = create_model();

        let (sender_to_daq, receiver_to_daq) =
            DataAcquisitionProcess::create_sender_and_receiver_to_data_acquisition();

        model.run_data_acquisition(sender_to_daq, receiver_to_daq);

        model.stop();
    }

    #[test]
    fn test_publish_telemetry_connection_off() {
        let model = create_model();

        let telemetry = TelemetryDataAcquisition::new();
        let did_something = model.publish_telemetry(telemetry.get_messages(model.telemetry_digit));

        assert!(!did_something);
    }

    #[test]
    fn test_publish_telemetry_connection_on() {
        let mut model = create_model();
        model.run_processes();

        // Create the clients to connect the servers
        let (_client_command, mut client_telemetry) =
            create_tcp_clients(model.ports["gui_command"], model.ports["gui_telemetry"]);

        sleep(Duration::from_millis(MAX_TIMEOUT));

        wait_for_connection(&mut model);

        // Write the telemetry message
        let telemetry = TelemetryDataAcquisition::new();
        let did_something = model.publish_telemetry(telemetry.get_messages(model.telemetry_digit));

        client_read_and_assert(
            &mut client_telemetry,
            "{\"id\":\"temperature\",\"temperatureItem\":[0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51,0.51]}\r\n",
        );
        client_read_and_assert(
            &mut client_telemetry,
            "{\"id\":\"pressure\",\"pressureItem\":[14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7,14.7]}\r\n",
        );

        assert!(did_something);

        model.stop();
    }

    #[test]
    fn test_publish_events_connection_on() {
        let mut model = create_model();
        model.run_processes();

        // Create the clients to connect the servers
        let (mut client_command, _client_telemetry) =
            create_tcp_clients(model.ports["gui_command"], model.ports["gui_telemetry"]);

        sleep(Duration::from_millis(MAX_TIMEOUT));

        wait_for_connection(&mut model);
        consume_welcome_messages(&model, &mut client_command);

        client_read_and_assert(
            &mut client_command,
            "{\"id\":\"digitalInputsMod4\",\"value\":682}\r\n",
        );

        // Write the event
        let did_something = model.publish_events(vec![
            Event::get_message_digital_outputs(1),
            Event::get_message_digital_outputs(2),
        ]);

        client_read_and_assert(
            &mut client_command,
            "{\"id\":\"digitalOutputs\",\"value\":1}\r\n",
        );
        client_read_and_assert(
            &mut client_command,
            "{\"id\":\"digitalOutputs\",\"value\":2}\r\n",
        );

        assert!(did_something);

        model.stop();
    }

    #[test]
    fn test_run_processes_and_command_data_acquisition_process() {
        let mut model = create_model();
        model.stop_publish_telemetry = true;

        model.run_processes();

        // Create the clients to connect the servers
        let (mut client_command_csc, _client_telemetry_csc) =
            create_tcp_clients(model.ports["csc_command"], model.ports["csc_telemetry"]);

        sleep(Duration::from_millis(MAX_TIMEOUT));

        wait_for_connection(&mut model);
        consume_welcome_messages(&model, &mut client_command_csc);

        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"digitalInputsMod4\",\"value\":682}\r\n",
        );

        // Issue a command to switch the digital outputs
        client_write_and_sleep(
            &mut client_command_csc,
            "{\"id\":\"cmd_switchDigitalOutput\",\"sequence_id\":1,\"parameters\":{\"bit\":1,\"status\":true}}\r\n",
            SLEEP_TIME,
        );

        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"cmd_switchDigitalOutput\",\"sequence_id\":1,\"state\":\"ack\"}\r\n",
        );

        wait_for_command_result_and_reset(&mut model);

        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"digitalInputsMod4\",\"value\":681}\r\n",
        );

        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"digitalOutputs\",\"value\":1}\r\n",
        );

        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"cmd_switchDigitalOutput\",\"sequence_id\":1,\"state\":\"success\"}\r\n",
        );

        model.stop();
    }

    #[test]
    fn test_run_processes_and_step_gui_become_commander() {
        let mut model = create_model();

        model.run_processes();

        // Create the clients to connect the servers
        let (mut client_command_gui, _client_telemetry_gui) =
            create_tcp_clients(model.ports["gui_command"], model.ports["gui_telemetry"]);

        let (mut client_command_csc, _client_telemetry_csc) =
            create_tcp_clients(model.ports["csc_command"], model.ports["csc_telemetry"]);

        sleep(Duration::from_millis(MAX_TIMEOUT));

        wait_for_connection(&mut model);
        consume_welcome_messages(&model, &mut client_command_gui);
        consume_welcome_messages(&model, &mut client_command_csc);

        client_read_and_assert(
            &mut client_command_gui,
            "{\"id\":\"digitalInputsMod4\",\"value\":682}\r\n",
        );
        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"digitalInputsMod4\",\"value\":682}\r\n",
        );

        // Issue a command
        client_write_and_sleep(
            &mut client_command_gui,
            "{\"id\":\"cmd_openPressureControlValve\",\"sequence_id\":1,\"parameters\":{\"idx\":0,\"percentage\":50.0}}\r\n",
            SLEEP_TIME,
        );

        client_read_and_assert(
            &mut client_command_gui,
            "{\"id\":\"cmd_openPressureControlValve\",\"sequence_id\":1,\"state\":\"ack\"}\r\n",
        );

        wait_for_command_result_and_reset(&mut model);

        client_read_and_assert(
            &mut client_command_gui,
            "{\"id\":\"commander\",\"name\":\"gui\"}\r\n",
        );

        client_read_and_assert(
            &mut client_command_gui,
            "{\"id\":\"cmd_openPressureControlValve\",\"sequence_id\":1,\"state\":\"success\"}\r\n",
        );

        // Commander should change to the GUI.
        assert_eq!(model.controller.commander, Commander::GUI);

        // Command from the CSC should fail.
        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"commander\",\"name\":\"gui\"}\r\n",
        );

        client_write_and_sleep(
            &mut client_command_csc,
            "{\"id\":\"cmd_openPressureControlValve\",\"sequence_id\":2,\"parameters\":{\"idx\":0,\"percentage\":50.0}}\r\n",
            SLEEP_TIME,
        );

        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"cmd_openPressureControlValve\",\"sequence_id\":2,\"state\":\"ack\"}\r\n",
        );

        sleep(Duration::from_millis(MAX_TIMEOUT));

        wait_for_command_result_and_reset(&mut model);

        client_read_and_assert(
            &mut client_command_csc,
            "{\"fail_reason\":\"Current commander is GUI instead of CSC.\",\"id\":\"cmd_openPressureControlValve\",\"sequence_id\":2,\"state\":\"fail\"}\r\n",
        );

        // Change the commander to the CSC again.
        client_write_and_sleep(
            &mut client_command_gui,
            "{\"id\":\"cmd_switchCommandSource\",\"sequence_id\":2,\"parameters\":{\"remote\":true}}\r\n",
            SLEEP_TIME,
        );

        client_read_and_assert(
            &mut client_command_gui,
            "{\"id\":\"cmd_switchCommandSource\",\"sequence_id\":2,\"state\":\"ack\"}\r\n",
        );

        wait_for_command_result_and_reset(&mut model);

        client_read_and_assert(
            &mut client_command_gui,
            "{\"id\":\"cmd_switchCommandSource\",\"sequence_id\":2,\"state\":\"success\"}\r\n",
        );

        client_read_and_assert(
            &mut client_command_gui,
            "{\"id\":\"commander\",\"name\":\"csc\"}\r\n",
        );

        // Commander should change to the CSC.
        assert_eq!(model.controller.commander, Commander::CSC);

        // Check the CSC gets this new event
        client_read_and_assert(
            &mut client_command_csc,
            "{\"id\":\"commander\",\"name\":\"csc\"}\r\n",
        );

        model.stop();
    }

    #[test]
    fn test_get_welcome_messages() {
        let model = create_model();

        let messages = Model::get_welcome_messages(&model.controller);

        assert_eq!(messages.len(), 4);
    }

    #[test]
    fn test_get_welcome_messages_multiple_times_connection() {
        let mut model = create_model();
        model.stop_publish_telemetry = true;

        model.run_processes();

        for _ in 0..2 {
            // Create the clients to connect the servers
            let (mut client_command, _client_telemetry) =
                create_tcp_clients(model.ports["gui_command"], model.ports["gui_telemetry"]);

            sleep(Duration::from_millis(MAX_TIMEOUT));

            wait_for_connection(&mut model);

            let messages = consume_welcome_messages(&model, &mut client_command);
            assert_eq!(messages.len(), 4);

            // Client disconnects the servers
            let _ = client_command.shutdown(Shutdown::Both);
            let _ = _client_telemetry.shutdown(Shutdown::Both);

            sleep(Duration::from_millis(MAX_TIMEOUT));
        }

        model.stop();
    }

    #[test]
    fn test_update_status_telemetry_data_acquisition() {
        let mut telemetry = TelemetryDataAcquisition::new();
        telemetry.digital_inputs_mod4 = 1;
        telemetry.digital_inputs_mod7 = 2;
        telemetry.digital_outputs = 3;

        let mut model = create_model();

        let events = model.update_status_telemetry_data_acquisition(&telemetry);

        assert_eq!(
            events,
            vec![
                Event::get_message_digital_inputs_mod4(1),
                Event::get_message_digital_inputs_mod7(2),
                Event::get_message_digital_outputs(3),
            ]
        );
    }
}

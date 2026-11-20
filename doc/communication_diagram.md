# Communication Diagram

The communication among threads is mainly by the [multi-producer, single-consumer FIFO queue communication](https://doc.rust-lang.org/std/sync/mpsc/index.html).
The only exception are:

- stopping of threads
- connection status (see [CommandTelemetryServer](../src/interface/command_telemetry_server.rs))

, which are by the thread-safe reference-counting pointer, [Arc (atomically reference counted)](https://doc.rust-lang.org/std/sync/struct.Arc.html).

The main thread is the [application](../src/application.rs) that runs the `Model.step()`.
The [Model](../src/model.rs) class holds the [Controller](../src/controller.rs) instance to send the commands to the **ControlLoopProcess** (not available yet) and [DataAcquisitionProcess](../src/daq/data_acquisition_process.rs).
The **ControlLoopProcess** can send the commands to the **DataAcquisitionProcess** as well.
The [CommandTelemetryServer](../src/interface/command_telemetry_server.rs) runs the [CommandServer](../src/interface/command_server.rs) and [TelemetryServer](../src/interface/telemetry_server.rs) as the TCP/IP servers.
The **CommandTelemetryServer** runs a monitor loop to check the connection status and feedbacks to the **Model**.
The exchanged data is the [CommandFormat](../src/command/command_format.rs), [Value](https://docs.rs/serde_json/latest/serde_json/value/index.html), or [Telemetry](../src/telemetry/telemetry.rs).
For each [Sender](https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html), there is a related [Receiver](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html) in the class or module.

Since the Mermaid does not support the communication diagram yet, we use the class diagram to mimic the data flow with the association relationship.
Each block is a process in the application and the line's guard is the exchanged data.
Show the communication diagram in the following:

```mermaid
classDiagram

class Model {
    senders_to_tcp
    sender_to_model
    receivers_from_command_telemetry_server
    receivers_from_tcp_command
    receiver_to_model
    Controller: sender_to_daq
}

class ControlLoopProcess {
    sender_to_model
    sender_to_daq
    sender_to_control_loop
    receiver_to_control_loop
}

class DataAcquisitionProcess {
    sender_to_model
    sender_to_daq
    receiver_to_daq
}

class CommandTelemetryServer {
    senders_to_tcp
    sender_from_tcp_command
    sender_from_command_telemetry_server
}

class CommandServer {
    sender_to_tcp
    receiver_to_tcp
    sender_from_tcp
}

class TelemetryServer {
    sender_to_tcp
    receiver_to_tcp
}

Model --> CommandServer : Value
Model --> TelemetryServer : Value
Model --> ControlLoopProcess : CommandFormat
Model --> DataAcquisitionProcess : CommandFormat

ControlLoopProcess --> Model : Telemetry
ControlLoopProcess --> DataAcquisitionProcess : CommandFormat

DataAcquisitionProcess --> Model : Telemetry
CommandTelemetryServer --> Model : Value
CommandServer --> Model : CommandFormat
```

The **CommandServer** receives the command from the clients and sends to the **Model** to process.
The **Model** processes the received command, and sends to the **ControlLoopProcess** and **DataAcquisitionProcess** to do the further processing.
They send the command's result, internal event, and telemetry to the **Model** to forward to the **CommandServer** and **TelemetryServer**.

For the telemetry, the **DataAcquisitionProcess** sends the **TelemetryDataAcquisition** to the **Model** to publish the data.
The **ControlLoopProcess** sends the **TelemetryControlLoop** (not available yet) to **Model** to publish the data.

# Software Architecture

This is a multi-threaded application.

## Communication Protocol

See the [communication protocol](communication_protocol.md) for the communication details between the controller and client (EUI/CSC).

## Command Factory Pattern

The [command factory pattern](../src/command/command_schema.rs) is applied in the system.
For the command parameters, they should be defined in the [command_format.rs](../src/command/command_format.rs) no matter which subsystem will execute the commands.

- [data acquisition](#command-of-the-data-acquisition)
- [controller](#command-of-the-controller)

### Command of the Data Acquisition

The followings are the commands:

- CommandSwitchDigitalOutput
- CommandOpenPressureControlValve
- CommandOpenControlMixingValve
- CommandResetPierFan
- CommandSetChillerTemperature

See [here](../src/command/command_data_acquisition.rs) for the details.

### Command of the Controller

The followings are the commands:

- CommandSwitchCommandSource

See [here](../src/command/command_controller.rs) for the details.

## Communication Diagram

See [here](communication_diagram.md) for the communication diagram.

## Class Diagram

See [here](class_diagram.md) for the class diagram.

## Clients of the Control System

The control system accepts two clients: GUI (graphical user interface) and CSC (commandable SAL component).
When there is the new connection, the new client gets the welcome messages to know the current system status.
The welcome messages are defined in `Model.get_welcome_messages()`.
The new client get the telemetry as well.

Although there are two clients, the controller can have only one commander at a single time.
The CSC is the default commander.
However, any command from GUI will take over the control.
The GUI can issue the `CommandSwitchCommandSource` command to switch the commander back to the CSC.
If the current commander is GUI, any command from CSC will be rejected.

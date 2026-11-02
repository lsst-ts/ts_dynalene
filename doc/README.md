# Software Architecture

This is a multi-threaded application.

## Communication Protocol

See the [communication protocol](communication_protocol.md) for the communication details between the controller and client (EUI/CSC).

## Command Factory Pattern

The [command factory pattern](../src/command/command_schema.rs) is applied in the system.
For the command parameters, they should be defined in the [command_format.rs](../src/command/command_format.rs) no matter which subsystem will execute the commands.

- [data acquisition](#command-of-the-data-acquisition)

### Command of the Data Acquisition

The followings are the commands:

- CommandSwitchDigitalOutput
- CommandOpenPressureControlValve
- CommandOpenControlMixingValve
- CommandResetPierFan
- CommandSetChillerTemperature

See [here](../src/command/command_data_acquisition.rs) for the details.

## Class Diagram

See [here](class_diagram.md) for the class diagram.

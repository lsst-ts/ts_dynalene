# Communication Protocol

This is the communication protocol between the controller and client.

## Header of JSON String

The controller will use the TCP/IP protocol to communicate with the client.
The JSON string packet is used in this process.
To make this work, some header information is required.

### Command from the Client

When the client sends the command, the JSON string should have the following keys:

1. **id**: Specify the name of command that begins with `cmd_`.
2. **sequence_id**: A positive unique ID (>= 0) that the value should increase among different commands (eg. from 0 to N) after the connection between the controller and client.
This item is unrelated to the command name or details.
It is used to track the internet traffic.
The controller can accept any starting **sequence_id** after the connection.

Once the controller receives the command or finishes the command, the related information should be sent to the client (see [Command Acknowledgement](#command-acknowledgement)).

## Command Acknowledgement

The controller should acknowledge the client actively for the command status.
To support this, the following header information is needed:

1. **id**: Echo the received command name.
2. **state**: Specify the command state.
The value should be in `ack`, `noack`, `success`, or `fail`.
The `ack` or `noack` notifies whether a command is received or might have been lost.
The `success` or `fail` gives the result after executing the command.
3. **fail_reason** (optional): Failed reason if the **state** is `noack` or `fail`.
The `ack` and `success` do not have this field.
4. **sequence_id**: The unique ID from the client.

The controller should acknowledge the client actively for the command status with `ack` if it is in the list of registered commands by controller.

If the received command is not in the list of registered commands, the `noack` will be sent such as:

```json
{
    "id": "cmd_move",
    "state": "noack",
    "fail_reason": "Not registered command.",
    "sequence_id": 2
}
```

In addition, if a command is suspected to be lost, the `noack` will be used as well.
In this condition, the **id** will be an empty string.
For example, if the controller receives a command with **sequence_id: 1** followed by **sequence_id: 3** in the same connection, the controller should send the `noack` as the following and vice versa:

```json
{
    "id": "",
    "state": "noack",
    "fail_reason": "Command lost.",
    "sequence_id": 2
}
```

## Details of the Parameters in JSON String

If there are no parameters in a command, the command packet from client looks like the following:

```json
{
    "id": "cmd_move",
    "sequence_id": 1
}
```

But if there are the parameters, a structured packet should be applied with a **parameters** field.
For example, the client might issue a `move` command with parameters `x`, `y`, and `z` as the following:

```json
{
    "id": "cmd_move",
    "sequence_id": 1,
    "parameters": {
        "x": 0.1,
        "y": 0.2,
        "z": 0.3
    }
}
```

The client might issue multiple commands in a short time, but the controller should only execute a single command in a single time and reply the result.

Take the above command as an example, the acknowledgement from controller at each time of receiving the command will be:

```json
{
    "id": "cmd_move",
    "state": "ack",
    "sequence_id": 1
}
```

If this command succeeds after the execution by controller, the following message will be issued to the client:

```json
{
    "id": "cmd_move",
    "state": "success",
    "sequence_id": 1
}
```

Otherwise, it would be:

```json
{
    "id": "cmd_move",
    "state": "fail",
    "fail_reason": "Drive is disabled.",
    "sequence_id": 1
}
```

### Message from the Controller

The controller should send the message to the client as an event when the internal state changes.
The message should contain the **id** to specify the message's name followed by the additional (optional) flat fields for the details.
For example, a message could be:

```json
{
    "id": "inPosition",
    "axis": "y"
}
```

or:

```json
{
    "id": "inPosition"
}
```

Some message can be sent by the controller in a fixed frequency as the telemetry as well.
The client should decide the received message belongs to an event or telemetry in the xml/avro level.

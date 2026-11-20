# Deployment in cRIO controller

The Dynalene controller cRIO code is compiled and deployed in the cRIO controller.

## Initialization File

Copy the [initialization file](../script/dynalene) to the `/etc/init.d/` directory.

## Use the Control System

To start the control system, do:

```bash
/etc/init.d/dynalene start
```

To stop the control system, do:

```bash
/etc/init.d/dynalene stop
```

To get the status of the control system, do:

```bash
/etc/init.d/dynalene status
```

More usage, do:

```bash
/etc/init.d/dynalene -h
```

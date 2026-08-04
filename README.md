# Dynalene Controller

This is the dynalene control system.

## Sensors

- Temperature hub: This is a home-made device by Oliver.
In the future, we will replace it with a commercial product.
- Pressure transducer: [PX459-100A485-I](https://www.dwyeromega.com/en-us/configurable-high-accuracy-pressure-transducers/PX409-Series/p/PX459-100A485-I).
You can download the **PX51-PXM51-SERIES_software** in the link to get the ModBus protocol document.

## Development Environment

You can develop the code under the Windows, Mac, and Linux.

## Log Data

The logging files are in the `log/` directory.

## Code Format

To format the code, do:

```bash
.githooks/pre-commit
```

## Unit Test

Each module and function have the related unit tests.
Since the CI test is needed, you can use the [cargo-nextest](https://crates.io/crates/cargo-nextest) instead of the built-in test framework.
Do the following to run all tests:

```bash
cargo nextest run
```

To test a single module, do:

```bash
cargo nextest run --lib $module_name
```

To generate the `junit.xml` (ouput path is `target/nextest/ci/junit.xml`), do:

```bash
cargo nextest run --profile ci
```

## Software Architecture

See [here](doc/README.md) for the design of software.

## UML Diagrams

The UML diagrams are used to detail the system design for each subsystem in the `doc/` directory.
The GitHub supports the [Mermaid](https://github.com/mermaid-js/mermaid) natively.
You can use the [online editor](https://mermaid.live) to edit them.

## Version History

See [here](doc/version_history.md) for the version history.

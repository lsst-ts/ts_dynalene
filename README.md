# Dynalene Controller

This is the dynalene control system as the successor of [dynalene_system](https://github.com/lsst-ts/dynalene_system) LabVIEW project.

## Controller Hardwares

The cRIO is the NI cRIO-9038.
The modules are:

- Slot 1: NI 9870
- Slot 2: NI 9871
- Slot 3: NI 9871
- Slot 4: NI 9425
- Slot 5: NI 9207
- Slot 6: NI 9476
- Slot 7: NI 9425
- Slot 8: NI 9264

## Sensors and Instruments

- Temperature hub: This is a home-made device by Oliver Weicha.
In the future, we will replace it with a commercial product.
For the thermal probes, the [Omega PR-21 series](https://www.dwyeromega.com/en-ca/rtd-probes-with-mounting-threads-and-m12-connectors/p/PR-21) are used.
- Pressure transducer: [PX459-100A485-I](https://www.dwyeromega.com/en-us/configurable-high-accuracy-pressure-transducers/PX409-Series/p/PX459-100A485-I).
You can download the **PX51-PXM51-SERIES_software** in the link to get the ModBus protocol document.
- Flowmeter: [FDT-40 Series](https://in.omega.com/pptst/FDT-40.html) with the transducers: FDT-46 (2") / FDT-45 (1.5") / FDT-42 (3/4").
You can download the [user guide](https://assets.omega.com/manuals/M5221.pdf) to get the ModBus protocol.
- Power grid: [PM5000 Power Meter Series](https://www.se.com/us/en/product/METSEPM5560/power-meter-powerlogic-pm5560-2-ethernet-up-to-63th-harmonic-11mb-4di-2do-52-alarms/?selectedNodeId=12146169702) are used.
You can download the [user guide](https://productinfo.se.com/pm5500/595e2aa946e0fb0001f715da/PM5500%20user%20manual/English/HRB1684301-18.pdf) and the [Modbus register list](https://se.my.site.com/ckmContent/sfc/servlet.shepherd/document/download/069Kj00000TH2DFIA1).
- Pier fan: [Ebmpapst EC axial fans - HyBlade](https://www.ebmpapst.com/content/dam/ebm-papst/media/catalogs/products/Catalog_Axialfans_EC-HyBlade_en.pdf) is used.
You need to [register the account](https://www.ebmpapst.com/us/en/support/downloads/modbus-ebmbus.html) and request the Modbus document to get the Modbus protocol.
- Recirculation pump: [Grundfos CRE 10-5 K-FJ-A-E-HQQE](https://product-selection.grundfos.com/products/cr-cre-cri-crie-crn-crne-crt-crte/cre/cre-10-5-99241450?pumpsystemid=3051387793&tab=variant-curves) is used.
You can get the [Modbus document](https://api.grundfos.com/literature/Grundfosliterature-6012947.pdf) to know the protocol details.

## Development Environment

You can develop the code under the Windows, Mac, and Linux.

## Configuration Files

See the [config/](config) directory for the configuration files:

- [parameters_app.yaml](config/parameters_app.yaml) is the configuration of application.

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

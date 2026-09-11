# ORBI Firmware

Production embedded firmware for the **ORBI Sensor Intelligence
Platform**.

> **Current milestone:** Concurrent live telemetry and background SD
> replay implemented and physically validated on ORBI reference
> hardware. Firmware milestone commit: `45b5f58`.

ORBI Firmware is a modular, production-oriented Rust (`no_std`) firmware
designed for intelligent telemetry devices deployed across fleets,
construction equipment, generators, stationary fuel tanks, energy
infrastructure, and future industrial monitoring applications.

Unlike traditional GPS tracker firmware, ORBI Firmware is designed as a
**sensor platform**. GPS is only one sensor within a larger architecture
capable of integrating multiple telemetry sources while maintaining a
unified telemetry pipeline.

Current hardware targets include the ESP32 platform with the SIMCom
A7670E LTE/GNSS modem, providing:

-   GNSS positioning
-   LTE communication
-   SD card persistence
-   Offline telemetry buffering
-   Automatic replay after connectivity restoration

The firmware is intentionally modular to support future hardware
revisions without changing the higher-level telemetry pipeline.

Current firmware capabilities include:

-   GNSS telemetry
-   Raw MPU6050 IMU telemetry
    -   Accelerometer (X, Y, Z)
    -   Gyroscope (X, Y, Z)
    -   IMU temperature
-   Verified RS485 / Modbus communication
-   Verified KUM ultrasonic fuel sensor communication
-   Raw KUM Modbus measurement acquisition
-   SensorSnapshot abstraction layer
-   Measurement-first telemetry model
-   Raw fuel telemetry generation
-   KUM telemetry integration into the unified telemetry pipeline
-   Raw fuel telemetry persistence to the SD queue
-   Raw fuel telemetry uploads
-   LTE backend communication
-   HTTP telemetry uploads
-   Persistent SD card queue
-   FIFO historical telemetry buffering
-   Concurrent background replay of offline telemetry
-   Foreground live telemetry during backlog recovery
-   Dedicated modem-owner task
-   Dedicated storage-owner task
-   Runtime queue cleanup
-   Upload acknowledgement tracking
-   Bounded multi-record replay batches
-   At-least-once delivery semantics
-   Motion-aware reporting scheduler
-   Persistent device identity stored in internal flash
-   Backend inventory and provisioning compatibility
-   Shared board-level peripheral power management
-   Verified SD card initialization during both USB-powered and
    LiPo-only operation
-   Deterministic hardware initialization sequence
-   Modular board abstraction separating hardware bring-up from device
    drivers

The long-term objective is to provide a reusable embedded platform
capable of supporting multiple ORBI hardware profiles while maintaining
a consistent backend interface across all deployments.

# ORBI Firmware Architecture

## Core Design Principle

ORBI Firmware follows a simple architectural principle:

> **Firmware acquires reality. The ORBI Platform interprets reality.**

The firmware is responsible only for interacting with physical hardware,
collecting sensor measurements, timestamping telemetry, buffering data
during communication failures, and reliably transmitting telemetry to
the ORBI Platform.

The firmware intentionally does **not** perform operational intelligence
such as:

-   Fuel theft detection
-   Fuel refill detection
-   Fuel leak detection
-   Movement classification
-   Driver behaviour analysis
-   Operational alert generation

The ORBI backend is responsible for:

-   Calibration
-   Measurement normalization
-   Filtering
-   Threshold profiles
-   Sensor fusion
-   Movement classification
-   Fuel theft detection
-   Fuel refill detection
-   Fuel leak detection
-   Driver behaviour analysis
-   Operational alert generation
-   Investigation intelligence
-   Replay intelligence
-   Map intelligence
-   Analytics intelligence
-   Future machine learning and AI capabilities

The firmware also does not derive IMU-based operational states such as:

-   vibration_score
-   motion_detected
-   movement_confidence
-   parked
-   idle
-   moving

Instead, it transmits raw IMU measurements and allows the backend to
interpret motion behaviour.

Those responsibilities belong to the ORBI Sensor Intelligence Platform.

------------------------------------------------------------------------

## Measurement-First Architecture

The firmware produces measurements, not intelligence.

Every telemetry cycle represents a snapshot of the physical sensors
installed on the device.

Telemetry may contain measurements from:

-   Position (GNSS)
-   Fuel
-   IMU
    -   Accelerometer (X, Y, Z)
    -   Gyroscope (X, Y, Z)
    -   Temperature
-   Power
-   Diagnostics

The firmware never embeds business decisions into telemetry.

Instead, the backend combines measurements from multiple sensor domains
to generate operational intelligence.

------------------------------------------------------------------------

## Firmware Processing Pipeline

Physical Sensors │ ▼ Hardware Drivers │ ▼ Raw Measurements │ ▼
SensorSnapshot │ ▼ TelemetryRecord │ ▼ Persistent Queue │ ▼ Publisher │
▼ ORBI Platform

This architecture allows new sensors to be integrated without
redesigning storage, replay, networking, or scheduling.

## Live Telemetry and Replay Concurrency Architecture

Physical hardware testing identified an important production-runtime
requirement: replaying historical telemetry from the SD queue must not
block or starve acquisition, persistence, or delivery of current
telemetry.

This requirement has now been implemented and physically validated using
the Embassy-based `no_std` runtime.

The firmware separates long-lived runtime responsibilities so that
sensor acquisition, persistent storage, live telemetry publishing, and
historical replay can progress cooperatively.

``` text
                         ORBI Firmware
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
 Sensor Acquisition      Storage Owner         Modem Owner
        │                     │                     │
        ▼                     │          ┌──────────┴──────────┐
 Current Telemetry ───────────┼─────────►│                     │
                              │          ▼                     ▼
                              │     Live Telemetry        Replay Batches
                              │        Priority             Background
                              │          │                     │
                              └──────────┴──────────┬──────────┘
                                                   ▼
                                                LTE Modem
```

The runtime policy is:

``` text
LIVE telemetry   = foreground priority
REPLAY telemetry = incremental background work
SD queue         = durable source of pending delivery
```

### Modem Ownership

The LTE modem has a single runtime owner. Other firmware tasks do not
independently contend for the modem; foreground operations and replay
operations submit work through Embassy channels and signals.

Foreground work is checked before replay work at modem transaction
boundaries. This gives current telemetry priority without attempting to
interrupt an already active modem transaction.

### Storage Ownership

Runtime SD-card operations are coordinated through a dedicated storage
owner. The storage owner serializes access to live telemetry
persistence, live acknowledgements, GNSS diagnostics, replay
preparation, and replay finalization.

Foreground persistence and acknowledgement work is handled before
background replay work.

### Direct Live Telemetry

Every current telemetry record is first written to persistent storage.
After persistence succeeds, the newest live record is submitted directly
to the modem owner rather than waiting behind the historical queue.

When the backend accepts the record:

1.  An acknowledgement is persisted for that exact device identity and
    timestamp.
2.  The physical telemetry record remains in the queue if older records
    are still ahead of it.
3.  When replay later reaches the already acknowledged record, it can
    remove the record without uploading it again.

This preserves persistent-first and at-least-once delivery while
allowing the backend to receive the device's current operational state
during backlog recovery.

If SD persistence is unavailable, the current implementation retains a
direct-upload fallback so current telemetry can still be attempted. This
fallback is an explicitly retained runtime behaviour and does not
replace the normal persistent-first path.

### Background Replay

Historical telemetry is drained incrementally in bounded batches. The
current replay batch size is four records.

Replay does not impose a logical maximum on the total historical
backlog. Instead, each individual batch is bounded so foreground work
receives regular opportunities to progress.

After a successful replay batch, acknowledgements are persisted and
acknowledged queue records are removed before replay yields and
continues with subsequent work.

A transient replay HTTP failure leaves the pending records unchanged.
The replay path waits before retrying rather than discarding the
historical records.

### Physical Validation

The concurrent architecture has been validated on physical ORBI hardware
with an accumulated SD-card telemetry backlog.

Testing confirmed that while historical replay was active:

-   GNSS acquisition continued;
-   sensor acquisition continued;
-   new live telemetry continued entering persistent storage;
-   current telemetry continued reaching the backend;
-   historical telemetry continued draining in bounded batches;
-   replay resumed after foreground live communication;
-   acknowledgements were persisted before queue removal;
-   previously acknowledged records were safely recovered;
-   heartbeat communication continued;
-   original telemetry timestamps and device identities were preserved.

Observed replay progress advanced through multiple batches while live
telemetry continued operating, confirming that historical recovery no
longer acts as a blocking `replay-entire-backlog-before-live-operation`
mode.

The firmware therefore preserves its persistent-first and at-least-once
delivery model while allowing current telemetry and historical recovery
to coexist.

A remaining implementation detail is that the current replay task can
finish after the queue becomes completely empty. A telemetry record that
subsequently fails direct live upload may therefore remain queued until
replay is started again, including after a later boot. Persistent
storage prevents loss, but continuous post-drain replay polling remains
a future hardening item.

## Project Goals

The ORBI Firmware project is designed around a single long-term
principle:

> **Build one firmware platform capable of supporting many intelligent
> sensing applications.**

Rather than developing separate firmware for every product, ORBI
Firmware provides a common embedded foundation that can be configured
for different hardware profiles while exposing a consistent telemetry
interface to the backend platform.

The firmware is designed to support deployments ranging from simple GPS
trackers to complex multi-sensor intelligence devices.

Current and planned sensor support includes:

-   GNSS positioning
-   Fuel level monitoring (RS485 / Modbus)
-   MPU6050 IMU (Accelerometer, Gyroscope, Temperature)
-   Ignition monitoring
-   Digital inputs and outputs
-   Relay/Kill switch control
-   Future CAN Bus integration
-   Future LoRa sensor gateways
-   Additional industrial sensors through a modular driver architecture

The firmware intentionally separates hardware interaction from telemetry
generation.

Sensor drivers are responsible only for communicating with physical
devices and acquiring sensor measurements.

The telemetry layer packages sensor measurements into a unified
telemetry payload shared with the ORBI Platform.

The telemetry contract represents measurements only.

Operational intelligence---including movement classification, fuel event
detection, driver behaviour analysis, and alert generation---is produced
by backend sensor services and the Intelligence Engine.

This modular architecture allows the same firmware foundation to support
multiple ORBI product variants, including:

-   GPS-only asset trackers
-   Fuel Intelligence devices
-   Fleet Intelligence devices
-   Generator monitoring systems
-   Stationary storage tank monitoring
-   Industrial and energy monitoring solutions

The result is a scalable embedded platform where new hardware
capabilities are added through modular sensor adapters rather than
separate firmware projects.

## System Architecture

ORBI Firmware follows a layered architecture that separates hardware
interaction, telemetry generation, persistent storage, networking, and
scheduling into independent modules.

``` text
                         ORBI Firmware

+-----------------------------------------------------------+
|                       Application Layer                   |
|-----------------------------------------------------------|
| Scheduler | Telemetry Builder | Replay | Heartbeat        |
+-----------------------------------------------------------+
                            │
                            ▼
+-----------------------------------------------------------+
|                     Telemetry Layer                       |
|-----------------------------------------------------------|
| TelemetryRecord | Payload Builder | Device Identity       |
+-----------------------------------------------------------+
                            │
                ┌───────────┴───────────┐
                ▼                       ▼
+---------------------------+   +---------------------------+
|      Storage Layer        |   |      Network Layer        |
|---------------------------|   |---------------------------|
| SD Queue                  |   | LTE HTTP Client           |
| ACK Log                   |   | Backend Communication     |
| Replay Queue              |   | Heartbeats               |
+---------------------------+   +---------------------------+
                ▲                       ▲
                └───────────┬───────────┘
                            │
                            ▼
+-----------------------------------------------------------+
|                     Hardware Drivers                      |
|-----------------------------------------------------------|
| GNSS | LTE Modem | SD Card | I²C | UART | GPIO | SPI      |
+-----------------------------------------------------------+
                            │
                            ▼
+-----------------------------------------------------------+
|                        ESP32 Hardware                     |
+-----------------------------------------------------------+
```

### Architectural Principles

The firmware is intentionally divided into independent layers.

### Hardware Drivers

The driver layer communicates directly with physical peripherals and
sensors.

Examples include:

-   GNSS receiver
-   LTE modem
-   SD card
-   RS485 interfaces
-   I²C devices
-   GPIO peripherals

Drivers should **never contain business logic**. Their responsibility is
simply to acquire or transmit data.

------------------------------------------------------------------------

### Telemetry Layer

The telemetry layer is built around a measurement-first architecture.

Individual hardware drivers contribute measurements to a SensorSnapshot,
which represents one complete physical sampling cycle.

The telemetry subsystem transforms the SensorSnapshot into a
TelemetryRecord for storage, replay, and backend transmission.

This abstraction allows new sensor technologies to be integrated without
changing the telemetry publishing interface.

The telemetry layer packages hardware measurements into a unified
telemetry payload for transmission to the ORBI Platform.

Every telemetry cycle produces a single `TelemetryRecord`, regardless of
which sensors are installed on the device.

This keeps the backend interface stable while allowing hardware profiles
to evolve independently.

------------------------------------------------------------------------

### Storage Layer

The storage layer provides persistent buffering using the SD card.

Responsibilities include:

-   Queueing telemetry before transmission
-   Maintaining upload acknowledgements
-   Offline persistence
-   serialized runtime SD-card ownership
-   replay batch preparation and finalization
-   queue cleanup

Current telemetry includes:

-   GPS position
-   Speed
-   Heading
-   Raw IMU measurements
    -   Accelerometer (X, Y, Z)
    -   Gyroscope (X, Y, Z)
    -   IMU temperature

The telemetry layer intentionally publishes measurements only and
performs no operational interpretation.

The storage layer guarantees that telemetry is never discarded simply
because connectivity is temporarily unavailable.

------------------------------------------------------------------------

### Network Layer

The network layer manages all communication with the ORBI backend.

Responsibilities include:

-   LTE connectivity
-   HTTP communication
-   Heartbeats
-   Backend uploads
-   Network diagnostics
-   foreground live telemetry requests
-   background replay requests
-   coordinated single-owner modem access

The networking layer is intentionally isolated from sensor drivers so
communication protocols can evolve independently.

Runtime modem access is coordinated by `network/modem_owner.rs`.
Foreground GNSS, diagnostics, heartbeat, and live-telemetry work is
arbitrated against background replay at modem transaction boundaries.

The communication architecture is intentionally divided into two
independent layers.

HTTP / GNSS Protocol Layer │ ▼ Generic Modem Transport │ ▼ UART Driver

The modem transport layer is responsible only for reliable UART
communication with the modem.

Its responsibilities are intentionally limited to:

-   Sending AT commands
-   Detecting when modem response data is available
-   Reading available UART data

The transport layer does **not** understand HTTP or GNSS behaviour.

Protocol-specific logic remains within the higher-level modules.

For example:

-   The HTTP module is responsible for waiting for asynchronous
    `+HTTPACTION` responses, validating HTTP status codes, and
    determining upload success.
-   The GNSS module is responsible for parsing `+CGNSSINFO` responses
    into normalized position measurements.

This separation keeps the modem driver reusable while allowing
individual protocols to evolve independently.

------------------------------------------------------------------------

### Scheduler Layer

The scheduler determines **when** telemetry should be generated.

Current scheduling decisions are based on:

-   Vehicle movement
-   Device activity
-   Heartbeat timing
-   Replay requirements

Future versions will allow these policies to be configured remotely from
the backend.

## Repository Structure

The firmware is organised into small, independent modules. Each module
has a single responsibility, making the codebase easier to understand,
test, and extend.

``` text
src/
├── board/
├── device/
├── drivers/
├── network/
├── scheduler/
├── storage/
├── telemetry/
└── main.rs
```

### `board/`

Contains board-specific hardware initialization and platform bring-up.

Responsibilities include:

-   ESP32 peripheral initialization
-   Pin assignments
-   Board configuration
-   Clock setup
-   Hardware abstraction for the target board
-   Shared peripheral power sequencing
-   Runtime board initialization order
-   Board-level GPIO ownership

The board layer is responsible for preparing the hardware platform
before any peripheral drivers are initialized.

This includes enabling shared power rails, configuring board-level GPIO
states, and ensuring that hardware dependencies are satisfied before
storage, networking, or sensor drivers begin operation.

For the current ORBI reference hardware (LilyGO T-A7670), the board
layer owns the shared peripheral power rail controlled by GPIO12.

The shared power rail supplies both:

-   MicroSD storage
-   SIMCom A7670E modem

During startup, the board enables this shared rail before SD card
initialization to guarantee reliable storage operation during both
USB-powered and LiPo-only deployments.

This separation keeps board-specific startup behaviour isolated from
individual hardware drivers and provides a clean foundation for
supporting future ORBI hardware revisions.

------------------------------------------------------------------------

### `device/`

Contains the identity and configuration of the physical ORBI device.

Responsibilities include:

-   Device identity
-   Device code
-   Firmware version
-   Hardware profile
-   Future manufacturing metadata

The device module represents **who the device is**, independent of the
sensors attached to it.

------------------------------------------------------------------------

### `drivers/`

Contains low-level hardware drivers.

Current drivers include:

-   LTE modem
-   GNSS
-   SD card
-   MPU6050 IMU
-   RS485 / Modbus KUM fuel sensor

Future drivers will include:

-   Ignition input
-   Digital I/O
-   CAN Bus
-   LoRa interfaces

Drivers communicate directly with hardware and should never contain
business or application logic.

------------------------------------------------------------------------

### `network/`

Handles backend communication and runtime ownership of the LTE/GNSS
modem.

Responsibilities include:

-   LTE connectivity
-   HTTP client
-   Heartbeats
-   Backend uploads
-   Network diagnostics
-   `modem_owner` foreground/replay arbitration
-   Embassy channels and signals used to coordinate modem work

Networking is intentionally isolated from the telemetry and storage
layers so communication protocols can evolve without affecting sensor
integration.

------------------------------------------------------------------------

### `scheduler/`

Determines when telemetry should be generated.

Current scheduling decisions consider:

-   Vehicle movement
-   Idle state
-   Parked state
-   Heartbeat timing

Future versions will support backend-configurable reporting policies.

------------------------------------------------------------------------

### `storage/`

Provides persistent storage using the SD card and coordinated runtime
ownership of the storage peripheral.

Responsibilities include:

-   Telemetry queue
-   Upload acknowledgements
-   owned telemetry/ACK records for task handoff
-   dedicated storage-owner task
-   foreground live persistence
-   replay batch preparation and finalization
-   queue cleanup
-   persistent buffering during network outages

The storage owner prevents independent runtime tasks from contending
directly for the SD card.

------------------------------------------------------------------------

### `telemetry/`

Builds the telemetry exchanged with the backend.

Responsibilities include:

-   Measurement packaging
-   SensorSnapshot construction
-   TelemetryRecord generation
-   Payload generation
-   direct live publishing
-   replay payload generation
-   backend telemetry serialization

The telemetry layer is intentionally measurement-oriented and remains
independent of operational intelligence.

------------------------------------------------------------------------

### `main.rs`

The firmware entry point.

Responsibilities include:

-   Hardware initialization
-   Driver startup
-   Runtime sequencing
-   Scheduler execution
-   Main telemetry loop

The `main` module coordinates the firmware but delegates implementation
details to the individual modules.

## Hardware Platform

The current ORBI Firmware reference platform is built around the ESP32
and the SIMCom A7670E LTE/GNSS modem.

The firmware has been designed so that future hardware revisions can
reuse the same software architecture with minimal changes.

### Primary Hardware

  Component           Description
  ------------------- --------------------------
  MCU                 ESP32
  Cellular Modem      SIMCom A7670E
  GNSS                Integrated within A7670E
  Storage             MicroSD Card
  LTE Connectivity    4G LTE
  Positioning         GNSS (GPS)
  Firmware Language   Rust (`no_std`)

------------------------------------------------------------------------

## Current Hardware Interfaces

The firmware is organised around reusable hardware interfaces rather
than application-specific code.

### UART

Used for communication with:

-   SIMCom A7670E LTE modem
-   GNSS interface (through the modem)

------------------------------------------------------------------------

### SPI

Used for:

-   MicroSD card communication

The SD card provides persistent telemetry storage, allowing the firmware
to continue operating during periods of network loss.

------------------------------------------------------------------------

### GPIO

Current GPIO usage includes:

-   Modem power control
-   Modem reset
-   Status signals

Future GPIO assignments will support:

-   Ignition detection
-   Digital inputs
-   Relay outputs
-   External alarms

------------------------------------------------------------------------

### I²C

The I²C interface currently supports the MPU6050 IMU.

Current implementation:

-   MPU6050 Accelerometer
-   MPU6050 Gyroscope
-   MPU6050 Temperature

Future I²C devices may include:

-   Environmental sensors
-   Additional motion sensors
-   Industrial monitoring sensors

------------------------------------------------------------------------

### RS485 / Modbus

The primary industrial sensor interface.

The RS485 communication layer has now been successfully validated on the
ORBI reference hardware using a MAX485 transceiver and a KUM ultrasonic
fuel sensor.

Verified capabilities include:

-   UART2 communication
-   MAX485 RS485 transceiver integration
-   Modbus RTU request transmission
-   Successful Modbus response reception
-   Raw KUM measurement acquisition

Future work includes:

-   Generic Modbus abstraction
-   Additional industrial sensors
-   Multi-vendor register profiles
-   Generic sensor adapters

The firmware is intentionally being developed with a generic Modbus
architecture rather than a sensor-specific implementation.

------------------------------------------------------------------------

## Device Profiles

The hardware architecture is designed to support multiple ORBI product
variants without changing the overall firmware structure.

Examples include:

  Product                      Sensors
  ---------------------------- --------------------------
  ORBI GPS Lite                GPS + IMU
  ORBI GPS Control Kit         GPS + IMU + Relay
  ORBI Fuel Intelligence Kit   Fuel + GPS + IMU
  ORBI Full Intelligence Kit   Fuel + GPS + IMU + Relay

Each hardware profile shares the same telemetry pipeline while enabling
only the drivers required for that deployment.

# Development Environment

ORBI Firmware is developed using the Rust embedded ecosystem targeting
the ESP32 platform.

The project uses a `no_std` architecture together with the ESP HAL
provided by Espressif.

## Development Environment

Current development platform:

  Component          Version
  ------------------ -------------------------
  Operating System   Ubuntu (WSL2)
  Rust               Stable
  Target             `xtensa-esp32-none-elf`
  Framework          esp-hal
  Build System       Cargo
  Flash Tool         espflash

------------------------------------------------------------------------

## Required Software

Install the following tools before building the firmware.

### Rust

``` bash
rustup update
```

------------------------------------------------------------------------

### ESP Toolchain

``` bash
espup install
```

------------------------------------------------------------------------

### Export Environment

Before building, load the ESP environment.

``` bash
source ~/export-esp.sh
```

This configures:

-   Rust target
-   Xtensa toolchain
-   Linker
-   ESP build environment

------------------------------------------------------------------------

## Build Firmware

Build the production firmware:

cd \~/projects/fuel-intelligence-platform/orbi-firmware

source \~/export-esp.sh

cargo build --release

The firmware image will be generated at:

target/xtensa-esp32-none-elf/release/orbi-firmware

------------------------------------------------------------------------

## Flash Firmware

Flash the firmware to the ESP32:

espflash flash\
--monitor\
--ignore-app-descriptor\
--port COM5\
"\\wsl.localhost`\Ubuntu`{=tex}`\home`{=tex}`\william`{=tex}`\projects`{=tex}`\fuel`{=tex}-intelligence-platform`\orbi`{=tex}-firmware`\target`{=tex}`\xtensa`{=tex}-esp32-none-elf`\release`{=tex}`\orbi`{=tex}-firmware"

------------------------------------------------------------------------

## Serial Monitor

To monitor serial output without reflashing:

``` bash
espflash monitor --port COM5
```

------------------------------------------------------------------------

## Expected Boot Sequence

A successful firmware startup follows a deterministic initialization
sequence before ownership of shared runtime resources is transferred to
long-lived Embassy tasks.

``` text
ESP32 Boot
│
▼
Board Initialization
│
▼
Enable Shared Peripheral Power (GPIO12)
│
▼
Load Persistent Device Identity
│
▼
Start Storage Owner
│
▼
Initialize LTE Modem
│
▼
Initialize GNSS / Network Readiness
│
▼
Start Modem Owner
│
▼
Start Background Replay
│
▼
Enter Live Telemetry Operation
```

The storage owner owns runtime SD-card access, while the modem owner
owns runtime access to the SIMCom A7670E modem. This prevents
independent tasks from contending directly for shared hardware.

Historical replay is no longer a blocking
`replay-entire-backlog-before-live-operation` startup stage. When replay
is started, it progresses cooperatively in the background while live
sensor acquisition and persistence continue.

The shared peripheral power rail is still enabled before storage and
modem initialization, which remains required for reliable operation on
the LilyGO T-A7670 reference platform.

# Device Provisioning Lifecycle

Every ORBI device follows a controlled lifecycle before it begins
transmitting telemetry to the ORBI backend.

The firmware is designed to operate as part of the wider ORBI
provisioning platform rather than as a standalone GPS tracker.

## Manufacturing Workflow

Every physical device progresses through the following stages:

Manufactured │ ▼ Firmware Programmed │ ▼ Hardware Tested │ ▼ Registered
in Inventory │ ▼ Provisioned to Customer │ ▼ Activated │ ▼ Operational

Each stage ensures that the device is correctly identified, tested, and
associated with the appropriate customer assets before live telemetry is
accepted by the backend.

------------------------------------------------------------------------

## Device Identity

Every firmware build includes a device identity that uniquely identifies
the physical hardware.

The current firmware uses the `DEVICE_IDENTITY` definition located
within the `device` module.

Typical information includes:

-   Device Code
-   Firmware Version
-   Hardware Profile
-   Device Model

The device identity is included in every telemetry payload transmitted
to the backend.

------------------------------------------------------------------------

## Backend Provisioning

Before telemetry is accepted, the device must exist within the ORBI
Platform.

Provisioning associates the physical device with:

-   Organization
-   Customer
-   Asset
-   Hardware Profile
-   Sensor Configuration

This allows identical firmware builds to operate across multiple
deployments while the backend determines which features and sensors are
enabled for each device.

------------------------------------------------------------------------

## Hardware Profiles

Hardware profiles define the capabilities of a device without requiring
separate firmware projects.

Examples include:

  Hardware Profile     Enabled Features
  -------------------- -----------------------------
  GPS Tracker          GNSS
  Fuel Intelligence    GNSS + Fuel
  Fleet Intelligence   GNSS + Fuel + Vibration
  Generator Monitor    Fuel + Digital Inputs
  Industrial Monitor   Custom Sensor Configuration

The firmware remains modular, while the backend controls which
capabilities are active for each deployment.

------------------------------------------------------------------------

## Future Remote Configuration

Future firmware versions will support backend-driven configuration,
allowing devices to receive operational settings remotely.

Planned remotely configurable parameters include:

-   Reporting intervals
-   Enabled sensors
-   Sensor calibration
-   Alert thresholds
-   Network behaviour
-   Power management policies

This approach minimizes firmware changes while maximizing deployment
flexibility.

# Runtime Architecture

Once powered on, ORBI Firmware follows a deterministic boot sequence and
then transitions into a cooperative Embassy runtime.

The runtime combines:

-   a main sensor/reporting loop;
-   a dedicated storage owner task;
-   a dedicated modem owner task;
-   a background replay task;
-   a heartbeat task.

Shared hardware is owned by the appropriate long-lived task rather than
accessed concurrently by unrelated subsystems.

``` text
Power On
│
▼
Board + Shared Power Initialization
│
▼
Load Persistent Device Identity
│
▼
Start Storage Owner
│
▼
Initialize Modem / GNSS / LTE
│
▼
Start Modem Owner
│
├──────────────► Foreground modem requests
│
└──────────────► Background replay requests
│
▼
Live Sensor + Reporting Runtime
```

The modem owner checks foreground work before accepting the next replay
transaction. An already active modem transaction is allowed to complete;
priority is enforced at transaction boundaries.

The storage owner similarly gives live persistence and acknowledgement
work priority over replay preparation/finalization.

# Operational Phase

Once initialization has completed, live operation and historical
recovery can progress cooperatively.

A live telemetry cycle follows this path:

``` text
Read GNSS
│
▼
Read IMU
│
▼
Read Fuel Sensor (if installed)
│
▼
Build SensorSnapshot
│
▼
Build TelemetryRecord
│
▼
Request Storage Owner to Append to ORBIQ.LOG
│
▼
Submit Current Record to Modem Owner
│
▼
HTTP 2xx?
├── NO  → record remains pending for replay
└── YES → persist ACK for exact device ID + timestamp
```

In parallel with successive live cycles, the replay task requests
bounded historical batches from the storage owner and submits them to
the modem owner as background work.

``` text
Historical Queue
│
▼
Prepare Bounded Replay Batch
│
▼
Submit Background Modem Request
│
▼
HTTP 2xx?
├── NO  → queue unchanged; wait and retry
└── YES → persist ACKs
           │
           ▼
        remove acknowledged queue records
           │
           ▼
        yield / continue replay
```

This architecture prevents a large historical backlog from monopolising
normal live operation.

## Runtime Principles

The runtime has been designed around several core principles.

### Persistent First

Telemetry is always written to persistent storage before any network
transmission occurs.

This guarantees that telemetry survives:

-   Network outages
-   LTE registration failures
-   Backend outages
-   Unexpected device resets
-   Power interruptions

------------------------------------------------------------------------

### At-Least-Once Delivery

The firmware guarantees **at-least-once delivery**.

Telemetry remains stored until the backend has successfully acknowledged
receipt.

Only after acknowledgement is the queued telemetry removed from
persistent storage.

------------------------------------------------------------------------

### FIFO Historical Replay

Historical records are processed in FIFO order within the replay path.

Live telemetry is intentionally allowed to reach the backend ahead of
historical backlog. Therefore, global network delivery is not strictly
FIFO across the combined live and replay paths.

Original timestamps and device identities are preserved so the backend
can correctly interpret historical records even when current telemetry
is delivered first.

------------------------------------------------------------------------

### Runtime Cleanup

A successful direct live upload persists an ACK for the exact record.
The physical record can remain in `ORBIQ.LOG` while older records are
still ahead of it.

When replay later reaches an already acknowledged queue-front record,
the storage layer removes it without uploading it again.

This preserves queue ordering while avoiding unnecessary duplicate
network transmission for records already accepted by the backend.

------------------------------------------------------------------------

### Scheduler Controlled Operation

The telemetry loop is driven entirely by the reporting scheduler.

The scheduler determines when the next telemetry cycle should occur
based on the current operating state.

This allows the firmware to reduce unnecessary network traffic while
maintaining timely updates during movement.

# SD Card Storage

The SD card provides persistent storage for telemetry, acknowledgements,
and queue management.

Rather than transmitting telemetry directly after acquisition, ORBI
Firmware first writes every telemetry record to persistent storage
before attempting any network communication.

This design ensures that telemetry is preserved even if connectivity is
unavailable or the device unexpectedly resets.

------------------------------------------------------------------------

## Board-Level Power Dependency

On the current ORBI reference hardware (LilyGO T-A7670), the MicroSD
card shares a board-level peripheral power rail with the SIMCom A7670E
modem.

This shared power rail is controlled by GPIO12 and is managed
exclusively by the board abstraction layer.

The firmware startup sequence therefore performs the following
operations before attempting to initialize the SD card:

Board Initialization │ ▼ Enable GPIO12 Shared Peripheral Power │ ▼
Initialize SD Card │ ▼ Initialize LTE Modem

Separating power management from the storage driver keeps the SD
subsystem independent of board-specific implementation details.

The SD driver assumes that required hardware resources are already
available when initialization begins.

This architecture also allows future ORBI hardware revisions to
implement different power-management strategies without requiring
changes to the SD card driver itself.

------------------------------------------------------------------------

## Verified Hardware Behaviour

The storage subsystem has been validated on physical ORBI hardware under
multiple operating conditions.

Successful verification includes:

-   SD card initialization during USB-powered operation
-   SD card initialization during LiPo-only operation
-   Persistent queue creation
-   Runtime telemetry persistence
-   Boot-time queue replay
-   Queue cleanup after successful backend acknowledgement
-   Offline telemetry buffering
-   Recovery after network restoration

These tests confirm that the persistent storage subsystem behaves
consistently regardless of the device power source and forms the
reliability foundation of the ORBI firmware.

------------------------------------------------------------------------

## Storage Files

The firmware currently uses the following SD-card files.

  -----------------------------------------------------------------------------
  File             Purpose
  ---------------- ------------------------------------------------------------
  `ORBITEST.TXT`   Verifies that the SD card is mounted correctly during
                   development and testing.

  `ORBIQ.LOG`      Persistent FIFO queue containing telemetry waiting to be
                   delivered to the backend.

  `ORBIACK.LOG`    Stores acknowledgements for telemetry successfully accepted
                   by the backend.

  `ORBITMP.LOG`    Temporary working file used while rebuilding the queue
                   during record removal.

  `ORBIGNSS.LOG`   Stores GNSS diagnostic information used during
                   development/runtime diagnostics.
  -----------------------------------------------------------------------------

------------------------------------------------------------------------

## Queue Behaviour

Every telemetry cycle follows the same storage sequence.

``` text
TelemetryRecord
        │
        ▼
Append to ORBIQ.LOG
        │
        ▼
Flush SD Card
        │
        ▼
Attempt HTTP Upload
```

Writing to the SD card always occurs before any network activity.

------------------------------------------------------------------------

## Successful Upload

Successful handling depends on whether the transmission is a direct live
record or a historical replay batch.

### Direct Live Success

``` text
Persist Current Record
│
▼
Direct Live HTTP Upload
│
▼
HTTP 2xx
│
▼
Persist ACK(device_id, timestamp)
│
▼
Leave physical queue ordering intact
│
▼
Replay later removes the record when it reaches queue front
```

### Replay Batch Success

``` text
Prepare Historical Batch
│
▼
HTTP 2xx
│
▼
Persist ACK(s)
│
▼
Remove acknowledged queue-front records
│
▼
Yield and continue background replay
```

Telemetry is not deliberately removed from persistent storage before
successful backend acceptance has been represented by acknowledgement
state.

------------------------------------------------------------------------

## Failed Upload

If a live or replay upload fails, pending telemetry is not discarded.

For replay failure:

``` text
Replay Batch
│
▼
Upload Fails
│
▼
No successful-finalization removal
│
▼
Queue remains pending
│
▼
Wait before retry
```

For a direct live record that was successfully persisted before the
failed upload, the record remains available in `ORBIQ.LOG` for later
recovery.

------------------------------------------------------------------------

## Background Replay

Historical recovery now runs as background work rather than as a
blocking startup mode.

``` text
Discover Pending Queue
│
▼
Start Operational Runtime
│
├── Acquire current sensor measurements
│
├── Persist current telemetry
│
├── Give current/live modem work foreground priority
│
└── Drain historical queue in bounded replay batches
```

Historical replay retains FIFO processing within the replay path. It
does not require the complete backlog to be transmitted before current
telemetry can reach the backend.

The modem owner arbitrates foreground and replay requests, while the
storage owner serializes persistent queue and ACK operations.

------------------------------------------------------------------------

## Runtime Queue Cleanup

Direct live telemetry that has already been accepted by the backend is
marked through its persisted ACK.

If older records are ahead of that live record, the physical record
remains in queue order. When replay eventually reaches an acknowledged
record at the front, it is removed without another upload.

This allows live telemetry to bypass a historical backlog without
corrupting the durable queue.

------------------------------------------------------------------------

## Verified Offline Recovery and Concurrency

The offline-first telemetry architecture and concurrent recovery model
have been validated on physical ORBI hardware.

Verified behaviour includes:

``` text
LTE / Backend Unavailable
│
▼
Telemetry Continues
│
▼
Records Persist to ORBIQ.LOG
│
▼
Historical Backlog Accumulates
│
▼
Connectivity Available
│
├── Live telemetry continues
│
├── New records continue being persisted
│
└── Historical replay drains bounded batches
│
▼
Successful records are acknowledged
│
▼
Acknowledged queue records are removed safely
```

During the concurrency stress test, replay progress advanced through
multiple four-record batches while GNSS acquisition, sensor acquisition,
live persistence, heartbeat activity, and direct live telemetry
continued.

This validates the core requirement that historical recovery must not
hide the device's current operational state from the backend.

### Remaining Replay Hardening

The core live/replay concurrency milestone is complete, but one
lifecycle edge case remains for future hardening: after replay has
completely drained the queue, the current replay task may terminate. If
a later direct live upload fails after that point, the persisted record
can remain queued until replay is started again, including on a later
boot.

The record remains durable; this is a replay-lifecycle scheduling
limitation rather than data loss.

## Delivery Guarantee

The current storage and delivery implementation provides:

-   Persistent telemetry storage
-   FIFO ordering within historical replay
-   Foreground direct-live delivery during backlog recovery
-   Automatic background replay when the replay task is active
-   Per-record ACK tracking
-   ACK-before-removal semantics
-   Runtime recovery of already acknowledged queue records
-   Preservation of original timestamps and device identity
-   At-least-once delivery semantics

These mechanisms form the reliability foundation of ORBI Firmware.

# Reporting Scheduler & Network Behaviour

ORBI Firmware does not transmit telemetry at a fixed interval.

Instead, telemetry generation is controlled by a reporting scheduler
that adjusts reporting frequency according to the operational state of
the device.

This approach reduces unnecessary LTE traffic while maintaining
responsive updates during movement.

------------------------------------------------------------------------

## Motion States

The scheduler currently classifies device movement into three operating
states.

  State    Description
  -------- ----------------------------------------------------------
  Moving   Device is travelling.
  Idle     Device has minimal movement but is not fully stationary.
  Parked   Device is stationary.

Movement classification is currently based on GNSS speed.

The modem reports speed in **knots**, which the firmware converts to
**kilometres per hour (km/h)** before evaluating the reporting policy.

------------------------------------------------------------------------

## Current Reporting Policy

The current firmware uses the following reporting intervals for
development and testing.

  Motion State   Reporting Interval
  -------------- --------------------
  Moving         10 seconds
  Idle           20 seconds
  Parked         30 seconds

These values are intentionally short to simplify firmware validation.

Production deployments will use configurable reporting intervals
supplied by the ORBI backend.

------------------------------------------------------------------------

## Scheduler Operation

Each telemetry cycle follows the sequence below.

``` text
Read GNSS

↓

Determine Motion State

↓

Select Reporting Interval

↓

Upload Telemetry

↓

Wait

↓

Repeat
```

The scheduler is responsible only for **when** telemetry is collected.

It does **not** determine trips, alerts, or operational intelligence.

Those responsibilities belong to the ORBI backend.

------------------------------------------------------------------------

# Heartbeat Behaviour

Heartbeats provide an additional indication that the device is
operational.

However, normal telemetry uploads already demonstrate device activity.

For this reason, heartbeats are transmitted only when required.

Current firmware behaviour is:

-   Successful telemetry uploads suppress heartbeat transmission.
-   If the backend has not received successful communication for an
    extended period, a heartbeat is generated.
-   Successful communication resets the heartbeat timer.

This significantly reduces unnecessary network traffic while preserving
device liveness monitoring.

------------------------------------------------------------------------

# Network Diagnostics

The firmware periodically evaluates modem connectivity.

Current checks include:

-   SIM availability
-   LTE registration
-   Packet data attachment
-   IP address allocation

Network diagnostics are **not** executed every telemetry cycle.

Instead they are performed:

-   Periodically
-   Immediately after communication failures

This reduces unnecessary AT command traffic while still allowing rapid
fault detection when connectivity problems occur.

------------------------------------------------------------------------

## Future Reporting Policy

Future firmware versions will receive reporting policies from the ORBI
backend.

Examples include:

-   Fleet-specific reporting intervals
-   Asset-specific reporting behaviour
-   Sensor-specific reporting frequencies
-   Dynamic reporting during alert conditions
-   Power-saving modes

This will allow reporting behaviour to be modified without rebuilding or
reflashing firmware.

# Verified Features

The current firmware has been validated through iterative hardware
testing on the ESP32-based ORBI development platform.

The following features have been implemented and verified.

------------------------------------------------------------------------

## Core Platform

-   ESP32 firmware running in a `no_std` environment.
-   Modular project architecture with clearly separated subsystems.
-   Hardware abstraction for board-specific functionality.
-   Device identity management.
-   Embassy cooperative runtime.
-   Dedicated modem and storage ownership.
-   Concurrent foreground live telemetry and background replay.

------------------------------------------------------------------------

## LTE Communication

Verified functionality includes:

-   SIM card detection
-   LTE network registration
-   Packet data attachment
-   IP address acquisition
-   HTTP communication with the ORBI backend
-   Automatic recovery from communication failures

------------------------------------------------------------------------

## GNSS

The firmware successfully retrieves and processes GNSS information,
including:

-   Latitude
-   Longitude
-   Altitude
-   UTC Time
-   Speed
-   Heading
-   Satellite count

GNSS speed is converted from knots to kilometres per hour before being
included in telemetry.

------------------------------------------------------------------------

## RS485 / Modbus

The RS485 communication subsystem has been successfully validated on the
ORBI reference hardware.

Verified functionality includes:

-   UART2 communication
-   MAX485 transceiver operation
-   Modbus RTU communication
-   Successful communication with the KUM ultrasonic fuel sensor
-   Raw measurement acquisition from the sensor
-   Decoding KUM measurement frames
-   SensorSnapshot integration
-   Unified telemetry generation
-   SD queue persistence of raw fuel telemetry
-   LTE transmission of raw fuel telemetry

This validation establishes the firmware communication foundation
required for future fuel telemetry integration.

### Hardware Validation Notes

The current ORBI reference hardware has been validated using:

-   LilyGO T-A7670
-   MAX485 RS485 transceiver
-   KUM ultrasonic fuel sensor

During validation it was confirmed that the reference MAX485 module
communicates successfully using the following TTL wiring:

GPIO21 → MAX485 TXD

GPIO22 → MAX485 RXD

VDD3V3 → MAX485 VCC

GND → MAX485 GND

This wiring was validated through successful Modbus RTU communication
with the KUM sensor.

Different RS485 modules may expose their TTL interface differently and
should always be verified during hardware bring-up.

## Telemetry

The telemetry subsystem currently supports:

-   Structured telemetry generation
-   Device identity inclusion
-   Timestamp generation
-   GPS measurements
-   Raw IMU measurements
-   JSON serialization
-   Backend upload
-   SensorSnapshot abstraction
-   Measurement-first telemetry
-   Raw KUM fuel telemetry
-   Physical sensor snapshots

------------------------------------------------------------------------

## Persistent Storage

The SD card subsystem provides:

-   Reliable SD card mounting
-   Queue file creation
-   Persistent telemetry storage
-   Background queue replay
-   ACK persistence
-   Runtime queue cleanup
-   Dedicated storage-owner coordination
-   Live persistence while replay is active

### Measurement-First Telemetry

Version 0.2.0 introduced a measurement-first telemetry architecture.

Firmware now publishes physical sensor measurements rather than derived
operational values.

A SensorSnapshot aggregates measurements from all installed sensors
during each sampling cycle before constructing a TelemetryRecord.

Raw fuel telemetry from the KUM ultrasonic sensor is transmitted without
calibration, allowing the ORBI backend to perform tank calibration,
normalization, and fuel intelligence centrally.

This architecture cleanly separates measurement acquisition from
operational interpretation and provides a scalable foundation for
supporting additional sensor technologies.

------------------------------------------------------------------------

## Reliable Delivery

The firmware currently provides:

-   Persistent-first telemetry storage for the normal live path
-   FIFO ordering within historical replay
-   Foreground direct-live delivery during backlog recovery
-   Bounded multi-record background replay
-   Per-record acknowledgement tracking
-   ACK-before-removal semantics
-   Runtime cleanup of already acknowledged queue records
-   Preservation of original timestamps and device identity
-   At-least-once delivery semantics
-   cooperative live/replay operation through dedicated modem and
    storage owners

------------------------------------------------------------------------

## Scheduler

The reporting scheduler currently supports:

-   Motion-aware reporting
-   Dynamic reporting intervals
-   Heartbeat suppression after successful uploads
-   Scheduled network diagnostics

------------------------------------------------------------------------

## Logging

Development logging currently provides visibility into:

-   Boot sequence
-   LTE registration
-   GNSS acquisition
-   Queue operations
-   ACK processing
-   Replay activity
-   Scheduler decisions
-   Upload success and failure

These logs have been extensively used to validate firmware behaviour
during development.

------------------------------------------------------------------------

## Backend Integration

The firmware has been successfully integrated with the ORBI backend,
including:

-   Device provisioning
-   Persistent device identity
-   Batch telemetry ingestion
-   Backend acknowledgement processing
-   Queue replay validation
-   Offline recovery validation
-   End-to-end telemetry persistence

Physical hardware testing has confirmed that queued telemetry survives
communication outages, historical records are processed in FIFO order
during replay, and current telemetry can continue reaching the backend
while historical backlog is draining.

Testing also confirmed ACK-before-removal behaviour and safe recovery of
records that had already been acknowledged through the direct-live path.

This validates the core concurrent telemetry path between the embedded
firmware and the ORBI Sensor Intelligence Platform.

# Firmware v0.2.0 Milestone

Version **0.2.0** represents the completion of the ORBI Firmware
communication and reliability foundation.

This milestone transitions the project from a proof-of-concept GPS
device into a production-oriented embedded telemetry platform capable of
reliable data persistence, network recovery, and backend integration.

------------------------------------------------------------------------

## Major Achievements

### Embedded Platform

-   Production `no_std` firmware architecture
-   Modular subsystem organization
-   Clean separation of drivers, networking, storage, scheduling, and
    telemetry

------------------------------------------------------------------------

### Communication

Successfully implemented:

-   LTE modem initialization
-   SIM management
-   Network registration
-   Packet data attachment
-   HTTP telemetry transmission
-   Separation of modem transport from HTTP protocol handling
-   Cooperative asynchronous `+HTTPACTION` polling
-   HTTP status validation before acknowledgement
-   Dedicated modem-owner task
-   Foreground live-telemetry priority at transaction boundaries
-   Background replay arbitration
-   Backend acknowledgement handling

------------------------------------------------------------------------

### Positioning

Successfully integrated:

-   GNSS initialization
-   Continuous location tracking
-   Speed conversion
-   Heading calculation
-   UTC timestamp acquisition

------------------------------------------------------------------------

### Persistent and Concurrent Telemetry

Implemented a persistent-first live path together with concurrent
background recovery:

``` text
Current Telemetry
│
▼
Persist to ORBIQ.LOG
│
▼
Direct Live Upload ───────────────┐
│                                 │
▼                                 │
Persist Live ACK                  │
                                  │
Historical Queue                  │
│                                 │
▼                                 │
Bounded Replay Batch              │
│                                 │
└────────► Modem Owner ◄──────────┘
             │
             ▼
          HTTP 2xx
             │
             ▼
      ACK / Queue Finalization
```

This allows current telemetry to reach the backend without waiting for
the complete historical backlog while preserving durable queue recovery.

------------------------------------------------------------------------

### Scheduler

Completed:

-   Motion-aware reporting
-   Dynamic reporting intervals
-   Heartbeat optimization
-   Network diagnostics scheduling

------------------------------------------------------------------------

### Backend Integration

Successfully validated against the ORBI backend:

-   Telemetry ingestion
-   ACK responses
-   Queue replay
-   Persistent delivery

The firmware now operates as an integrated component of the wider ORBI
Platform rather than as a standalone embedded application.

------------------------------------------------------------------------

## Development Status

At the completion of Version **0.2.0**, the firmware has established a
production-ready telemetry foundation comprising:

-   Reliable modem communication
-   Modular transport/protocol architecture
-   Embassy cooperative runtime
-   Dedicated modem ownership
-   Dedicated storage ownership
-   Persistent-first telemetry storage
-   Foreground live telemetry during backlog recovery
-   Bounded background replay
-   FIFO historical replay and queue management
-   Runtime acknowledgement processing
-   Motion-aware scheduling
-   Persistent device identity
-   End-to-end backend integration

These capabilities have been validated through physical hardware testing
and now provide a stable platform for expanding hardware support without
redesigning the communication architecture.

------------------------------------------------------------------------

## Next Major Objective

The next development phase introduces the **Sensor Abstraction Layer**.

This layer will allow ORBI Firmware to support multiple sensor
technologies through a common interface while maintaining a single
firmware codebase.

Future integrations will include:

-   RS485 / Modbus sensors
-   Fuel level sensors
-   Vibration sensors
-   Digital inputs
-   Ignition sensing
-   CAN bus
-   LoRa devices
-   Future ORBI sensor modules

# Development Roadmap

The ORBI Firmware roadmap is organized into progressive development
phases.

Each phase builds upon the previous one while preserving a stable and
production-ready core.

------------------------------------------------------------------------

# Phase 1 --- Embedded Platform Foundation ✅

Completed in Version 0.2.0.

This phase established the production-ready embedded foundation for ORBI
Firmware, including board bring-up, persistent storage, reliable
communications, runtime scheduling, provisioning support, and backend
integration.

Completed features include:

-   ESP32 board abstraction
-   Shared peripheral power sequencing
-   Persistent runtime device identity
-   LTE communication
-   GNSS integration
-   HTTP telemetry uploads
-   SD card persistence
-   FIFO replay
-   ACK processing
-   Runtime queue cleanup
-   Motion-aware scheduler
-   Backend provisioning compatibility
-   Backend integration

This phase provides the foundation for all future sensor integrations.

------------------------------------------------------------------------

# Phase 2 --- Sensor Abstraction Layer

With the communication and reliability foundation complete, the next
milestone is the introduction of the Sensor Abstraction Layer.

The objective is to standardize how physical sensors integrate with the
firmware while keeping the telemetry pipeline unchanged.

Current measurements already supported include:

-   GNSS
-   MPU6050 Accelerometer
-   MPU6050 Gyroscope
-   MPU6050 Temperature

The Sensor Abstraction Layer will introduce common interfaces for:

-   Sensor registration
-   Sensor initialization
-   Sensor polling
-   Sensor health monitoring
-   Sensor diagnostics
-   Measurement normalization
-   Unified telemetry generation

Each sensor driver will become responsible only for acquiring
measurements from its hardware.

The telemetry subsystem will remain responsible for building a single
normalized telemetry payload regardless of which sensors are installed
on a device.

This architecture will allow future hardware profiles to be created by
combining sensor drivers rather than maintaining separate firmware
projects.

------------------------------------------------------------------------

# Phase 3 --- RS485 / Modbus Integration

Following the Sensor Abstraction Layer, the first production sensor
integration is the KUM ultrasonic fuel sensor.

## Hardware Foundation Completed ✅

The following capabilities have been successfully validated on the ORBI
reference hardware:

-   UART2 bring-up
-   MAX485 RS485 transceiver integration
-   Modbus RTU request transmission
-   Successful Modbus RTU response reception
-   Physical communication with the KUM ultrasonic fuel sensor
-   Raw KUM measurement acquisition
-   UART loopback validation on GPIO21 and GPIO22

These validations establish the firmware communication foundation
required for RS485-based sensor support.

During hardware bring-up, the following reference wiring was
successfully validated for the current MAX485 transceiver module:

LilyGO GPIO21 → MAX485 TXD LilyGO GPIO22 → MAX485 RXD LilyGO VDD3V3 →
MAX485 VCC LilyGO GND → MAX485 GND

MAX485 A+ → KUM A MAX485 B- → KUM B

The communication path validated during testing is:

ESP32 UART2 │ ▼ MAX485 RS485 Transceiver │ ▼ RS485 Bus │ ▼ KUM
Ultrasonic Fuel Sensor │ ▼ 21-byte Modbus RTU Response

This milestone confirms that the firmware can reliably communicate with
the KUM sensor over RS485 using Modbus RTU.

------------------------------------------------------------------------

## Current Integration Status and Remaining Work

The KUM path has progressed beyond communication bring-up. Real hardware
has now validated raw KUM measurement acquisition, decoding,
SensorSnapshot integration, unified telemetry generation, SD
persistence, LTE batch upload, and backend measurement-first ingestion.

Tank calibration and fuel intelligence intentionally remain backend
responsibilities rather than firmware responsibilities.

Remaining firmware-side generalization includes:

-   Generic RS485 transport abstraction
-   Generic Modbus client
-   Sensor abstraction integration/generalization
-   Sensor diagnostics
-   Multi-vendor register mapping

The implementation will follow the established firmware architecture:

RS485 Driver │ ▼ Modbus Client │ ▼ KUM Device Profile │ ▼ Normalized
Fuel Measurement │ ▼ Telemetry Builder

The objective is to support additional Modbus-based sensors in the
future without changing the telemetry pipeline.

------------------------------------------------------------------------

# Replay / Live Telemetry Concurrency --- Completed Firmware Milestone ✅

The replay/live concurrency milestone has been implemented and
physically validated.

Completed capabilities include:

-   Embassy-based cooperative runtime tasks
-   continued sensor acquisition while replay is active
-   continued persistence of newly acquired telemetry
-   dedicated runtime modem ownership
-   dedicated runtime storage ownership
-   foreground priority for current/live modem work at transaction
    boundaries
-   bounded four-record replay batches
-   explicit Embassy channels and signals for task coordination
-   ACK-before-removal semantics
-   direct-live ACK recovery when replay later reaches the record
-   preservation of original timestamps and device identity
-   retry behaviour after transient replay upload failure
-   physical stress validation with historical backlog and simultaneous
    live operation

Validation demonstrated:

``` text
Large historical backlog exists
+
Current sensors continue producing measurements
+
New records continue reaching persistent storage
+
Current telemetry continues reaching the backend
+
Historical records continue draining in bounded batches
+
Acknowledged records are recovered safely
=
Replay / Live Concurrency Validated
```

This milestone is represented by firmware commit:

``` text
45b5f58 implement concurrent live telemetry and background replay
```

The core concurrency requirement is complete. Future hardening can
improve replay-task lifetime after the queue has fully drained and
optimize queue-front removal efficiency without redesigning the owner
architecture.

------------------------------------------------------------------------

# Phase 4 --- Vehicle Interface Expansion

Once RS485 sensor support has been established, the firmware will expand
to additional vehicle interfaces.

Planned integrations include:

-   Ignition sensing
-   Digital inputs
-   Digital outputs
-   Relay / Kill Switch control
-   Battery voltage monitoring
-   CAN bus interfaces
-   Driver identification

These capabilities will extend the firmware beyond telemetry collection
while maintaining the same modular architecture.

------------------------------------------------------------------------

# Phase 5 --- Industrial Intelligence

The firmware architecture is designed to support applications beyond
vehicle tracking.

Future deployments may include:

-   Generator monitoring
-   Stationary fuel tanks
-   Cold chain monitoring
-   Environmental sensing
-   Energy monitoring
-   Remote industrial assets

The same firmware architecture will be reused across these deployments
through hardware profiles and sensor abstraction.

------------------------------------------------------------------------

# Phase 6 --- Production Hardware

Once the firmware architecture has matured, development will transition
from evaluation hardware to custom ORBI hardware.

Future work includes:

-   Custom PCB design
-   Integrated LTE and GNSS
-   Industrial power management
-   Automotive-grade protection
-   Production enclosure design
-   Hardware certification
-   Manufacturing optimization

This phase marks the transition from prototype hardware to dedicated
ORBI devices.

# Future Device Management Capabilities

As ORBI devices are deployed into production, the firmware will
gradually introduce secure remote management capabilities.

Planned capabilities include:

-   Remote modem restart
-   Remote firmware restart
-   Watchdog reset reporting
-   Reset reason reporting
-   Remote diagnostics
-   Device health reporting
-   Command acknowledgement
-   Secure command validation
-   Remote configuration
-   OTA firmware updates (future production hardware)

These capabilities will be introduced incrementally alongside the
corresponding firmware subsystems rather than as a single development
milestone.

------------------------------------------------------------------------

# Guiding Principles

Throughout every development phase, the following principles remain
unchanged:

-   Modular architecture
-   Reliable telemetry delivery
-   Live telemetry priority during historical recovery
-   Backend-driven intelligence
-   Hardware abstraction
-   Reusable firmware components
-   Long-term maintainability

These principles ensure that ORBI Firmware continues to scale without
requiring fundamental architectural redesign.

# Future Platform Vision

ORBI Firmware is being developed as the embedded foundation of the wider
**ORBI Sensor Intelligence Platform**.

The long-term vision extends beyond GPS tracking or fuel monitoring.

The objective is to create a reusable embedded platform capable of
connecting physical assets, vehicles, infrastructure, and industrial
equipment to a common intelligence platform.

# Version Summary

## Version 0.2.0

The ORBI Firmware project has successfully completed its embedded
platform foundation.

Major milestones achieved include:

-   Production `no_std` firmware architecture
-   Board abstraction layer
-   Shared peripheral power management
-   Persistent runtime device identity
-   Reliable LTE communication
-   GNSS integration
-   Persistent SD-card telemetry queue
-   Concurrent background offline replay
-   Foreground live telemetry during backlog recovery
-   Dedicated modem and storage ownership
-   Backend provisioning compatibility
-   Motion-aware reporting scheduler
-   End-to-end backend integration
-   Physical hardware validation on the ORBI reference platform

With this foundation complete, the embedded runtime now supports
concurrent current telemetry and historical recovery without requiring a
redesign of the reliability architecture.

RS485/Modbus integration with the KUM ultrasonic fuel sensor has also
progressed beyond bring-up: raw measurements, SensorSnapshot
integration, SD persistence, LTE transmission, and backend
measurement-first ingestion have been physically validated. Remaining
firmware work focuses on abstraction/generalization, additional
interfaces, production hardening, and future ORBI hardware.

------------------------------------------------------------------------

## From Telemetry Device to Sensor Platform

Traditional telemetry systems are often designed around a specific
application.

A GPS tracker tracks vehicles.

A fuel monitoring device measures fuel.

A generator controller monitors generators.

ORBI takes a different approach.

``` text
Physical Asset
      │
      ▼
ORBI Device
      │
      ├── GNSS
      ├── Fuel
      ├── IMU
│     ├── Accelerometer
│     ├── Gyroscope
│     └── Temperature
      ├── Ignition
      ├── Digital Inputs
      ├── CAN Bus
      └── Future Sensors
      │
      ▼
Normalized Telemetry
      │
      ▼
ORBI Sensor Intelligence Platform
```

The firmware provides the common embedded infrastructure required to
collect and reliably transmit sensor data.

The backend transforms that telemetry into operational intelligence.

------------------------------------------------------------------------

## Potential Deployment Domains

The same firmware architecture can support multiple industries and
deployment types.

### Fleet and Logistics

-   Vehicle tracking
-   Fuel monitoring
-   Driver behaviour
-   Asset utilization
-   Route intelligence

### Construction and Mining

-   Heavy equipment monitoring
-   Fuel consumption
-   Equipment utilization
-   Unauthorized movement detection
-   Remote asset monitoring

### Energy and Utilities

-   Generator monitoring
-   Fuel storage monitoring
-   Power infrastructure telemetry
-   Remote substation monitoring
-   Distributed sensor networks

### Cold Chain

-   Temperature monitoring
-   Location tracking
-   Door monitoring
-   Environmental telemetry

### Industrial Monitoring

-   Tank level monitoring
-   Pressure sensing
-   Equipment vibration
-   Machine state monitoring
-   Remote industrial assets

------------------------------------------------------------------------

## Hardware Independence

The long-term architecture is designed to avoid dependency on a single
sensor manufacturer or hardware vendor.

Different devices may expose data through:

-   RS485 / Modbus
-   I²C
-   SPI
-   UART
-   GPIO
-   CAN Bus
-   LoRa
-   Future industrial protocols

The Sensor Abstraction Layer will normalize these hardware interfaces
into consistent telemetry that can be processed by the rest of the
firmware.

This allows hardware components to evolve without requiring fundamental
changes to the ORBI backend.

------------------------------------------------------------------------

## Backend-Driven Intelligence

ORBI Firmware intentionally performs minimal operational intelligence on
the embedded device.

The firmware is responsible for:

-   Sensor acquisition
-   Data normalization
-   Persistent buffering
-   Reliable transmission
-   Device health
-   Communication recovery

The ORBI backend is responsible for:

-   Fuel theft detection
-   Refill detection
-   Leak detection
-   Geofence intelligence
-   Trip analysis
-   Movement classification
-   Alert generation
-   Historical investigation
-   Replay intelligence
-   Map intelligence
-   Analytics
-   Future machine learning and AI capabilities

This separation allows intelligence models and business rules to evolve
without requiring firmware updates across deployed devices.

------------------------------------------------------------------------

## Long-Term Objective

The long-term objective is to establish ORBI as a hardware-independent
sensor intelligence ecosystem.

``` text
Sensors
    │
    ▼
ORBI Firmware
    │
    ▼
Reliable Telemetry
    │
    ▼
ORBI Platform
    │
    ├── Operational Intelligence
    ├── Investigation Intelligence
    ├── Replay Intelligence
    ├── Map Intelligence
    └── Analytics Intelligence
```

By maintaining a modular firmware architecture and a consistent
telemetry interface, ORBI can expand into new industries and sensor
technologies without rebuilding the platform from the ground up.

The firmware therefore serves as the bridge between physical-world
sensing and the intelligence capabilities of the ORBI Platform.

# Persistent Device Identity Provisioning

ORBI devices use a persistent runtime identity stored in the ESP32
internal flash.

The device code is no longer required to remain permanently hardcoded
into the telemetry subsystem.

This allows the same firmware architecture to support multiple physical
devices, each with its own unique identity.

------------------------------------------------------------------------

## Identity Architecture

Firmware metadata and physical-device identity are handled separately.

``` text
Firmware Identity
├── Firmware Version
├── Product Code
├── Hardware Profile
└── Capabilities

Runtime Device Identity
├── Device Code
└── Provisioning Status
```

Firmware metadata describes the software build and hardware
capabilities.

The runtime device identity uniquely identifies a particular physical
ORBI device.

------------------------------------------------------------------------

## Persistent Configuration Partition

A dedicated internal-flash partition stores device-specific
configuration.

``` text
Partition Name: orbi_config
Offset:         0x001F0000
Size:           64 KB
```

The partition is separate from:

-   The application firmware
-   The bootloader
-   The partition table
-   SD telemetry storage

This ensures that normal firmware updates do not replace the provisioned
device identity.

------------------------------------------------------------------------

## Current Partition Layout

``` text
0x000000
    │
    ├── Bootloader
    │
0x008000
    ├── Partition Table
    │
0x009000
    ├── NVS
    │
0x00F000
    ├── PHY Initialization
    │
0x010000
    ├── Factory Application
    │
0x1F0000
    ├── ORBI Configuration
    │
0x200000
```

The current layout uses the first 2 MB of the ESP32's available 4 MB
flash.

The remaining flash capacity is reserved for future expansion, including
possible OTA firmware support.

------------------------------------------------------------------------

## Identity Record Format

The first 64 bytes of the `orbi_config` partition contain the persistent
identity record.

``` text
Bytes 0–3     Magic value: ORBI
Byte 4        Record format version
Byte 5        Provisioning flags
Byte 6        Device-code length
Byte 7        Reserved
Bytes 8–39    Device code
Bytes 40–43   Checksum
Bytes 44–63   Reserved
```

The current maximum device-code length is 32 ASCII characters.

------------------------------------------------------------------------

## Identity Validation

Before using a stored identity, the firmware validates:

-   Magic value
-   Record-format version
-   Provisioning flag
-   Device-code length
-   UTF-8 validity
-   Record checksum

Invalid or corrupted identity records are rejected.

The firmware does not use unverified configuration data for telemetry.

------------------------------------------------------------------------

## Boot Identity Flow

At startup, the firmware follows this sequence:

``` text
Power On
    │
    ▼
Read orbi_config Partition
    │
    ▼
Identity Record Found?
    │
  ┌─┴───────────────┐
  │                 │
 NO                YES
  │                 │
  ▼                 ▼
Development      Validate Record
Fallback             │
  │                  ▼
  │           Validation Successful?
  │                ┌─┴─────────┐
  │                │           │
  │               NO          YES
  │                │           │
  ▼                ▼           ▼
Unprovisioned   Unprovisioned  Load Stored
Identity        Identity       Device Code
```

A valid persistent identity is marked:

``` text
Provisioned: true
```

A development fallback is marked:

``` text
Provisioned: false
```

------------------------------------------------------------------------

## Telemetry Identity Flow

The loaded runtime identity is passed into all outgoing communication
paths.

``` text
Persistent Device Identity
        │
        ▼
RuntimeDeviceIdentity
        │
        ├── Live Telemetry
        └── Heartbeat
```

The telemetry publisher and heartbeat builder do not directly access a
hardcoded global device code.

This allows the identity source to change without redesigning the
telemetry pipeline.

------------------------------------------------------------------------

## Replay Identity Behaviour

Queued telemetry retains the identity under which it was originally
generated.

``` text
ORBIQ.LOG Record
        │
        ├── Original Device Code
        └── Original Timestamp
        │
        ▼
Replay Payload
```

The replay subsystem does not replace historical record identities with
the current runtime identity.

This preserves telemetry ownership and acknowledgement consistency.

------------------------------------------------------------------------

## Verified Provisioning Test

Persistent provisioning was verified using:

``` text
Device Code: ORBI-GPS-003
Firmware:    0.2.0
Product:     ORBI-GPS-LITE
Profile:     GPS_ONLY
```

The following behaviour was confirmed:

-   Identity written to internal flash
-   Identity read back successfully
-   Checksum validation passed
-   Identity survived device reset
-   Firmware loaded the stored identity at boot
-   Live telemetry used `ORBI-GPS-003`
-   Heartbeats used `ORBI-GPS-003`
-   Backend inventory recognized the device
-   Device completed the manufacturing lifecycle
-   Device was provisioned to an asset
-   Backend accepted telemetry for the commissioned identity

------------------------------------------------------------------------

## Backend Commissioning Lifecycle

A physical identity must also exist within the ORBI Platform before
operational telemetry is accepted.

``` text
Write Device Identity
        │
        ▼
Create Inventory Record
        │
        ▼
PROGRAMMED
        │
        ▼
TESTED / PASSED
        │
        ▼
READY_FOR_DEPLOYMENT
        │
        ▼
Provision to Asset
        │
        ▼
Operational Telemetry
```

The embedded identity and backend inventory record must use the same
device code.

------------------------------------------------------------------------

## Current Limitation

The persistent storage mechanism is complete, but production
provisioning still requires an external provisioning utility.

The planned utility will:

-   Accept a device code
-   Validate its format
-   Write the identity record
-   Read the record back
-   Verify its checksum
-   Confirm the stored identity
-   Support repeatable manufacturing workflows

The objective is to flash one standard firmware image onto every board
and provision each unit separately without editing firmware source code.

------------------------------------------------------------------------

## Next Development Objective

Persistent Identity Storage ✅ │ ▼ External Provisioning Utility │ ▼
Repeatable Device Manufacturing │ ▼ Sensor Abstraction Layer │ ▼ RS485 /
Modbus Integration │ ▼ Vehicle Interface Expansion │ ▼ Industrial
Intelligence │ ▼ Production ORBI Hardware

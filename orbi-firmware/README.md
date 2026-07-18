# ORBI Firmware

Production embedded firmware for the **ORBI Sensor Intelligence Platform**.

ORBI Firmware is a modular, production-oriented Rust (`no_std`) firmware designed for intelligent telemetry devices deployed across fleets, construction equipment, generators, stationary fuel tanks, energy infrastructure, and future industrial monitoring applications.

Unlike traditional GPS tracker firmware, ORBI Firmware is designed as a **sensor platform**. GPS is only one sensor within a larger architecture capable of integrating multiple telemetry sources while maintaining a unified telemetry pipeline.

Current hardware targets include the ESP32 platform with the SIMCom A7670E LTE/GNSS modem, providing:

- GNSS positioning
- LTE communication
- SD card persistence
- Offline telemetry buffering
- Automatic replay after connectivity restoration

The firmware is intentionally modular to support future hardware revisions without changing the higher-level telemetry pipeline.

Current firmware capabilities include:

- GPS telemetry
- LTE backend communication
- Persistent SD card queue
- Upload acknowledgement tracking
- Automatic replay of offline telemetry
- Runtime queue cleanup
- Motion-aware reporting scheduler
- Production device identity
- Backend provisioning compatibility

The long-term objective is to provide a reusable embedded platform capable of supporting multiple ORBI hardware profiles while maintaining a consistent backend interface across all deployments.

## Project Goals

The ORBI Firmware project is designed around a single long-term principle:

> **Build one firmware platform capable of supporting many intelligent sensing applications.**

Rather than developing separate firmware for every product, ORBI Firmware provides a common embedded foundation that can be configured for different hardware profiles while exposing a consistent telemetry interface to the backend platform.

The firmware is designed to support deployments ranging from simple GPS trackers to complex multi-sensor intelligence devices.

Current and planned sensor support includes:

- GNSS positioning
- Fuel level monitoring (RS485 / Modbus)
- Vibration sensing (I²C)
- Ignition monitoring
- Digital inputs and outputs
- Relay/Kill switch control
- Future CAN Bus integration
- Future LoRa sensor gateways
- Additional industrial sensors through a modular driver architecture

The firmware intentionally separates hardware interaction from telemetry generation.

Sensor drivers are responsible only for communicating with physical devices and producing normalized sensor readings.

The telemetry layer combines those readings into a unified `TelemetryRecord`, allowing new sensors to be integrated without redesigning the networking, storage, replay, scheduling, or backend communication layers.

This modular architecture allows the same firmware foundation to support multiple ORBI product variants, including:

- GPS-only asset trackers
- Fuel Intelligence devices
- Fleet Intelligence devices
- Generator monitoring systems
- Stationary storage tank monitoring
- Industrial and energy monitoring solutions

The result is a scalable embedded platform where new hardware capabilities are added through modular sensor adapters rather than separate firmware projects.

## System Architecture

ORBI Firmware follows a layered architecture that separates hardware interaction, telemetry generation, persistent storage, networking, and scheduling into independent modules.

```text
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

The driver layer communicates directly with physical peripherals and sensors.

Examples include:

- GNSS receiver
- LTE modem
- SD card
- RS485 interfaces
- I²C devices
- GPIO peripherals

Drivers should **never contain business logic**. Their responsibility is simply to acquire or transmit data.

---

### Telemetry Layer

The telemetry layer converts raw hardware readings into normalized telemetry.

Every telemetry cycle produces a single `TelemetryRecord`, regardless of which sensors are installed on the device.

This keeps the backend interface stable while allowing hardware profiles to evolve independently.

---

### Storage Layer

The storage layer provides persistent buffering using the SD card.

Responsibilities include:

- Queueing telemetry before transmission
- Maintaining upload acknowledgements
- Offline persistence
- Queue replay
- Queue cleanup

The storage layer guarantees that telemetry is never discarded simply because connectivity is temporarily unavailable.

---

### Network Layer

The network layer manages all communication with the ORBI backend.

Responsibilities include:

- LTE connectivity
- HTTP communication
- Heartbeats
- Backend uploads
- Network diagnostics

The networking layer is intentionally isolated from sensor drivers so communication protocols can evolve independently.

---

### Scheduler Layer

The scheduler determines **when** telemetry should be generated.

Current scheduling decisions are based on:

- Vehicle movement
- Device activity
- Heartbeat timing
- Replay requirements

Future versions will allow these policies to be configured remotely from the backend.

## Repository Structure

The firmware is organised into small, independent modules. Each module has a single responsibility, making the codebase easier to understand, test, and extend.

```text
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

Contains board-specific hardware initialization.

Responsibilities include:

- ESP32 peripheral initialization
- Pin assignments
- Board configuration
- Clock setup
- Hardware abstraction for the target board

This layer isolates hardware-specific details from the rest of the firmware.

---

### `device/`

Contains the identity and configuration of the physical ORBI device.

Responsibilities include:

- Device identity
- Device code
- Firmware version
- Hardware profile
- Future manufacturing metadata

The device module represents **who the device is**, independent of the sensors attached to it.

---

### `drivers/`

Contains low-level hardware drivers.

Current drivers include:

- LTE modem
- GNSS
- SD card

Future drivers will include:

- RS485 / Modbus
- I²C sensors
- Ignition input
- Digital I/O
- CAN Bus
- LoRa interfaces

Drivers communicate directly with hardware and should never contain business or application logic.

---

### `network/`

Handles all backend communication.

Responsibilities include:

- LTE connectivity
- HTTP client
- Heartbeats
- Backend uploads
- Network diagnostics

Networking is intentionally isolated from the telemetry and storage layers so communication protocols can evolve without affecting sensor integration.

---

### `scheduler/`

Determines when telemetry should be generated.

Current scheduling decisions consider:

- Vehicle movement
- Idle state
- Parked state
- Heartbeat timing

Future versions will support backend-configurable reporting policies.

---

### `storage/`

Provides persistent storage using the SD card.

Responsibilities include:

- Telemetry queue
- Upload acknowledgements
- Queue replay
- Queue cleanup
- Persistent buffering during network outages

The storage layer guarantees reliable telemetry delivery even when connectivity is unavailable.

---

### `telemetry/`

Builds the telemetry exchanged with the backend.

Responsibilities include:

- TelemetryRecord creation
- Payload generation
- Replay payload construction
- Runtime publishing
- Replay processing

This layer combines normalized sensor data into a consistent backend interface.

---

### `main.rs`

The firmware entry point.

Responsibilities include:

- Hardware initialization
- Driver startup
- Runtime sequencing
- Scheduler execution
- Main telemetry loop

The `main` module coordinates the firmware but delegates implementation details to the individual modules.

## Hardware Platform

The current ORBI Firmware reference platform is built around the ESP32 and the SIMCom A7670E LTE/GNSS modem.

The firmware has been designed so that future hardware revisions can reuse the same software architecture with minimal changes.

### Primary Hardware

| Component         | Description              |
| ----------------- | ------------------------ |
| MCU               | ESP32                    |
| Cellular Modem    | SIMCom A7670E            |
| GNSS              | Integrated within A7670E |
| Storage           | MicroSD Card             |
| LTE Connectivity  | 4G LTE                   |
| Positioning       | GNSS (GPS)               |
| Firmware Language | Rust (`no_std`)          |

---

## Current Hardware Interfaces

The firmware is organised around reusable hardware interfaces rather than application-specific code.

### UART

Used for communication with:

- SIMCom A7670E LTE modem
- GNSS interface (through the modem)

---

### SPI

Used for:

- MicroSD card communication

The SD card provides persistent telemetry storage, allowing the firmware to continue operating during periods of network loss.

---

### GPIO

Current GPIO usage includes:

- Modem power control
- Modem reset
- Status signals

Future GPIO assignments will support:

- Ignition detection
- Digital inputs
- Relay outputs
- External alarms

---

### I²C

Reserved for sensor expansion.

Planned devices include:

- Vibration sensor
- Temperature sensor
- Environmental sensors
- Future industrial sensors

---

### RS485 / Modbus

The primary industrial sensor interface.

Planned support includes:

- Fuel level sensors
- Industrial level sensors
- Pressure sensors
- Third-party Modbus devices

The firmware is intentionally being developed with a generic Modbus architecture rather than a sensor-specific implementation.

---

## Device Profiles

The hardware architecture is designed to support multiple ORBI product variants without changing the overall firmware structure.

Examples include:

| Profile               | Sensors                           |
| --------------------- | --------------------------------- |
| GPS Tracker           | GNSS                              |
| Fuel Intelligence     | GNSS + RS485 Fuel Sensor          |
| Fleet Intelligence    | GNSS + Fuel + Vibration           |
| Generator Monitoring  | Fuel + Vibration + Digital Inputs |
| Industrial Monitoring | Custom sensor combinations        |

Each hardware profile shares the same telemetry pipeline while enabling only the drivers required for that deployment.

# Development Environment

ORBI Firmware is developed using the Rust embedded ecosystem targeting the ESP32 platform.

The project uses a `no_std` architecture together with the ESP HAL provided by Espressif.

## Development Environment

Current development platform:

| Component        | Version                 |
| ---------------- | ----------------------- |
| Operating System | Ubuntu (WSL2)           |
| Rust             | Stable                  |
| Target           | `xtensa-esp32-none-elf` |
| Framework        | esp-hal                 |
| Build System     | Cargo                   |
| Flash Tool       | espflash                |

---

## Required Software

Install the following tools before building the firmware.

### Rust

```bash
rustup update
```

---

### ESP Toolchain

```bash
espup install
```

---

### Export Environment

Before building, load the ESP environment.

```bash
source ~/export-esp.sh
```

This configures:

- Rust target
- Xtensa toolchain
- Linker
- ESP build environment

---

## Build Firmware

Build the production firmware:

```bash
cd ~/projects/fuel-intelligence-platform/orbi-firmware

source ~/export-esp.sh

cargo build --release
```

The firmware image will be generated at:

```text
target/xtensa-esp32-none-elf/release/orbi-firmware
```

---

## Flash Firmware

Flash the firmware to the ESP32:

```bash
espflash flash \
    --monitor \
    --ignore-app-descriptor \
    --port COM5 \
    "\\wsl.localhost\Ubuntu\home\william\projects\fuel-intelligence-platform\orbi-firmware\target\xtensa-esp32-none-elf\release\orbi-firmware"
```

---

## Serial Monitor

To monitor serial output without reflashing:

```bash
espflash monitor --port COM5
```

---

## Expected Boot Sequence

A successful firmware startup should include messages similar to:

```text
ESP32 Boot

↓

Board Initialization

↓

Modem Initialization

↓

Network Registration

↓

GNSS Initialization

↓

SD Card Mounted

↓

Replay Queue

↓

Live Telemetry Loop
```

Successful completion of this sequence indicates that the firmware is ready for normal telemetry operation.

# Device Provisioning Lifecycle

Every ORBI device follows a controlled lifecycle before it begins transmitting telemetry to the ORBI backend.

The firmware is designed to operate as part of the wider ORBI provisioning platform rather than as a standalone GPS tracker.

## Manufacturing Workflow

Every physical device progresses through the following stages:

```text
Manufactured
      │
      ▼
Firmware Programmed
      │
      ▼
Hardware Tested
      │
      ▼
Registered in Inventory
      │
      ▼
Provisioned to Customer
      │
      ▼
Activated
      │
      ▼
Operational
```

Each stage ensures that the device is correctly identified, tested, and associated with the appropriate customer assets before live telemetry is accepted by the backend.

---

## Device Identity

Every firmware build includes a device identity that uniquely identifies the physical hardware.

The current firmware uses the `DEVICE_IDENTITY` definition located within the `device` module.

Typical information includes:

- Device Code
- Firmware Version
- Hardware Profile
- Device Model

The device identity is included in every telemetry payload transmitted to the backend.

---

## Backend Provisioning

Before telemetry is accepted, the device must exist within the ORBI Platform.

Provisioning associates the physical device with:

- Organization
- Customer
- Asset
- Hardware Profile
- Sensor Configuration

This allows identical firmware builds to operate across multiple deployments while the backend determines which features and sensors are enabled for each device.

---

## Hardware Profiles

Hardware profiles define the capabilities of a device without requiring separate firmware projects.

Examples include:

| Hardware Profile   | Enabled Features            |
| ------------------ | --------------------------- |
| GPS Tracker        | GNSS                        |
| Fuel Intelligence  | GNSS + Fuel                 |
| Fleet Intelligence | GNSS + Fuel + Vibration     |
| Generator Monitor  | Fuel + Digital Inputs       |
| Industrial Monitor | Custom Sensor Configuration |

The firmware remains modular, while the backend controls which capabilities are active for each deployment.

---

## Future Remote Configuration

Future firmware versions will support backend-driven configuration, allowing devices to receive operational settings remotely.

Planned remotely configurable parameters include:

- Reporting intervals
- Enabled sensors
- Sensor calibration
- Alert thresholds
- Network behaviour
- Power management policies

This approach minimizes firmware changes while maximizing deployment flexibility.

# Runtime Architecture

Once powered on, ORBI Firmware follows a deterministic execution sequence designed to maximise reliability, recover from communication failures, and guarantee telemetry persistence.

The runtime is divided into two phases:

1. **Boot Phase**
2. **Operational Phase**

---

## Boot Phase

Every power cycle follows the same initialization sequence.

```text
Power On
    │
    ▼
Board Initialization
    │
    ▼
Load Device Identity
    │
    ▼
Initialize LTE Modem
    │
    ▼
Verify SIM Card
    │
    ▼
Register on LTE Network
    │
    ▼
Attach Packet Data
    │
    ▼
Acquire IP Address
    │
    ▼
Initialize GNSS
    │
    ▼
Mount SD Card
    │
    ▼
Replay Pending Queue
    │
    ▼
Enter Live Operation
```

The firmware does not begin normal telemetry collection until the replay stage has completed.

This guarantees that previously stored telemetry is transmitted before new telemetry is generated.

---

# Operational Phase

Once initialization has completed, the firmware enters a continuous telemetry loop.

Each iteration performs the following sequence.

```text
Read GNSS Position
        │
        ▼
Build TelemetryRecord
        │
        ▼
Append to ORBIQ.LOG
        │
        ▼
Flush SD Card
        │
        ▼
Heartbeat (only when due)
        │
        ▼
HTTP Upload
        │
        ▼
Upload Successful?
        │
      ┌─┴──────────────┐
      │                │
     NO               YES
      │                │
      ▼                ▼
Remain in Queue    Append ACK
                        │
                        ▼
             Runtime Queue Cleanup
                        │
                        ▼
            Reporting Scheduler
                        │
                        ▼
         Wait Until Next Reporting Cycle
```

---

## Runtime Principles

The runtime has been designed around several core principles.

### Persistent First

Telemetry is always written to persistent storage before any network transmission occurs.

This guarantees that telemetry survives:

- Network outages
- LTE registration failures
- Backend outages
- Unexpected device resets
- Power interruptions

---

### At-Least-Once Delivery

The firmware guarantees **at-least-once delivery**.

Telemetry remains stored until the backend has successfully acknowledged receipt.

Only after acknowledgement is the queued telemetry removed from persistent storage.

---

### FIFO Replay

Queued telemetry is replayed in chronological order.

Older telemetry is always transmitted before newer telemetry.

This preserves event ordering within the backend.

---

### Runtime Cleanup

After every successful live upload, the firmware immediately removes acknowledged telemetry from the queue.

This prevents long startup delays caused by accumulated acknowledged records and keeps the SD queue compact during normal operation.

---

### Scheduler Controlled Operation

The telemetry loop is driven entirely by the reporting scheduler.

The scheduler determines when the next telemetry cycle should occur based on the current operating state.

This allows the firmware to reduce unnecessary network traffic while maintaining timely updates during movement.

# SD Card Storage

The SD card provides persistent storage for telemetry, acknowledgements, and queue management.

Rather than transmitting telemetry directly after acquisition, ORBI Firmware first writes every telemetry record to persistent storage before attempting any network communication.

This design ensures that telemetry is preserved even if connectivity is unavailable or the device unexpectedly resets.

---

## Storage Files

The firmware currently maintains four files on the SD card.

| File           | Purpose                                                                            |
| -------------- | ---------------------------------------------------------------------------------- |
| `ORBITEST.TXT` | Verifies that the SD card is mounted correctly during development and testing.     |
| `ORBIQ.LOG`    | Persistent FIFO queue containing telemetry waiting to be delivered to the backend. |
| `ORBIACK.LOG`  | Stores acknowledgements for telemetry successfully accepted by the backend.        |
| `ORBITMP.LOG`  | Temporary working file used while rebuilding the queue during record removal.      |

---

## Queue Behaviour

Every telemetry cycle follows the same storage sequence.

```text
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

---

## Successful Upload

When the backend successfully accepts telemetry, the firmware performs the following operations.

```text
HTTP 200
        │
        ▼
Append ACK to ORBIACK.LOG
        │
        ▼
Remove acknowledged record from ORBIQ.LOG
        │
        ▼
Queue ready for next telemetry cycle
```

This guarantees that telemetry is only removed after the backend has confirmed successful receipt.

---

## Failed Upload

If communication fails for any reason, no data is discarded.

```text
TelemetryRecord
        │
        ▼
Stored in ORBIQ.LOG
        │
        ▼
HTTP Upload Fails
        │
        ▼
Record remains queued
```

The queued telemetry will be replayed automatically once connectivity is restored.

---

## Boot Replay

During startup, the firmware checks whether queued telemetry exists.

If queued records are found, they are processed before live telemetry begins.

```text
Boot
    │
    ▼
Replay Queue
    │
    ▼
Backend Upload
    │
    ▼
ACK Received
    │
    ▼
Remove Queue Record
    │
    ▼
Repeat Until Queue Empty
```

This prevents telemetry generated during previous offline periods from being lost.

---

## Runtime Queue Cleanup

After each successful live upload, the firmware immediately checks whether the oldest queued record has already been acknowledged.

If so, it is removed without waiting for the next reboot.

This keeps the queue small during normal operation and significantly reduces startup replay time.

---

## Delivery Guarantee

The current storage implementation provides:

- Persistent telemetry storage
- FIFO queue ordering
- Automatic replay
- ACK tracking
- Runtime queue cleanup
- Boot queue cleanup
- At-least-once telemetry delivery

These mechanisms form the reliability foundation of ORBI Firmware.

# Reporting Scheduler & Network Behaviour

ORBI Firmware does not transmit telemetry at a fixed interval.

Instead, telemetry generation is controlled by a reporting scheduler that adjusts reporting frequency according to the operational state of the device.

This approach reduces unnecessary LTE traffic while maintaining responsive updates during movement.

---

## Motion States

The scheduler currently classifies device movement into three operating states.

| State  | Description                                              |
| ------ | -------------------------------------------------------- |
| Moving | Device is travelling.                                    |
| Idle   | Device has minimal movement but is not fully stationary. |
| Parked | Device is stationary.                                    |

Movement classification is currently based on GNSS speed.

The modem reports speed in **knots**, which the firmware converts to **kilometres per hour (km/h)** before evaluating the reporting policy.

---

## Current Reporting Policy

The current firmware uses the following reporting intervals for development and testing.

| Motion State | Reporting Interval |
| ------------ | ------------------ |
| Moving       | 10 seconds         |
| Idle         | 20 seconds         |
| Parked       | 30 seconds         |

These values are intentionally short to simplify firmware validation.

Production deployments will use configurable reporting intervals supplied by the ORBI backend.

---

## Scheduler Operation

Each telemetry cycle follows the sequence below.

```text
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

---

# Heartbeat Behaviour

Heartbeats provide an additional indication that the device is operational.

However, normal telemetry uploads already demonstrate device activity.

For this reason, heartbeats are transmitted only when required.

Current firmware behaviour is:

- Successful telemetry uploads suppress heartbeat transmission.
- If the backend has not received successful communication for an extended period, a heartbeat is generated.
- Successful communication resets the heartbeat timer.

This significantly reduces unnecessary network traffic while preserving device liveness monitoring.

---

# Network Diagnostics

The firmware periodically evaluates modem connectivity.

Current checks include:

- SIM availability
- LTE registration
- Packet data attachment
- IP address allocation

Network diagnostics are **not** executed every telemetry cycle.

Instead they are performed:

- Periodically
- Immediately after communication failures

This reduces unnecessary AT command traffic while still allowing rapid fault detection when connectivity problems occur.

---

## Future Reporting Policy

Future firmware versions will receive reporting policies from the ORBI backend.

Examples include:

- Fleet-specific reporting intervals
- Asset-specific reporting behaviour
- Sensor-specific reporting frequencies
- Dynamic reporting during alert conditions
- Power-saving modes

This will allow reporting behaviour to be modified without rebuilding or reflashing firmware.

# Verified Features

The current firmware has been validated through iterative hardware testing on the ESP32-based ORBI development platform.

The following features have been implemented and verified.

---

## Core Platform

- ESP32 firmware running in a `no_std` environment.
- Modular project architecture with clearly separated subsystems.
- Hardware abstraction for board-specific functionality.
- Device identity management.

---

## LTE Communication

Verified functionality includes:

- SIM card detection
- LTE network registration
- Packet data attachment
- IP address acquisition
- HTTP communication with the ORBI backend
- Automatic recovery from communication failures

---

## GNSS

The firmware successfully retrieves and processes GNSS information, including:

- Latitude
- Longitude
- Altitude
- UTC Time
- Speed
- Heading
- Satellite count

GNSS speed is converted from knots to kilometres per hour before being included in telemetry.

---

## Telemetry

The telemetry subsystem currently supports:

- Structured telemetry generation
- Device identity inclusion
- Timestamp generation
- Motion state determination
- JSON serialization
- Backend upload

---

## Persistent Storage

The SD card subsystem provides:

- Reliable SD card mounting
- Queue file creation
- Persistent telemetry storage
- Queue replay after restart
- ACK persistence
- Runtime queue cleanup
- Boot queue cleanup

---

## Reliable Delivery

The firmware currently guarantees:

- Persistent-first telemetry storage
- FIFO queue ordering
- Automatic replay
- ACK verification
- Runtime acknowledgement processing
- At-least-once telemetry delivery

---

## Scheduler

The reporting scheduler currently supports:

- Motion-aware reporting
- Dynamic reporting intervals
- Heartbeat suppression after successful uploads
- Scheduled network diagnostics

---

## Logging

Development logging currently provides visibility into:

- Boot sequence
- LTE registration
- GNSS acquisition
- Queue operations
- ACK processing
- Replay activity
- Scheduler decisions
- Upload success and failure

These logs have been extensively used to validate firmware behaviour during development.

---

## Backend Integration

The firmware has been successfully integrated with the ORBI backend, including:

- Device registration
- Telemetry ingestion
- Persistent storage
- ACK responses
- Queue replay validation

This confirms end-to-end communication between the embedded device and the ORBI Platform.

# Firmware v0.2.0 Milestone

Version **0.2.0** represents the completion of the ORBI Firmware communication and reliability foundation.

This milestone transitions the project from a proof-of-concept GPS device into a production-oriented embedded telemetry platform capable of reliable data persistence, network recovery, and backend integration.

---

## Major Achievements

### Embedded Platform

- Production `no_std` firmware architecture
- Modular subsystem organization
- Clean separation of drivers, networking, storage, scheduling, and telemetry

---

### Communication

Successfully implemented:

- LTE modem initialization
- SIM management
- Network registration
- Packet data attachment
- HTTP telemetry transmission
- Backend acknowledgement handling

---

### Positioning

Successfully integrated:

- GNSS initialization
- Continuous location tracking
- Speed conversion
- Heading calculation
- UTC timestamp acquisition

---

### Persistent Telemetry

Implemented a complete persistent telemetry pipeline:

```text
Telemetry Generated
        │
        ▼
Persist to SD Card
        │
        ▼
Attempt Upload
        │
        ▼
Backend ACK
        │
        ▼
Runtime Queue Cleanup
```

This ensures telemetry is never discarded before successful backend acknowledgement.

---

### Offline Recovery

The firmware now survives:

- LTE outages
- Backend downtime
- Device resets
- Unexpected power loss

Pending telemetry is replayed automatically when connectivity returns.

---

### Scheduler

Completed:

- Motion-aware reporting
- Dynamic reporting intervals
- Heartbeat optimization
- Network diagnostics scheduling

---

### Backend Integration

Successfully validated against the ORBI backend:

- Telemetry ingestion
- ACK responses
- Queue replay
- Persistent delivery

The firmware now operates as an integrated component of the wider ORBI Platform rather than as a standalone embedded application.

---

## Development Status

At the completion of Version **0.2.0**, the following foundation has been established:

- Reliable communication
- Reliable storage
- Reliable replay
- Reliable scheduling
- Reliable backend integration

With these core capabilities complete, subsequent development can focus on expanding hardware support without redesigning the communication architecture.

---

## Next Major Objective

The next development phase introduces the **Sensor Abstraction Layer**.

This layer will allow ORBI Firmware to support multiple sensor technologies through a common interface while maintaining a single firmware codebase.

Future integrations will include:

- RS485 / Modbus sensors
- Fuel level sensors
- Vibration sensors
- Digital inputs
- Ignition sensing
- CAN bus
- LoRa devices
- Future ORBI sensor modules

# Development Roadmap

The ORBI Firmware roadmap is organized into progressive development phases.

Each phase builds upon the previous one while preserving a stable and production-ready core.

---

# Phase 1 — Communication Foundation ✅

Completed in Version 0.2.0.

This phase established the embedded communication platform.

Completed features include:

- ESP32 board support
- LTE communication
- GNSS integration
- HTTP telemetry uploads
- SD card persistence
- FIFO replay
- ACK processing
- Runtime queue cleanup
- Motion-aware scheduler
- Backend integration

This phase provides the foundation for all future sensor integrations.

---

# Phase 2 — Sensor Abstraction Layer

The next milestone introduces a unified sensor framework.

Rather than writing firmware for individual devices, ORBI Firmware will expose a common sensor interface that allows different sensor technologies to be integrated consistently.

Planned capabilities include:

- Sensor registration
- Sensor discovery
- Sensor polling
- Sensor health monitoring
- Sensor configuration
- Standardized telemetry generation

The objective is to ensure that new sensors can be added with minimal impact on the existing firmware architecture.

---

# Phase 3 — Fuel Intelligence

The first production sensor integration will focus on fuel monitoring.

Planned functionality includes:

- RS485 / Modbus communication
- Ultrasonic fuel sensors
- Fuel level normalization
- Sensor calibration
- Tank profile support
- Diagnostic monitoring
- Multi-vendor device profiles

This phase enables reliable fuel telemetry while supporting different sensor manufacturers through configurable register mappings.

---

# Phase 4 — Vehicle Intelligence

Following fuel integration, the firmware will expand to additional vehicle telemetry.

Planned integrations include:

- Ignition detection
- Digital inputs
- Relay outputs
- Engine status
- Battery voltage monitoring
- CAN bus interfaces
- Driver identification

These capabilities will extend ORBI Firmware into a comprehensive fleet telemetry platform.

---

# Phase 5 — Industrial Intelligence

The firmware architecture is designed to support applications beyond vehicle tracking.

Future deployments may include:

- Generator monitoring
- Stationary fuel tanks
- Cold chain monitoring
- Environmental sensing
- Energy monitoring
- Remote industrial assets

The same firmware architecture will be reused across these deployments through hardware profiles and sensor abstraction.

---

# Phase 6 — Production Hardware

Once the firmware architecture has matured, development will transition from evaluation hardware to custom ORBI hardware.

Future work includes:

- Custom PCB design
- Integrated LTE and GNSS
- Industrial power management
- Automotive-grade protection
- Production enclosure design
- Hardware certification
- Manufacturing optimization

This phase marks the transition from prototype hardware to dedicated ORBI devices.

---

# Guiding Principles

Throughout every development phase, the following principles remain unchanged:

- Modular architecture
- Reliable telemetry delivery
- Backend-driven intelligence
- Hardware abstraction
- Reusable firmware components
- Long-term maintainability

These principles ensure that ORBI Firmware continues to scale without requiring fundamental architectural redesign.

# Future Platform Vision

ORBI Firmware is being developed as the embedded foundation of the wider **ORBI Sensor Intelligence Platform**.

The long-term vision extends beyond GPS tracking or fuel monitoring.

The objective is to create a reusable embedded platform capable of connecting physical assets, vehicles, infrastructure, and industrial equipment to a common intelligence platform.

---

## From Telemetry Device to Sensor Platform

Traditional telemetry systems are often designed around a specific application.

A GPS tracker tracks vehicles.

A fuel monitoring device measures fuel.

A generator controller monitors generators.

ORBI takes a different approach.

```text
Physical Asset
      │
      ▼
ORBI Device
      │
      ├── GNSS
      ├── Fuel
      ├── Vibration
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

The firmware provides the common embedded infrastructure required to collect and reliably transmit sensor data.

The backend transforms that telemetry into operational intelligence.

---

## Potential Deployment Domains

The same firmware architecture can support multiple industries and deployment types.

### Fleet and Logistics

- Vehicle tracking
- Fuel monitoring
- Driver behaviour
- Asset utilization
- Route intelligence

### Construction and Mining

- Heavy equipment monitoring
- Fuel consumption
- Equipment utilization
- Unauthorized movement detection
- Remote asset monitoring

### Energy and Utilities

- Generator monitoring
- Fuel storage monitoring
- Power infrastructure telemetry
- Remote substation monitoring
- Distributed sensor networks

### Cold Chain

- Temperature monitoring
- Location tracking
- Door monitoring
- Environmental telemetry

### Industrial Monitoring

- Tank level monitoring
- Pressure sensing
- Equipment vibration
- Machine state monitoring
- Remote industrial assets

---

## Hardware Independence

The long-term architecture is designed to avoid dependency on a single sensor manufacturer or hardware vendor.

Different devices may expose data through:

- RS485 / Modbus
- I²C
- SPI
- UART
- GPIO
- CAN Bus
- LoRa
- Future industrial protocols

The Sensor Abstraction Layer will normalize these hardware interfaces into consistent telemetry that can be processed by the rest of the firmware.

This allows hardware components to evolve without requiring fundamental changes to the ORBI backend.

---

## Backend-Driven Intelligence

ORBI Firmware intentionally performs minimal operational intelligence on the embedded device.

The firmware is responsible for:

- Sensor acquisition
- Data normalization
- Persistent buffering
- Reliable transmission
- Device health
- Communication recovery

The ORBI backend is responsible for:

- Fuel theft detection
- Refill detection
- Leak detection
- Geofence intelligence
- Trip analysis
- Movement classification
- Alert generation
- Historical investigation
- Replay intelligence
- Map intelligence
- Analytics
- Future machine learning and AI capabilities

This separation allows intelligence models and business rules to evolve without requiring firmware updates across deployed devices.

---

## Long-Term Objective

The long-term objective is to establish ORBI as a hardware-independent sensor intelligence ecosystem.

```text
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

By maintaining a modular firmware architecture and a consistent telemetry interface, ORBI can expand into new industries and sensor technologies without rebuilding the platform from the ground up.

The firmware therefore serves as the bridge between physical-world sensing and the intelligence capabilities of the ORBI Platform.

# Persistent Device Identity Provisioning

ORBI devices use a persistent runtime identity stored in the ESP32 internal flash.

The device code is no longer required to remain permanently hardcoded into the telemetry subsystem.

This allows the same firmware architecture to support multiple physical devices, each with its own unique identity.

---

## Identity Architecture

Firmware metadata and physical-device identity are handled separately.

```text
Firmware Identity
├── Firmware Version
├── Product Code
├── Hardware Profile
└── Capabilities

Runtime Device Identity
├── Device Code
└── Provisioning Status
```

Firmware metadata describes the software build and hardware capabilities.

The runtime device identity uniquely identifies a particular physical ORBI device.

---

## Persistent Configuration Partition

A dedicated internal-flash partition stores device-specific configuration.

```text
Partition Name: orbi_config
Offset:         0x001F0000
Size:           64 KB
```

The partition is separate from:

- The application firmware
- The bootloader
- The partition table
- SD telemetry storage

This ensures that normal firmware updates do not replace the provisioned device identity.

---

## Current Partition Layout

```text
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

The current layout uses the first 2 MB of the ESP32's available 4 MB flash.

The remaining flash capacity is reserved for future expansion, including possible OTA firmware support.

---

## Identity Record Format

The first 64 bytes of the `orbi_config` partition contain the persistent identity record.

```text
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

---

## Identity Validation

Before using a stored identity, the firmware validates:

- Magic value
- Record-format version
- Provisioning flag
- Device-code length
- UTF-8 validity
- Record checksum

Invalid or corrupted identity records are rejected.

The firmware does not use unverified configuration data for telemetry.

---

## Boot Identity Flow

At startup, the firmware follows this sequence:

```text
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

```text
Provisioned: true
```

A development fallback is marked:

```text
Provisioned: false
```

---

## Telemetry Identity Flow

The loaded runtime identity is passed into all outgoing communication paths.

```text
Persistent Device Identity
        │
        ▼
RuntimeDeviceIdentity
        │
        ├── Live Telemetry
        └── Heartbeat
```

The telemetry publisher and heartbeat builder do not directly access a hardcoded global device code.

This allows the identity source to change without redesigning the telemetry pipeline.

---

## Replay Identity Behaviour

Queued telemetry retains the identity under which it was originally generated.

```text
ORBIQ.LOG Record
        │
        ├── Original Device Code
        └── Original Timestamp
        │
        ▼
Replay Payload
```

The replay subsystem does not replace historical record identities with the current runtime identity.

This preserves telemetry ownership and acknowledgement consistency.

---

## Verified Provisioning Test

Persistent provisioning was verified using:

```text
Device Code: ORBI-GPS-003
Firmware:    0.2.0
Product:     ORBI-GPS-LITE
Profile:     GPS_ONLY
```

The following behaviour was confirmed:

- Identity written to internal flash
- Identity read back successfully
- Checksum validation passed
- Identity survived device reset
- Firmware loaded the stored identity at boot
- Live telemetry used `ORBI-GPS-003`
- Heartbeats used `ORBI-GPS-003`
- Backend inventory recognized the device
- Device completed the manufacturing lifecycle
- Device was provisioned to an asset
- Backend accepted telemetry for the commissioned identity

---

## Backend Commissioning Lifecycle

A physical identity must also exist within the ORBI Platform before operational telemetry is accepted.

```text
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

The embedded identity and backend inventory record must use the same device code.

---

## Current Limitation

The persistent storage mechanism is complete, but production provisioning still requires an external provisioning utility.

The planned utility will:

- Accept a device code
- Validate its format
- Write the identity record
- Read the record back
- Verify its checksum
- Confirm the stored identity
- Support repeatable manufacturing workflows

The objective is to flash one standard firmware image onto every board and provision each unit separately without editing firmware source code.

---

## Next Development Objective

```text
Persistent Identity Storage ✅
        ↓
External Provisioning Utility
        ↓
Repeatable Device Manufacturing
        ↓
Sensor Abstraction Layer
```

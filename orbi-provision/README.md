# ORBI Provision Utility

**ORBI Provision** is the official manufacturing and provisioning utility for the ORBI Sensor Intelligence Platform.

It is responsible for generating, validating, programming and verifying immutable device identities during manufacturing.

Unlike application software, this utility is intended for factory provisioning, hardware bring-up and production workflows.

---

# Purpose

Every ORBI hardware device requires a permanent identity before it can join the ORBI platform.

This utility provides a secure and repeatable provisioning workflow that:

- Generates authenticated device identities
- Programs identities into ESP32 flash
- Reads identities from devices
- Verifies identity integrity
- Prevents accidental overwriting of existing identities

The provisioning process is deliberately separated from the backend platform because device identity exists before the device is ever registered inside the cloud.

---

# Position within the ORBI Ecosystem

```text
                     ORBI Platform

                 ┌───────────────────────┐
                 │   Web Dashboard        │
                 └──────────┬────────────┘
                            │
                 ┌──────────▼────────────┐
                 │ Backend APIs          │
                 └──────────┬────────────┘
                            │
                 ┌──────────▼────────────┐
                 │ Device Registration   │
                 └──────────┬────────────┘
                            │
                     Physical Device
                            ▲
                            │
                 ┌──────────┴────────────┐
                 │ ORBI Provision Utility│
                 └───────────────────────┘
```

Provisioning is therefore the bridge between hardware manufacturing and the ORBI cloud platform.

---

# Manufacturing Workflow

```text
Generate Device Identity
            │
            ▼
Validate Identity
            │
            ▼
Read Device Flash
            │
            ▼
Blank Identity Region?
       │             │
      Yes            No
       │             │
       ▼             ▼
Program Flash     Abort
       │
       ▼
Read Back
       │
       ▼
Verify Authentication
       │
       ▼
Provisioning Complete
```

---

# System Architecture

The project follows a layered architecture.

```text
CLI
 │
 ▼
Commands
 │
 ▼
Services
 │
 ├── Identity Service
 └── Provision Service
 │
 ▼
Identity Layer
 │
 ├── Version 1
 ├── Version 2
 ├── Authentication
 └── Validation
 │
 ▼
Flash Provider
 │
 ▼
Espflash Provider
 │
 ▼
ESP32 Flash
```

Each layer has a single responsibility.

---

# Project Structure

```text
src/

├── cli.rs
├── commands/
│   ├── generate.rs
│   ├── provision.rs
│   └── read.rs
│
├── flash/
│   ├── mod.rs
│   └── espflash.rs
│
├── identity/
│   ├── auth/
│   ├── record.rs
│   ├── validation.rs
│   ├── v1.rs
│   └── v2.rs
│
├── services/
│   ├── identity.rs
│   └── provision.rs
│
└── main.rs
```

---

# Identity Formats

The utility currently supports two identity formats.

## Version 1

Legacy format.

Authentication:

- FNV-1a checksum

Purpose:

- Early development
- Initial prototypes

---

## Version 2

Current production format.

Authentication:

- HMAC-SHA-256

Features:

- Runtime manufacturing key
- Tamper detection
- Cryptographic verification

Version 2 is the default format for new ORBI hardware.

---

# Flash Layout

Each ORBI device stores a fixed 64-byte identity record.

```text
Offset      Size

0x00        4      Magic
0x04        1      Format Version
0x05        1      Flags
0x06        1      Device Code Length
0x07        1      Authentication Algorithm
0x08        32     Device Code
0x28        16     Authentication Tag
0x38        8      Reserved
```

Flash Address:

```
0x001F0000
```

---

# Authentication

## Version 1

Uses:

- FNV-1a checksum

Provides accidental corruption detection only.

---

## Version 2

Uses:

- HMAC-SHA-256

Provides:

- Authentication
- Tamper detection
- Identity verification

The HMAC is generated using a manufacturing key that never becomes part of the programmed identity.

---

# Manufacturing Key

Version 2 identities require a binary manufacturing key.

Example:

```
secrets/
    orbi-development.key
```

The key is loaded only during provisioning.

It is never stored inside the identity record.

---

# Security Model

The provisioning utility follows several safety principles.

## Immutable Identity

Device identities should never change after manufacturing.

---

## Overwrite Protection

Before programming:

- Flash is read
- Identity region inspected

If the region already contains an identity:

```
Provisioning Aborted
```

This prevents accidental destruction of existing identities.

---

## Authentication Verification

Every programmed Version 2 identity is:

- Read back
- HMAC verified

before provisioning succeeds.

---

# Building

```bash
cargo build --release
```

Run tests:

```bash
cargo test
```

---

# CLI Commands

## Generate Version 1

```bash
cargo run --release -- generate-v1 \
    --device-code ORBI-A100-000001
```

---

## Generate Version 2

```bash
cargo run --release -- generate-v2 \
    --device-code ORBI-A100-000001 \
    --key-file secrets/orbi-development.key
```

---

## Read Identity

```bash
cargo run --release -- read \
    --port COM5
```

---

## Provision Device

```bash
cargo run --release -- provision-v2 \
    --port COM5 \
    --device-code ORBI-A100-000001 \
    --key-file secrets/orbi-development.key
```

---

# Typical Provisioning Sequence

```text
Connect Board

↓

Generate Identity

↓

Read Flash

↓

Blank?

↓

Write Identity

↓

Read Back

↓

Verify HMAC

↓

Provision Complete
```

---

# Current Status

## Completed

- Versioned identities
- Version 1 generation
- Version 2 generation
- HMAC authentication
- Runtime manufacturing keys
- Identity validation
- Flash reading
- Flash writing
- Read-back verification
- Overwrite protection
- Provisioning CLI
- Windows COM port support
- WSL-compatible implementation

---

# Hardware Validation

Successfully validated:

- ESP32 connection
- Flash reading
- Identity decoding
- Provisioning safety checks
- Overwrite protection

Pending validation:

- Full provisioning cycle on a blank device
- Write → Read-back → HMAC verification on erased identity region

---

# Roadmap

Future manufacturing improvements include:

- Batch provisioning
- Factory programming station
- QR code generation
- Barcode integration
- Device certificates
- Firmware flashing during provisioning
- Secure manufacturing key management
- Audit logging
- Production database integration

---

# Relationship to the ORBI Platform

The ORBI Provision Utility is intentionally independent of the backend.

Responsibilities are separated as follows.

## ORBI Provision

- Manufacture identities
- Program hardware
- Verify hardware
- Protect immutable identities

## ORBI Backend

- Register devices
- Associate assets
- Manage organizations
- Store telemetry
- Analytics
- Alerting
- Investigation
- Intelligence

Provisioning happens once.

The backend operates throughout the device's lifetime.

---

# Development Philosophy

The project follows several principles.

- Layered architecture
- Strong separation of concerns
- Immutable hardware identity
- Cryptographic verification
- Explicit validation
- Production-safe provisioning
- Minimal hardware assumptions
- Testable business logic
- Hardware abstraction through providers

These principles ensure the provisioning utility can evolve into a production-grade manufacturing tool without major architectural changes.

---

# License

This project is part of the ORBI Sensor Intelligence Platform.

Copyright © ORBI Geospatial.

All rights reserved.

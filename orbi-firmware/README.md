1. Project Overview

Explain that orbi-firmware is the production embedded firmware for ORBI Sensor Intelligence Platform.

2. System Architecture
   Drivers
   │
   ├── GNSS
   ├── LTE Modem
   ├── I2C
   │
   ▼
   TelemetryRecord
   │
   ├── Storage
   └── Network
   │
   ▼
   Rust Backend
   │
   ▼
   Dashboard
3. Repository Structure

The new structure:

src/
├── board/
├── device/
├── drivers/
├── network/
├── storage/
├── telemetry/
└── main.rs

with an explanation of every folder.

4. Hardware

Include all confirmed GPIO assignments:

Modem
SD
I²C
External GNSS
UART 5. Build Environment

Update to:

cd ~/projects/fuel-intelligence-platform/orbi-firmware
source ~/export-esp.sh
cargo build --release 6. Flash Commands

Update to:

espflash flash --monitor --ignore-app-descriptor --port COM5 "\\wsl.localhost\Ubuntu\home\william\projects\fuel-intelligence-platform\orbi-firmware\target\xtensa-esp32-none-elf\release\orbi-firmware"

and:

espflash monitor --port COM5 7. Provisioning

Document that the firmware uses:

DEVICE_IDENTITY

and that every physical ORBI device must first be:

Manufactured
↓

Programmed

↓

Tested

↓

Provisioned

↓

Activated

before telemetry is accepted by the backend.

8. Runtime Flow

Document the exact runtime sequence we have now proven:

Power On

↓

Modem

↓

Network

↓

GNSS Fix

↓

TelemetryRecord

↓

Append ORBIQUE.LOG

↓

Flush SD

↓

Heartbeat

↓

HTTP Upload

↓

Backend Storage 9. SD Storage

Document:

ORBITEST.TXT

and:

ORBIQUE.LOG

Explain that:

ORBITEST.TXT is the filesystem verification file.
ORBIQUE.LOG stores live telemetry before transmission. 10. Verified Features

Everything we have physically verified:

Feature Status
ESP32 Boot ✅
Flashing ✅
Serial Monitor ✅
LTE Modem ✅
SIM Detection ✅
LTE Registration ✅
HTTP ✅
GNSS ✅
Device Identity ✅
TelemetryRecord ✅
SD Detection ✅
FAT Mount ✅
File Creation ✅
Live SD Logging ✅
Backend Upload ✅
End-to-End Rust Pipeline ✅ 11. Current Architecture Milestone

Something like:

v0.1.0

✓ Rust no_std firmware

✓ LTE

✓ GNSS

✓ SD logging

✓ TelemetryRecord

✓ Heartbeat

✓ HTTP upload

✓ Backend integration 12. Next Milestones

The roadmap should now be:

□ Offline retry queue

□ Upload acknowledgement

□ Fuel (RS485)

□ Vibration

□ Ignition

□ Remote configuration

□ OTA firmware updates

□ Production PCB

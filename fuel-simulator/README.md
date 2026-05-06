# Fuel Simulator

This Rust project simulates a fuel monitoring device for the Fuel Intelligence Platform.

It acts as a fake device before real hardware is connected.

## Purpose

The simulator helps us test the full fuel monitoring pipeline before using real sensors.

It currently simulates:

- normal fuel consumption
- sudden fuel drop/theft
- slow leak
- refill event
- offline network periods
- delayed batch syncing
- local file logging

## Current System Flow

````text
FuelSimulator
→ generates FuelReading
→ stores reading locally
→ adds reading to SyncQueue
→ checks NetworkSimulator
→ syncs ReadingBatch when online
→ logs synced batch to file
Project Structure
fuel-simulator/
├── config.toml
├── Cargo.toml
├── data/
│   ├── readings.jsonl
│   └── synced_batches.jsonl
└── src/
    ├── config.rs
    ├── main.rs
    ├── models.rs
    ├── network.rs
    ├── simulator.rs
    ├── storage.rs
    └── sync_queue.rs
Main Modules
models.rs

Contains shared data structures.

Important structs:

FuelReading
ReadingBatch

FuelReading is one time-based reading.

ReadingBatch is a group of readings sent together.

simulator.rs

Contains the FuelSimulator.

The simulator holds the internal changing state of the fuel tank/device.

It tracks:

current fuel level
current simulated time
device ID
tank capacity
location
event schedule

It produces FuelReading records over time.

sync_queue.rs

Stores readings temporarily before syncing.

This simulates local device buffering when internet is poor or unavailable.

network.rs

Simulates online/offline network conditions.

This helps us test Africa-ready offline behaviour.

storage.rs

Writes readings and synced batches to local .jsonl files.

config.rs

Loads configuration from config.toml.

This keeps device settings out of the Rust source code.

Configuration

Edit:

config.toml

Example:

device_id = "DEV001"
tank_capacity_litres = 200.0
initial_fuel_litres = 180.0
latitude = 5.6037
longitude = -0.1870
batch_size = 5
reading_sleep_seconds = 2

theft_reading_number = 10
leak_start_reading = 20
leak_end_reading = 25
refill_reading_number = 35
How to Run

From inside fuel-simulator/:

cargo run
Output Files

The simulator creates a data/ folder.

data/readings.jsonl

Stores every generated reading.

data/synced_batches.jsonl

Stores every batch that was synced while the device was online.

Important Concept

There are two different times in the system:

Reading timestamp

When the fuel reading actually happened.

Synced time

When the device successfully sent the batch.

This matters because in real African deployments, the device may collect readings while offline and sync them later.

Example:

Fuel theft happened at 02:00
Device synced data at 05:00
System must still detect the theft at 02:00
Current Limitations

This simulator does not yet:

send data to an API
store data in PostgreSQL
run backend anomaly detection
use real hardware
use GSM/WiFi
use real fuel sensors
Next Phase

The next phase is to build the Rust ingestion service.

Planned endpoint:

POST /api/fuel-readings/batch

The simulator will send ReadingBatch payloads to the backend service.

Long-Term Goal

This simulator will later be replaced by a real device flow:

Fuel Sensor
→ ESP32
→ GSM/WiFi
→ Rust Backend
→ Database
→ Analytics
→ Dashboard/Alerts

What this README does:

```text
documents the current simulator phase
explains the modules
records the system flow
prepares the next backend phase
````

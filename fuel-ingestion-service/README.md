# Fuel Ingestion Service

## Overview

The Fuel Ingestion Service is the operational intelligence backend for the Sensor Intelligence Platform.

It is responsible for:

- receiving sensor/device batches
- persisting sensor readings
- reconstructing historical timelines
- detecting operational fuel events
- supporting offline-safe synchronization
- exposing operational intelligence through APIs

The system is designed primarily for African deployment environments where:

- internet may be unstable
- synchronization may delay
- devices may reconnect later
- operational reconstruction is critical

---

# Current Capabilities

## Sensor Reading Ingestion

The service currently supports:

- batch reading ingestion
- delayed synchronization
- timestamp preservation
- PostgreSQL persistence
- fuel telemetry persistence
- GPS telemetry persistence
- vibration telemetry persistence
- motion detected telemetry persistence

---

# Detection Engine

Current implemented event types:

## THEFT

Detects large sudden fuel drops.

Example:

```text
180L → 130L
```

---

## REFILL

Detects large sudden fuel increases.

Example:

```text
120L → 160L
```

---

## LEAK (Pattern-Based)

Detects continuous gradual fuel reduction across multiple readings.

Example:

```text
180L
178L
176L
174L
```

This currently represents:

```text
possible leak detection
```

not guaranteed confirmation.

---

## Confidence Scoring

Fuel events now include deterministic confidence scoring.

Current confidence levels:

````text
Low
Medium
High
Critical

Confidence is based on:

event type
device operational state
telemetry outlier count
candidate count
suspicious fuel jump status
delayed detection status

The confidence value is stored separately on fuel events and exposed through the fuel events API.

---

# Offline-Safe Detection Philosophy

The platform distinguishes:

- recorded_at
- received_at
- detected_at

This allows:

- delayed synchronization
- historical event reconstruction
- delayed operational alerts

Example:

```text
event occurs at 02:00
device reconnects at 05:00
backend still detects event correctly at 05:00
````

---

# Current Operational Intelligence Features

Implemented:

- event persistence
- duplicate suppression
- event correlation suppression
- timeline reconstruction
- delayed-event handling

---

## Device Health Intelligence

The service supports device operational health tracking.

Implemented:

- heartbeat ingestion
- payload seen tracking
- `last_seen_at`
- `last_payload_at`
- `last_heartbeat_at`
- device status classification
- device health event persistence
- configurable health thresholds
- automatic background health refresh

Device status values:

```text
ONLINE
STALE
OFFLINE
UNKNOWN
```

Status logic is based on `last_seen_at`.

Configuration:

```env
DEVICE_STALE_AFTER_SECONDS=120
DEVICE_OFFLINE_AFTER_SECONDS=600
DEVICE_HEALTH_REFRESH_INTERVAL_SECONDS=30
```

For development testing, smaller values can be used:

```env
DEVICE_STALE_AFTER_SECONDS=5
DEVICE_OFFLINE_AFTER_SECONDS=15
DEVICE_HEALTH_REFRESH_INTERVAL_SECONDS=5
```

The backend automatically refreshes device health in the background using `DEVICE_HEALTH_REFRESH_INTERVAL_SECONDS`.

Manual refresh is still available for testing, but production does not depend on manually calling the refresh endpoint.

# Current API Endpoints

## Batch Ingestion

```http
POST /api/fuel-readings/batch
```

---

## List Fuel Events

```http
GET /api/fuel-events
```

Fuel event responses include:

- severity
- confidence
- message
- fuel difference
- delayed detection metadata
  Returns recent operational events as JSON.

---

---

## Device Heartbeat

```http
POST /api/heartbeat
```

Receives device heartbeat messages and marks the device as online.

---

## Refresh Device Health

```http
POST /api/devices/refresh-health
```

Manually reclassifies device health using the configured thresholds.

This is useful for testing, but background refresh runs automatically.

---

## List Device Health Events

```http
GET /api/device-health-events
```

Returns recent device health transitions such as:

```text
ONLINE → STALE
STALE → OFFLINE
OFFLINE → ONLINE
```

---

## List Sensor Health Events

```http
GET /api/sensor-health-events
```

Returns recent sensor operational health issues such as:

```text
SENSOR_FROZEN
```

---

## List Device State Events

````http
GET /api/device-state-events

Returns recent classified operational states such as:

MOVING
IDLE
PARKED

## Sensor Health Intelligence

The service now supports independent sensor operational health monitoring.

Current implemented sensor health events:

```text
SENSOR_FROZEN
````

Frozen sensor detection identifies situations where:

- the device is still online
- heartbeats continue arriving
- but sensor values remain suspiciously unchanged

Possible causes include:

- RS485 communication failure
- disconnected sensor
- stuck sensor value
- replayed/static telemetry
- wiring issues
- sensor malfunction

Current capabilities:

- deterministic state classification
- GPS-aware movement classification
- vibration-aware classification
- batch-sequential movement analysis
- distance estimation in meters
- approximate speed estimation
- operational state persistence
- historical state reconstruction
- operational state API endpoint
- frozen sensor detection
- duplicate suppression
- sensor health event persistence
- sensor health event APIs

---

# Device State Engine

The service now supports operational device state classification using incoming telemetry.

Current implemented states:

````text
MOVING
IDLE
PARKED
OFFLINE
UNKNOWN

The Device State Engine currently uses:

- vibration telemetry
- motion detection
- device health status
- previous GPS location
- current GPS location

to classify operational behavior.

The engine now supports GPS-aware movement classification by comparing previous and current coordinates during telemetry ingestion.

Movement detection now uses estimated GPS distance in meters instead of raw coordinate subtraction.

Example correlations:

high vibration
+ motion detected
=================
MOVING
low vibration
+ no motion
=================
PARKED

Current capabilities:

State history is stored in:

device_state_events

Each state event can also store:

- estimated distance moved
- approximate speed in km/h

This provides the foundation for:

trip reconstruction
fleet monitoring
operational analytics
tamper reasoning
future GPS-aware intelligence
future ML-assisted behavioral analysis

---

# Telemetry Quality Layer

The service now supports statistical telemetry quality analysis before operational intelligence processing.

Current implemented telemetry quality tools:

- fuel range validation
- impossible jump detection
- median calculation
- rolling median windows
- interquartile range (IQR) calculation
- IQR-based outlier detection
- persistent outlier counting
- telemetry quality window summaries

The filtering layer is designed to distinguish between:

- real operational events
- sensor spikes
- noisy telemetry
- suspicious abnormal patterns

Outliers are not automatically discarded.

Instead, the platform supports persistence-aware anomaly reasoning to avoid suppressing legitimate operational incidents such as:

- real fuel theft
- real fuel leaks
- legitimate fuel refills

---

## Telemetry Detection Configuration

Current configurable telemetry intelligence settings:

```env
DEFAULT_TANK_CAPACITY_LITRES=200
MAX_ALLOWED_FUEL_JUMP_LITRES=80
FUEL_ROLLING_WINDOW_SIZE=5
FUEL_IQR_MULTIPLIER=1.5

# Current Architecture

```text
Device / Simulator
→ Offline Queue
→ Batch Synchronization
→ Axum API
→ PostgreSQL
→ Stored Telemetry Layer
    ├── Fuel Telemetry
    ├── GPS Telemetry
    ├── Vibration Telemetry
    └── Motion Telemetry
→ Operational Intelligence Layer
    ├── Fuel Event Detection
    ├── Device Health Intelligence
    ├── Sensor Health Intelligence
    └── Device State Engine
````

---

# Technology Stack

- Rust
- Axum
- SQLx
- PostgreSQL

---

# Current Development Status

Implemented:

- modular ingestion architecture
- PostgreSQL persistence for fuel, GPS, vibration, and motion telemetry
- operational event generation
- leak/theft/refill detection
- suppression logic
- event APIs
- heartbeat endpoint
- device health status tracking
- configurable health thresholds
- automatic background device health refresh
- device health event history
- deterministic confidence scoring for fuel events
- confidence field on fuel event API responses

Pending:

- WebSocket broadcasting
- live dashboard
- ML-assisted anomaly scoring
- Redis/Kafka streaming
- industrial hardware integration

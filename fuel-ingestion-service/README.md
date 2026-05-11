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
```

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

The service now supports device operational health tracking.

Implemented:

- heartbeat ingestion
- payload seen tracking
- `last_seen_at`
- `last_payload_at`
- `last_heartbeat_at`
- device status classification
- device health event persistence
- configurable health thresholds

Device status values:

````text
ONLINE
STALE
OFFLINE
UNKNOWN

Status logic is based on last_seen_at.

Configuration:

DEVICE_STALE_AFTER_SECONDS=120
DEVICE_OFFLINE_AFTER_SECONDS=600

For development testing, smaller values can be used:

DEVICE_STALE_AFTER_SECONDS=5
DEVICE_OFFLINE_AFTER_SECONDS=15

Also add these under **Current API Endpoints**:

```md
## Device Heartbeat

```http
POST /api/heartbeat

Receives device heartbeat messages and marks the device as online.

Refresh Device Health
POST /api/devices/refresh-health

Reclassifies device health using the configured thresholds.

List Device Health Events
GET /api/device-health-events

Returns recent device health transitions such as:

ONLINE → STALE
STALE → OFFLINE
OFFLINE → ONLINE

Then update **Current Development Status → Implemented** with:

```md
- heartbeat endpoint
- device health status tracking
- configurable health thresholds
- device health event history

# Current API Endpoints

## Batch Ingestion

```http
POST /api/fuel-readings/batch
````

---

## List Fuel Events

```http
GET /api/fuel-events
```

Returns recent operational events as JSON.

---

# Current Architecture

```text
Device / Simulator
→ Offline Queue
→ Batch Synchronization
→ Axum API
→ PostgreSQL
→ Detection Engine
→ Fuel Events
```

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
- PostgreSQL persistence
- operational event generation
- leak/theft/refill detection
- suppression logic
- event APIs

Pending:

- WebSocket broadcasting
- live dashboard
- GPS-aware logic
- engine-state-aware logic
- ML-assisted anomaly scoring
- Redis/Kafka streaming
- industrial hardware integration

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

## Live Alerts WebSocket

```http
GET /ws/alerts
```

Provides live operational alert streaming for dashboards and monitoring systems.

---

## List Device State Events

````http
GET /api/device-state-events

Returns recent classified operational states such as:

MOVING
IDLE
PARKED

---

## List Alerts

```http
GET /api/alerts

Returns operationally escalated alerts generated from fuel event intelligence.




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

---

# Multi-Sensor Correlation Layer

The platform now supports deterministic multi-sensor operational correlation.

This layer does not only detect events.

It also evaluates whether detected operational events logically align with:

- operational device state
- movement conditions
- motion behavior
- telemetry quality context

The correlation layer helps distinguish between:

- operationally consistent events
- suspicious contradictions
- conflicting telemetry patterns
- insufficient operational context

Current correlation statuses:

```text
Consistent
Suspicious
Conflicting
Unknown

Current implemented examples:

THEFT + PARKED
→ Consistent

THEFT + IDLE
→ Consistent

REFILL + IDLE
→ Consistent

REFILL + MOVING
→ Conflicting

Each fuel event can now include:

confidence
correlation_status
correlation_reason

Example:

{
  "event_type": "THEFT",
  "confidence": "High",
  "correlation_status": "Consistent",
  "correlation_reason": "Fuel theft pattern aligns with parked stationary vehicle."
}

This provides the foundation for:

forensic operational reasoning
explainable anomaly detection
tamper investigation
operational evidence reconstruction
future ML-assisted behavioral intelligence

---

# Alert Rules Engine

The platform now supports deterministic operational alert escalation.

Not every detected fuel event automatically becomes an operational alert.

The Alert Rules Engine evaluates:

- event type
- confidence level
- operational correlation status

before deciding whether an alert should be escalated.

Current alert severities:

```text
Info
Warning
Critical

Current implemented examples:

THEFT
+ High confidence
+ Consistent operational correlation
====================================
Critical alert

REFILL
+ Medium confidence
+ Conflicting operational correlation
====================================
Warning alert

Alerts are persisted independently from raw fuel events.

This allows:

operational escalation
alert acknowledgment workflows
dashboard alert feeds
future SMS/email escalation
future WebSocket live notifications

Current alert fields:

alert_type
severity
reason
is_acknowledged
created_at

Current alert endpoint:

GET /api/alerts

Example response:

[
  {
    "alert_type": "THEFT",
    "severity": "Critical",
    "reason": "High-confidence theft with operationally consistent correlation."
  }
]

This layer provides the operational foundation for:

incident escalation
fleet operations monitoring
forensic operational analysis
future enterprise alert workflows

# Live Operational Alert Streaming

The platform now supports real-time operational alert broadcasting using WebSockets.

Current implementation:

```text
fuel event detected
→ alert rule evaluation
→ alert persistence
→ immediate WebSocket broadcast
```

Current endpoint:

```http
GET /ws/alerts
```

Current capabilities:

- event-driven broadcasting
- zero polling architecture
- shared in-memory alert hub
- live operational alert feeds
- heartbeat keepalive messages
- real-time dashboard support

Heartbeat messages are periodically sent to connected clients to support unstable or intermittent network environments.

Example heartbeat:

```json
{
  "type": "heartbeat",
  "message": "alerts_ws_alive"
}
```

## WebSocket Recovery Synchronization

Dashboards can reconnect with a `since` timestamp to recover missed alerts.

Example:

```http
GET /ws/alerts?since=2026-05-18T19:30:00Z
```

Recovery flow:

```text
dashboard reconnects
→ backend loads alerts created after `since`
→ missed alerts are sent first
→ WebSocket then continues with live alerts
→ heartbeat messages continue
```

This protects dashboards from temporary network drops while keeping PostgreSQL as the source of truth.

## WebSocket Message Envelope Types

The WebSocket stream now uses typed message envelopes to distinguish between:

- historical recovery replay
- live operational alerts
- connection heartbeat events

Current message types:

### Recovery Alert

```json
{
  "type": "recovery_alert",
  "data": {
    "alert_type": "THEFT"
  }
}
```

### Live Alert

```json
{
  "type": "live_alert",
  "data": {
    "alert_type": "REFILL"
  }
}
```

### Heartbeat

```json
{
  "type": "heartbeat",
  "message": "alerts_ws_alive"
}
```

This structure allows frontend dashboards to safely route messages into:

- recovery synchronization flows
- real-time alert feeds
- connection health monitoring

## Alert Acknowledgment Workflow

The platform now supports operational alert acknowledgment workflows.

Current acknowledgment endpoint:

```http
PATCH /api/alerts/{alert_id}/acknowledge
```

Acknowledgment flow:

```text
operator receives live alert
→ operator acknowledges alert
→ alert state updates in PostgreSQL
→ acknowledgment broadcasts live to dashboards
```

This enables dashboards to distinguish between:

- active operational incidents
- acknowledged incidents
- handled operational alerts

Current acknowledgment WebSocket message:

```json
{
  "type": "alert_acknowledged",
  "data": {
    "alert_type": "THEFT",
    "is_acknowledged": true
  }
}
```

Operational acknowledgment events are broadcast through the same shared WebSocket operational stream used for live alerts.

This provides the foundation for:

- multi-operator operational coordination
- incident handling workflows
- future escalation management
- future role-based operational control

Operational design philosophy:

- PostgreSQL remains the source of truth
- WebSocket delivery is transient
- alerts are persisted before broadcast
- forensic reconstruction remains possible even if clients disconnect

Current implementation uses a single-process in-memory broadcaster.

Future scaling roadmap:

- Redis Pub/Sub
- Kafka/NATS streaming
- distributed event fanout
- enterprise escalation pipelines

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

→ Live Distribution Layer
├── In-Memory Alert Hub
├── WebSocket Alert Streaming
└── Heartbeat Keepalive

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
- multi-sensor operational correlation layer
- correlation status persistence
- correlation reasoning persistence
- forensic operational interpretation
- operational alert rules engine
- alert persistence
- alert severity classification
- alert API endpoint
- WebSocket broadcasting
- event-driven live alert streaming
- heartbeat keepalive support
- WebSocket operational feeds

Pending:

- live dashboard
- ML-assisted anomaly scoring
- Redis/Kafka streaming
- industrial hardware integration

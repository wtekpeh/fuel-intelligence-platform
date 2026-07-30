# Fuel Ingestion Service

## Platform Philosophy

The Sensor Intelligence Platform separates business management,
hardware management, and operational intelligence into distinct
layers.

Platform Administration

- Organizations
- Assets
- Device Inventory
- Provisioning
- Device Lifecycle

Operational Intelligence

- Telemetry Ingestion
- Event Detection
- Investigation
- Analytics
- Decision Support

This separation allows the platform to manage the complete
lifecycle of intelligent devices, from manufacturing and
inventory through deployment, telemetry processing, and
retirement, without coupling operational analytics to
hardware administration.

# ORBI Sensor Intelligence Architecture

## Core Design Principle

ORBI follows a simple architectural principle:

> Firmware acquires reality. The ORBI Platform interprets reality.
> For IMU-equipped ORBI devices, the firmware transmits raw accelerometer, gyroscope, and temperature measurements rather than interpreted motion states.

The backend is responsible for interpreting these measurements into:

- vibration_score
- motion_detected
- movement_confidence

These interpreted values are then used by the Device State Engine and future operational intelligence modules.

This architecture allows motion intelligence to evolve without requiring firmware updates.

Firmware is responsible for acquiring sensor measurements, timestamping telemetry, buffering data during communication failures, and reliably transmitting telemetry to the ORBI Platform.

The backend is responsible for:

- validating telemetry
- persisting raw telemetry
- resolving provisioned sensors
- loading applicable calibration
- applying calibration
- filtering measurements
- correlating data across sensors
- deriving operational behaviour
- detecting operational events
- generating alerts
- supporting investigation workflows
- producing analytics

This separation allows intelligence to evolve continuously without requiring firmware updates.

---

## Operational Intelligence Philosophy

The platform distinguishes three different classes of information.

Telemetry

Raw measurements received from physical hardware.

Examples:

- GPS coordinates
- Fuel level
- Accelerometer
- Gyroscope
- Temperature

Operational State

The backend continuously evaluates telemetry to determine the current
operational state of a device.

Examples:

- MOVING
- IDLE
- PARKED
- OFFLINE

Operational Events

Operational events represent meaningful transitions or inferred
intelligence derived from telemetry and operational state.

Examples:

- JOURNEY_STARTED
- VEHICLE_IDLING
- VEHICLE_PARKED
- ENTERED_ZONE
- EXITED_ZONE
- FUEL_THEFT
- REFILL
- LEAK

Telemetry is stored continuously.

Operational state is evaluated continuously.

Operational events are persisted only when meaningful changes occur.

This separation keeps investigation timelines concise while preserving
complete telemetry for replay, analytics, and forensic reconstruction.

## Telemetry Philosophy

Telemetry represents measurements collected from physical sensors.

Telemetry does not contain operational intelligence such as:

- Fuel Theft
- Fuel Refill
- Fuel Leak
- Device State
- Movement Classification
- Driver Behaviour
- Operational Alerts

Raw IMU measurements are treated as first-class telemetry and include:

- Acceleration (X, Y, Z)
- Angular Velocity (X, Y, Z)
- IMU Temperature

Like GPS and Fuel telemetry, these measurements contain no operational interpretation.

These are produced by the Operational Intelligence layer after telemetry has been processed.

---

## Telemetry Processing Pipeline

The platform processes telemetry using the following architecture:

Telemetry Packet
↓
Telemetry Router
↓
Canonical Telemetry Mapping
↓
Persist Raw Telemetry
↓
Resolve Provisioned Sensor
↓
Load Active Calibration
↓
Apply Calibration
↓
Telemetry Processing Pipeline
├── Filtering
├── IMU Interpretation
├── Rolling Motion Tracking
└── Processed Telemetry
↓
────────────────────────────────────
GPS Service
Fuel Service
Motion Service
Power Service
Health Service
────────────────────────────────────
↓
Operational Intelligence Engine
↓
Investigation Engine
↓
Analytics Intelligence

Each sensor service is responsible only for processing its own calibrated sensor domain.

Sensor services never resolve calibration directly. They receive measurements that have already passed through the telemetry processing pipeline.

The Intelligence Engine combines observations from multiple sensor services to produce operational events and alerts.

---

## Sensor Service Responsibilities

GPS Service

- GPS persistence
- Trip reconstruction
- Geofence intelligence
- Replay support

Fuel Service

- Fuel persistence
- Fuel event detection
- Leak detection
- Refill detection

Motion Service

Current capabilities:

- Raw IMU telemetry processing
- Canonical telemetry normalization
- Shared IMU interpretation
- Rolling per-device motion tracking
- MotionEvidence generation
- Vibration score derivation
- Motion detection
- Movement confidence estimation
- Motion ratio calculation
- Average movement confidence calculation
- Motion-aware device-state support
- Cross-request GPS movement tracking
- Previous-position retrieval from persisted telemetry
- Distance calculation in meters
- Estimated speed calculation (km/h)
- Operational transition detection
- Journey lifecycle detection
- Transition-only device-state persistence

Future capabilities:

- Driver behaviour analysis
- Motion event generation
- Crash detection
- Harsh manoeuvre detection

Power Service

- Battery monitoring
- Ignition monitoring
- Power diagnostics

Health Service

- Device health
- Sensor health
- Communication diagnostics
- Firmware diagnostics

---

## Intelligence Engine

The Intelligence Engine combines observations from multiple sensor services to produce operational intelligence.

Example:

```text
Fuel Drop
+
Vehicle Stationary
+
Inside Depot
=
Fuel Theft
```

Another example:

```text
High Acceleration
+
Rapid Steering Change
=
Harsh Cornering
```

This architecture allows new sensors and new intelligence capabilities to be added without redesigning the telemetry ingestion pipeline.

## Overview

The Fuel Ingestion Service is the operational intelligence backend for the Sensor Intelligence Platform.

It is responsible for:

- receiving sensor/device batches
- persisting sensor readings
- reconstructing historical timelines
- detecting operational fuel events
- supporting offline-safe synchronization
- exposing operational intelligence through APIs
- geospatial operational intelligence
- PostGIS spatial investigation support
- operational geofence intelligence

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
- raw IMU telemetry ingestion
  - accelerometer (X, Y, Z)
  - gyroscope (X, Y, Z)
  - IMU temperature
- backwards-compatible support for legacy vibration and motion fields

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

## Spatial Intelligence Layer

The backend now includes a PostGIS-backed spatial intelligence layer.

Current spatial intelligence capabilities:

- PostGIS polygon persistence
- GeoJSON operational geofence APIs
- telemetry position spatial checks
- replay-aware spatial intelligence
- operational zone intelligence
- backend ST_Contains geofence checks
- device-aware geofence assignment foundation

Current spatial intelligence workflow:

```text
Leaflet Draw
→ GeoJSON payload
→ Rust geofence APIs
→ PostgreSQL + PostGIS geometry storage
→ ST_Contains spatial intelligence
→ replay-aware operational investigation
```

Current geofence APIs:

```http
POST /api/geofences

GET /api/geofences/{organization_id}

POST /api/geofences/check-position
```

`````md
## Geofence Transition Events

The backend now supports automatic geofence transition detection during telemetry ingestion.

Current transition types:

````text
ENTERED_ZONE
EXITED_ZONE

Transition logic:

outside → inside = ENTERED_ZONE

inside → outside = EXITED_ZONE

inside → inside = no event

outside → outside = no event

Transition events are stored in:

geofence_transition_events

Current event fields:

id
organization_id
device_id
geofence_id
transition_type
latitude
longitude
recorded_at
detected_at
created_at

Geofence transition responses are enriched with:

geofence_name
geofence_type

Current APIs:

GET /api/geofence-transition-events

GET /api/geofence-transition-events?device_id={device_id}

Current ordering:

ORDER BY detected_at DESC

This allows operational dashboards to show the most recently detected geofence transitions first.

Current use cases:

depot entry detection
depot exit detection
operational zone monitoring
replay investigation
investigation timeline correlation
future geofence intelligence rules

---

### Location 3

Go all the way to the bottom.

Find:

```md
# Current Development Status

Implemented:

- PostGIS spatial intelligence layer
- GeoJSON geofence APIs
- backend geofence persistence
- operational polygon intelligence
- ST_Contains spatial checks
- replay-aware geofence intelligence
- device-aware geofence assignment foundation
- telemetry position geofence checks
- geofence transition detection
- geofence transition event persistence
- enriched geofence transition APIs
- device-filtered geofence transition queries
- investigation-ready geofence transition intelligence
- persisted GPS movement recovery
- cross-request movement continuity
- GPS distance estimation
- approximate speed estimation
- operational transition detection
- journey lifecycle intelligence
- transition-only operational state persistence
- investigation timeline optimization

---

### Location 4

Find:

```md
Planned spatial intelligence capabilities:

- depot entry/exit events
```
````

Remove:

```md
- depot entry/exit events
```

because we have already implemented it.

Leave the rest:

```md
- restricted-zone alerts
- dwell detection
- route corridor analysis
- unauthorized fueling detection
- theft outside depot intelligence
- replay spatial investigation workflows
- operational hotspot analysis
- future route-risk intelligence
```

---

Those are the only four places I would touch in the ingestion service README right now. That keeps the document accurate and avoids duplication.

Current geofence types:

```text
DEPOT
FUELING_STATION
RESTRICTED_ZONE
SAFE_CORRIDOR
CUSTOMER_SITE
```

Current spatial intelligence capabilities include:

- live telemetry geofence awareness
- replay geofence analysis
- operational zone status evaluation
- backend spatial filtering
- organization-wide geofence intelligence
- geofence transition detection
- geofence transition event persistence
- geofence transition event enrichment
- device-filtered geofence transition queries
- investigation-ready geofence transition timelines

Current architecture foundation:

```text
geofences
→ operational zone geometry

geofence_device_assignments
→ optional per-device inclusion/exclusion
```

Current spatial intelligence query foundation uses:

```sql
ST_Contains(
  geometry,
  ST_SetSRID(ST_MakePoint(longitude, latitude), 4326)
)
```

Important coordinate rule:

```text
PostGIS:
longitude, latitude

Leaflet:
latitude, longitude
```

Planned spatial intelligence capabilities:

- depot entry/exit events
- restricted-zone alerts
- dwell detection
- route corridor analysis
- unauthorized fueling detection
- theft outside depot intelligence
- replay spatial investigation workflows
- operational hotspot analysis
- future route-risk intelligence

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

# Analytics Intelligence

The backend now exposes dedicated analytics endpoints that provide
aggregated operational intelligence separate from raw operational
event streams.

Operational Endpoints

→ raw events
→ investigations
→ timeline reconstruction

Examples:

GET /api/device-health-events
GET /api/geofence-transition-events
GET /api/alerts

Analytics Endpoints

→ trend analysis
→ fleet intelligence
→ operational scoring

Current analytics APIs:

GET /api/analytics/alert-trends
GET /api/analytics/geofence-activity
GET /api/analytics/device-health-trends
GET /api/analytics/geofence-utilization

Supported analytics periods:

days=7
days=30
days=90

## Analytics APIs

Document:

GET /api/analytics/alert-trends

Returns:

total alerts
alert breakdown
daily alert trend
GET /api/analytics/geofence-activity

Returns:

entries per day
exits per day
zone activity trends
GET /api/analytics/device-health-trends

Returns:

offline events
stale events
recovery events
reliability issue counts
most unreliable devices
GET /api/analytics/geofence-utilization

Returns:

zone visit counts
most active zones
geofence utilization ranking

# Current API Endpoints

## Organization Operational Overview

````http
GET /api/organizations/overview

Returns operational organization summaries for the frontend landing page.

Current response fields:

organization_id
organization_name
industry
asset_count
device_count
online_device_count
stale_device_count
offline_device_count
open_alert_count

Example response:

[
  {
    "organization_name": "Demo Transport Company",
    "industry": "Transport",
    "asset_count": 1,
    "device_count": 9,
    "online_device_count": 1,
    "offline_device_count": 8,
    "open_alert_count": 17
  }
]

This endpoint provides the operational landing-page foundation for:

organizations
→ assets
→ devices
→ operational status
→ live dashboard access
Platform Hierarchy

Current operational hierarchy:

Organization
→ Assets
→ Devices
→ Sensors

Example:

Mining Company
→ Excavator
→ Fuel Monitoring Device
→ Fuel Sensor / GPS / Vibration Sensor

This structure enables:

multi-company support
fleet segmentation
mining/construction deployments
device grouping
future role-based access control
operational dashboard routing

This is an important architectural milestone because the platform is now transitioning from:

```text
single operational dashboard

to:

multi-organization operational intelligence platform

based on the existing backend hierarchy already present in the database.

## Batch Ingestion

```http
POST /api/fuel-readings/batch
````
`````

## Organization Fleet Overview

````http
GET /api/organizations/{organization_id}/fleet-overview

Returns the assets, devices, sensors, device status, and open alert counts for a selected organization.

This endpoint powers the second-level frontend page after selecting an organization from the landing page.

Current response fields:

asset_id
asset_name
asset_type
capacity_litres
device_id
device_code
device_status
last_seen_at
sensor_count
sensor_types
open_alert_count

Example response:

[
  {
    "asset_name": "Demo Fuel Truck",
    "asset_type": "truck",
    "capacity_litres": 200.0,
    "device_code": "DEV_CORR_TEST_004",
    "device_status": "ONLINE",
    "sensor_count": 1,
    "sensor_types": ["fuel_level"],
    "open_alert_count": 13
  }
]

Frontend flow supported by this endpoint:

Organization landing page
→ select organization
→ view fleet/assets/devices/sensors
→ select device
→ open operational dashboard

This endpoint strengthens the platform structure by making the dashboard organization-aware and device-aware instead of being a single global screen.


Your README already has the organization overview section, so this fleet overview belongs directly after it. :contentReference[oaicite:0]{index=0}

---

## List Fuel Events

```http
GET /api/fuel-events
````

Fuel event responses include:

- severity
- confidence
- message
- fuel difference
- delayed detection metadata
  Returns recent operational events as JSON.

---

## Device-Aware Operational Dashboard Filtering

The platform now supports device-scoped operational dashboard routing.

Previously, dashboard operational feeds were globally aggregated.

The backend now supports optional `device_id` filtering across operational intelligence endpoints so that frontend dashboards can operate on a selected device context.

Current supported filtered endpoints:

```http
GET /api/alerts?device_id={device_id}

GET /api/fuel-readings/recent?device_id={device_id}

GET /api/device-health-events?device_id={device_id}

GET /api/fuel-events?device_id={device_id}

GET /api/device-state-events?device_id={device_id}

GET /api/sensor-health-events?device_id={device_id}
```

This enables the frontend hierarchy:

```text
Organization
→ Fleet Overview
→ Device Selection
→ Device-Specific Operational Dashboard
```

This architectural step is important because the platform is transitioning from:

```text
single global telemetry dashboard

to:

multi-organization
multi-fleet
multi-device
operational intelligence routing
```

The backend now supports isolated operational investigation per physical device.

This foundation will support future features such as:

- operational replay
- investigation timelines
- route reconstruction
- forensic fuel analysis
- predictive maintenance
- sensor diagnostics
- ML-assisted analytics
- device-specific operational dashboards

WebSocket live alert filtering is also now device-aware at the frontend layer using `device_id` carried in alert payloads.

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


## Acknowledge Alert

```http
PATCH /api/alerts/{alert_id}/acknowledge
```

Marks an operational alert as acknowledged.

This updates:

```text
is_acknowledged = true
status = ACKNOWLEDGED
```

---

## Resolve Alert

```http
PATCH /api/alerts/{alert_id}/resolve
```

Marks an operational alert as resolved.

This updates:

```text
is_acknowledged = true
status = RESOLVED
```


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
- Registered device validation
- Production device provisioning workflow
- Unknown device rejection

---

# Device State Engine

The service now supports operational device state classification using incoming telemetry.

Current implemented states:

MOVING
IDLE
PARKED
OFFLINE
UNKNOWN

The Device State Engine currently uses:

- backend IMU interpretation
- rolling MotionEvidence
- device health status
- previous GPS location
- current GPS location

The Telemetry Pipeline derives:

- vibration_score
- motion_detected
- movement_confidence
- motion_ratio
- average_movement_confidence

These interpreted values are encapsulated within a shared MotionEvidence model before being combined with GPS movement to classify operational behaviour.

The platform currently maintains two classification paths:

- Legacy IMU-based classification
- MotionEvidence-aware classification

The MotionEvidence classifier is currently used for validation alongside the legacy classifier while the platform transitions to the new motion-aware architecture.

The engine now supports GPS-aware movement classification by comparing previous and current coordinates during telemetry ingestion.

Movement detection now uses estimated GPS distance in meters instead of raw coordinate subtraction.

Example correlations:

high vibration

- # motion detected
  MOVING
  low vibration
- # no motion
  PARKED

Current capabilities:

Current motion intelligence capabilities include:

- rolling motion accumulation
- motion evidence generation
- movement confidence aggregation
- motion ratio calculation
- shared processed telemetry
- dual-classifier validation
- persisted previous-position recovery
- cross-request movement continuity
- batch-aware movement chaining
- transition-aware operational state persistence

Device operational states are evaluated for every telemetry reading.

However, device_state_events are only persisted when the operational
state changes.

Example:

MOVING
↓

IDLE
↓

PARKED

Repeated classifications such as:

MOVING
MOVING
MOVING

are intentionally not stored because they do not represent operational
events.

Raw telemetry remains fully available through sensor_readings for
historical replay and analytics.

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

````text
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
````

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

## Alert Lifecycle States

Alerts now support a basic operational lifecycle:

````text
OPEN
→ ACKNOWLEDGED
→ RESOLVED

Lifecycle meaning:

OPEN: alert has been created and still requires attention
ACKNOWLEDGED: operator has seen and accepted the alert
RESOLVED: incident has been handled or closed

Resolved alerts are also treated as acknowledged.

Current resolve endpoint:

PATCH /api/alerts/{alert_id}/resolve

Resolve flow:

operator investigates alert
→ operator resolves incident
→ alert status updates in PostgreSQL
→ resolution update broadcasts live to dashboards

This README update is important because alerts now have operational state, not just a boolean ackno

## Telemetry Detection Configuration

Current configurable telemetry intelligence settings:

```env
DEFAULT_TANK_CAPACITY_LITRES=200
MAX_ALLOWED_FUEL_JUMP_LITRES=80
FUEL_ROLLING_WINDOW_SIZE=5
FUEL_IQR_MULTIPLIER=1.5

# Current Architecture

```text
Platform Management
────────────────────────────────────

Organizations
        ↓
Assets
        ↓
Devices
        ↓
Device Models
        ↓
Hardware Profiles
        ↓
Provisioned Sensors

────────────────────────────────────

Device / Simulator
        ↓
Offline Queue
        ↓
Batch Synchronization
        ↓
Axum API
        ↓
Registered Device Validation
        ↓
PostgreSQL

────────────────────────────────────

Operational Intelligence Layer
        ├── IMU Interpretation
        ├── Device State Engine
        ├── Fuel Event Detection
        ├── Device Health Intelligence
        ├── Sensor Health Intelligence
        ├── Geofence Intelligence
        └── Analytics Intelligence

────────────────────────────────────

Live Distribution Layer
        ├── Alert Hub
        ├── WebSocket Streaming
        └── Heartbeat Keepalive


→ Operational Intelligence Layer
    ├── Fuel Event Detection
    ├── Device Health Intelligence
    ├── Sensor Health Intelligence
    └── Device State Engine
    ├── Geofence Intelligence
    ├── PostGIS Spatial Checks
    └── Replay Spatial Investigation

  Analytics Intelligence Layer
    ├── Alert Trends
    ├── Device Reliability Analytics
    ├── Geofence Activity Analytics
    └── Geofence Utilization Analytics
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
- PostgreSQL persistence for fuel telemetry
- PostgreSQL persistence for GPS telemetry
- raw IMU telemetry ingestion
- canonical telemetry normalization
- shared telemetry processing pipeline
- shared ProcessedTelemetry model
- backend IMU interpretation
- rolling MotionEvidence generation
- per-device rolling motion tracking
- motion-aware device-state validation
- device-state integration using interpreted IMU measurements
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
- PostGIS spatial intelligence layer
- GeoJSON geofence APIs
- backend geofence persistence
- operational polygon intelligence
- ST_Contains spatial checks
- replay-aware geofence intelligence
- device-aware geofence assignment foundation
- telemetry position geofence checks
- organization operational overview API
- organization fleet overview API
- device-scoped operational filtering
- alert trend analytics
- geofence activity analytics
- device reliability analytics
- geofence utilization analytics
- analytics aggregation endpoints
- shared telemetry processing architecture
- reusable motion intelligence foundation
- rolling motion evidence pipeline

# Device & Hardware Management

The platform now includes a dedicated Device & Hardware Management
layer that is separate from the Operational Intelligence layer.

This provides the production foundation for onboarding physical
tracking hardware independently of telemetry ingestion.

Current capabilities:

- Organization CRUD
- Asset CRUD
- Device Model management
- Hardware Profile management
- Hardware Profile sensor definitions
- Device deactivation
- Automatic sensor provisioning
- Device listing
- Device sensor listing
- ORBI Device Inventory module
- Dedicated inventory repository
- Inventory creation APIs
- Inventory listing APIs
- Inventory verification by Device Code
- Inventory retrieval by ID
- Inventory lifecycle management
- Inventory status updates
- Manufacturing-first provisioning foundation
- Installed Devices
- Provision device
- Reassign device
- Deactivate device
- Retire device

Current hardware profiles:

GPS_ONLY
GPS_CONTROL
FUEL_INTELLIGENCE
FULL_INTELLIGENCE

Current device models:

ORBI-GPS-LITE
ORBI-GPS-CONTROL
ORBI-FUEL-KIT
ORBI-FULL-KIT

Current onboarding workflow:

Organization
↓
Asset
↓
Device Model
↓
Automatic Hardware Profile Selection
↓
Review Provisioning
↓
Verify Device Code
↓
Provision Device
↓
Automatic Sensor Provisioning

## ORBI Product Catalogue

| Product                    | Hardware Profile  | Capabilities                                                           |
| -------------------------- | ----------------- | ---------------------------------------------------------------------- |
| ORBI GPS Lite              | GPS_ONLY          | GPS Tracking                                                           |
| ORBI GPS Control Kit       | GPS_CONTROL       | GPS Tracking, Remote Kill Switch                                       |
| ORBI Fuel Intelligence Kit | FUEL_INTELLIGENCE | Fuel Monitoring, GPS Tracking, Vibration Detection                     |
| ORBI Full Intelligence Kit | FULL_INTELLIGENCE | Fuel Monitoring, GPS Tracking, Vibration Detection, Remote Kill Switch |

# ORBI Device Inventory

The platform now includes a fully implemented ORBI Device Inventory
module that manages physical ORBI hardware before it is provisioned
to customers.

Unlike Operational Intelligence, Device Inventory represents the
manufacturing and inventory lifecycle of ORBI hardware.

Current implemented capabilities:

- Create inventory device
- List inventory devices
- Retrieve inventory device by ID
- Verify inventory device by Device Code
- Update inventory lifecycle status

Current inventory lifecycle:

ASSEMBLED
↓
PROGRAMMED
↓
TESTED
↓
READY_FOR_DEPLOYMENT
↓
PROVISIONED
↓
RETIRED

Each inventory record contains:

- Device Code
- Serial Number
- IMEI
- Device Model
- Hardware Profile
- Firmware Version
- Production Batch
- Inventory Status
- Quality Test Status
- Notes

Platform installers never create devices.

Instead the workflow is:

```text
PCB Assembly
↓
Firmware Programming
↓
Quality Testing
↓
Create Inventory Record
↓
Inventory Management
↓
Installer Verification
↓
Provision From Inventory
↓
Operational Device
↓
Telemetry Ingestion
↓
Operational Intelligence
```

Current Inventory APIs:

POST /api/device-inventory

GET /api/device-inventory

GET /api/device-inventory/{inventory_device_id}

GET /api/device-inventory/verify/{device_code}

PATCH /api/device-inventory/{inventory_device_id}/status

POST /api/devices/provision-from-inventory

## Production Provisioning Model

The platform now separates **Platform Provisioning** from
**Operational Telemetry Ingestion**.

Business relationships are created only through Platform
Management.

Platform Administration
────────────────────────────────

Organizations
↓
Assets
↓
ORBI Device Inventory
↓
Provisioning
↓
Provisioned Devices
↓
Telemetry Ingestion
↓
Operational Intelligence
↓
Analytics

Only after a device has been provisioned will the backend accept:

- telemetry batches
- heartbeats

Unknown devices are rejected and are **not** automatically created.

This transition marks the move from development bootstrap behaviour
to production-ready device provisioning.

Automatic provisioning examples:

GPS_ONLY
→ GPS

GPS_CONTROL
→ GPS
→ Kill Switch

FUEL_INTELLIGENCE
→ Fuel
→ GPS
→ Vibration

FULL_INTELLIGENCE
→ Fuel
→ GPS
→ Vibration
→ Kill Switch

Current Platform Management APIs:

# Organizations

POST /api/organizations
PATCH /api/organizations/{organization_id}
DELETE /api/organizations/{organization_id}

# Assets

POST /api/assets
PATCH /api/assets/{asset_id}
DELETE /api/assets/{asset_id}

# Device Models

GET /api/device-models

# Hardware Profiles

GET /api/hardware-profiles
GET /api/hardware-profiles/{hardware_profile_id}/sensors

# Devices

POST /api/devices
POST /api/devices/provision-from-inventory

GET /api/devices

PATCH /api/devices/{device_id}

PATCH /api/devices/{device_id}/assign-asset

DELETE /api/devices/{device_id}

GET /api/devices/{device_id}/sensors

# ORBI Device Inventory

POST /api/device-inventory

GET /api/device-inventory

GET /api/device-inventory/{inventory_device_id}

GET /api/device-inventory/verify/{device_code}

PATCH /api/device-inventory/{inventory_device_id}/status

## Ingestion Validation

Telemetry ingestion now validates every incoming device against the
registered Platform Management inventory.

Current flow:

Telemetry
↓
Lookup Registered Device
↓
Found
↓
Accept

Unknown
↓
Reject

Heartbeat processing follows the same validation workflow.

## Device Lifecycle

The platform separates operational device health from platform lifecycle.

Operational Health

ONLINE
STALE
OFFLINE
UNKNOWN

Platform Lifecycle

is_active = true
is_active = false

This allows a device to be operationally offline while still being an
active provisioned device, or to be administratively deactivated
without affecting the operational health model.

This guarantees that operational telemetry can only originate from
devices that have been provisioned through Platform Management.

This architecture separates Platform Management from Operational
Intelligence, allowing the ingestion engine to process only the
telemetry supported by the registered hardware profile.

Future hardware support includes:

LilyGO devices

GPS-only deployments

GPS + Kill Switch deployments

Fuel Intelligence deployments

Full Intelligence deployments

Custom PCB hardware

Additional hardware profiles without database redesign

# Modular Sensor Management Architecture

## Architectural Decision

ORBI devices are modular.

A provisioned ORBI device may contain different combinations of:

- GPS modules
- IMU or vibration modules
- Fuel-level sensors
- Kill-switch modules
- Power and ignition sensors
- Temperature sensors
- Payload sensors
- Future industrial sensors

Sensors of the same functional type may also come from different
manufacturers, use different protocols, expose different registers,
and require different installation-specific calibration.

For this reason, ORBI does not treat a sensor type such as `FUEL`,
`GPS`, or `VIBRATION` as a complete physical sensor definition.

The platform distinguishes between:

- Hardware Profile Sensor Capability
- Sensor Profile
- Provisioned Sensor Instance
- Installation-Specific Sensor Calibration

---

## Core Platform Hierarchy

The long-term Platform Management hierarchy is:

Organization
↓
Asset
↓
Provisioned Device
↓
Hardware Profile
↓
Sensor Capability / Sensor Slot
↓
Provisioned Sensor Instance
↓
Sensor Profile
↓
Installation-Specific Calibration
↓
Operational Intelligence

Each level has a separate responsibility.

Hardware Profile

A Hardware Profile defines the capabilities expected from a device
configuration.

Examples:

GPS_ONLY
→ GPS

GPS_CONTROL
→ GPS
→ Kill Switch

FUEL_INTELLIGENCE
→ Fuel
→ GPS
→ Vibration

FULL_INTELLIGENCE
→ Fuel
→ GPS
→ Vibration
→ Kill Switch

A Hardware Profile defines what sensor capabilities should exist.

It does not permanently identify the exact physical sensor model
installed on a particular asset.

Sensor Capability / Sensor Slot

A sensor capability represents a functional position supported by a
Hardware Profile.

Examples:

GPS
VIBRATION
FUEL
KILL_SWITCH
TEMPERATURE
PAYLOAD
POWER

A sensor slot may later contain different supported physical sensors.

Example:

Fuel Sensor Slot
├── KUM Ultrasonic Fuel Sensor
├── Escort Fuel Sensor
├── Omnicomm Fuel Sensor
├── Technoton Fuel Sensor
└── Future ORBI Fuel Sensor

This allows ORBI hardware to remain modular without changing the
Operational Intelligence layer whenever a different sensor brand or
protocol is introduced.

Sensor Profile

A Sensor Profile describes a supported physical sensor model.

A Sensor Profile may define:

sensor type
manufacturer
model
protocol
communication interface
unit
register mappings
scaling rules
supported measurements
default interpretation settings
default calibration values
compatible sensor adapters

Examples:

MPU6050 IMU Profile
KUM Ultrasonic Fuel Sensor Profile
ORBI GNSS Profile
Future CAN/J1939 Payload Sensor Profile

Sensor Profile values provide reusable defaults.

They do not replace calibration for a specific installation.

Provisioned Sensor Instance

A Provisioned Sensor Instance represents an actual sensor module
attached to a particular provisioned ORBI device.

It may contain:

device ID
sensor profile ID
sensor capability or slot
sensor code
sensor serial number
interface or port
mounting position
installation date
operational status
calibration status

This entity allows sensors to be independently:

installed
configured
calibrated
replaced
deactivated
diagnosed

A physical sensor may therefore be replaced without replacing the
asset or the main ORBI device.

Installation-Specific Sensor Calibration

Calibration belongs to the provisioned sensor instance because sensor
behaviour depends on the exact installation.

Examples of factors that can affect calibration include:

vehicle model
engine condition
engine size
sensor mounting position
sensor orientation
mounting tightness
vehicle age
tank shape
tank dimensions
environmental conditions
physical sensor variation

Two vehicles of the same model may therefore use the same Sensor
Profile while requiring different calibration values.

Example IMU calibration:

{
"engine_off_average": 0.34,
"engine_idle_average": 0.72,
"idle_vibration_threshold": 0.53,
"moving_vibration_threshold": 2.0
}

Example ultrasonic fuel-sensor calibration:

{
"empty_distance_cm": 145.0,
"full_distance_cm": 12.0,
"tank_capacity_litres": 600.0,
"mounting_offset_cm": 2.5
}

Different sensor types may therefore use different calibration
structures.

Calibration Resolution Order

Operational Intelligence resolves configuration in the following order:

Active installation-specific sensor calibration
↓
Sensor Profile default
↓
Platform fallback

The platform fallback exists only to keep processing safe when no
profile or installation-specific value has been configured.

Installation-specific calibration has the highest priority.

Domain Ownership
ORBI Provision Utility

The independent orbi-provision utility owns factory identity
programming.

Its responsibilities are:

generate immutable device identities
program identities into ESP32 flash
verify identity integrity
protect existing identities from accidental overwrite

It does not manage:

organizations
assets
installed sensors
sensor profiles
sensor calibration
operational telemetry
Platform Management

Platform Management owns:

organizations
assets
device models
hardware profiles
sensor capabilities
sensor profiles
device inventory
device provisioning
provisioned sensor instances
installation records
calibration values
calibration history
sensor replacement
deployment activation

Platform Management is the only layer that creates or changes
installation and calibration configuration.

Operational Intelligence

Operational Intelligence owns:

telemetry ingestion
canonical telemetry mapping
sensor interpretation
motion classification
fuel-event detection
operational-state classification
multi-sensor correlation
alerts
investigation
replay
analytics

Operational Intelligence reads active sensor configuration and
calibration but does not own the calibration workflow.

Database access must remain outside pure classification functions.

A classifier should receive resolved configuration values as
arguments rather than loading them directly from PostgreSQL.

Shared Database Strategy

The current ORBI backend uses one PostgreSQL/PostGIS database.

Platform Management and Operational Intelligence remain separate
logical domains within the same Rust application and database.

Table ownership rules are:

Platform Management
→ creates and updates device, sensor, profile, installation and
calibration records

Operational Intelligence
→ reads configuration records and creates telemetry, state, event,
alert and analytics records

This approach avoids unnecessary distributed-system complexity while
preserving clear boundaries.

The architecture may later evolve toward service-owned read models and
configuration-change events when scale requires it.

Sensor Adapter Relationship

The Sensor Adapter Layer translates vendor-specific sensor communication
into ORBI's normalized telemetry model.

The complete future flow is:

Physical Sensor
↓
Provisioned Sensor Instance
↓
Sensor Profile
↓
Sensor Adapter
↓
Raw Vendor Measurement
↓
Canonical ORBI Telemetry
↓
Installation-Specific Calibration
↓
Operational Intelligence

Examples of supported communication technologies may include:

Modbus RTU
RS485
CAN/J1939
UART
I2C
SPI
MQTT
LoRaWAN
vendor-specific protocols

Operational Intelligence must consume normalized telemetry rather than
vendor-specific register structures.

Installation and Activation Lifecycle

The target deployment lifecycle is:

Manufacture Device
↓
Program Immutable Identity
↓
Create Inventory Record
↓
Progress Inventory Lifecycle
↓
Provision Device to Asset
↓
Automatically Create Expected Sensor Instances
↓
Confirm Physical Sensor Installation
↓
Apply Sensor Profiles
↓
Calibrate Required Sensors
↓
Validate Sensor Health
↓
Activate Deployment
↓
Accept Operational Telemetry

Initial provisioning may create expected sensor instances from the
selected Hardware Profile.

Installation and calibration then confirm the physical modules and
installation-specific values.

Incremental Implementation Strategy

This architecture will be implemented incrementally.

The platform will not build every future sensor-management feature
before current hardware integration continues.

Initial implementation:

Provisioned Vibration Sensor
↓
Active IMU Calibration
↓
Resolved Idle Vibration Threshold
↓
Motion Classification

The next validation will extend the same architecture to:

Provisioned KUM Fuel Sensor
↓
KUM Sensor Profile
↓
Tank and Distance Calibration
↓
Normalized Fuel Telemetry
↓
Fuel Intelligence

Future capabilities such as guided calibration sessions, sensor
replacement workflows, deployment profiles and adapter-management
interfaces will be introduced when required.

Locked Architectural Principles

The following decisions are now locked:

ORBI devices are modular.
Hardware Profiles define expected sensor capabilities.
Sensor Profiles define reusable physical sensor behaviour and defaults.
Provisioned Sensor Instances represent actual installed modules.
Calibration belongs to the provisioned sensor instance.
Installation calibration overrides Sensor Profile defaults.
Sensor Profile defaults override platform fallback values.
Platform Management owns calibration writes.
Operational Intelligence consumes resolved calibration.
Pure classification functions do not access the database directly.
Vendor-specific sensor protocols are isolated behind Sensor Adapters.
The platform will use one PostgreSQL/PostGIS database for the current stage.
The architecture will be implemented incrementally rather than through a large upfront rewrite.
KUM fuel-sensor integration will follow the initial IMU calibration foundation.

# Platform Lifecycle Architecture

The ORBI platform manages several independent business lifecycles.

Although they interact with one another, each lifecycle has a different
purpose, owner, and progression.

Keeping these lifecycles independent allows the platform to evolve
without coupling manufacturing, deployment, operational intelligence,
or maintenance into a single workflow.

---

# Manufacturing Lifecycle

The Manufacturing Lifecycle represents the creation of a physical ORBI
device before it is assigned to a customer.

Current lifecycle:

```text
ASSEMBLED
        ↓
PROGRAMMED
        ↓
TESTED
        ↓
READY_FOR_DEPLOYMENT
        ↓
INVENTORY
        ↓
PROVISIONED
        ↓
RETIRED
```

Manufacturing owns:

- PCB assembly
- immutable device identity
- firmware programming
- production testing
- quality assurance
- inventory registration

Manufacturing ends once a device is provisioned to an asset.

Operational Intelligence is not involved in this lifecycle.

---

# Device Lifecycle

The Device Lifecycle represents the administrative state of a provisioned
ORBI device.

```text
PROVISIONED
        ↓
ASSIGNED
        ↓
ACTIVE
        ↓
SUSPENDED
        ↓
DEACTIVATED
        ↓
RETIRED
```

Platform Management owns this lifecycle.

Examples include:

- assigning a device to an asset
- moving a device to another asset
- suspending a deployment
- retiring obsolete hardware

A device may remain ACTIVE while individual sensors are replaced or
recalibrated.

---

# Sensor Lifecycle

Each physical sensor attached to an ORBI device has its own independent
lifecycle.

```text
SUPPORTED
        ↓
EXPECTED
        ↓
INSTALLED
        ↓
DETECTED
        ↓
CONFIGURED
        ↓
CALIBRATED
        ↓
VALIDATED
        ↓
ACTIVE
        ↓
MAINTENANCE
        ↓
RECALIBRATED
        ↓
REPLACED
        ↓
RETIRED
```

Definitions:

SUPPORTED

The platform contains a Sensor Profile describing a supported sensor
model.

EXPECTED

The selected Hardware Profile expects this sensor capability to exist.

INSTALLED

A physical sensor has been connected to the provisioned device.

DETECTED

Communication with the sensor has been verified.

Examples:

- Modbus responding
- GPS producing fixes
- IMU transmitting data

CONFIGURED

Communication settings have been confirmed.

Examples:

- Modbus address
- baud rate
- communication protocol
- CAN identifiers

CALIBRATED

Installation-specific calibration values have been recorded.

Examples:

- tank dimensions
- IMU vibration thresholds
- antenna offsets

VALIDATED

The calibration has been verified using real operational testing.

Examples:

- parked classification verified
- idle classification verified
- moving classification verified
- fuel readings verified

ACTIVE

The sensor is fully trusted for Operational Intelligence.

MAINTENANCE

Maintenance is in progress.

Telemetry may still arrive but calibration or configuration may require
review.

RECALIBRATED

A newer calibration supersedes the previous active calibration while
retaining calibration history.

REPLACED

A different physical sensor replaces the previous sensor instance.

Historical telemetry remains associated with the original sensor.

RETIRED

The sensor is permanently removed from service.

---

# Deployment Lifecycle

The Deployment Lifecycle represents the readiness of an entire ORBI
installation.

Unlike Device or Sensor Lifecycles, it evaluates the deployment as a
complete operational system.

```text
DEVICE PROVISIONED
        ↓
SENSORS INSTALLED
        ↓
SENSORS CONFIGURED
        ↓
SENSORS CALIBRATED
        ↓
DEPLOYMENT VALIDATED
        ↓
ACTIVATED
        ↓
LIVE OPERATION
        ↓
MAINTENANCE
        ↓
SUSPENDED
        ↓
DEACTIVATED
```

Deployment validation confirms that all required hardware is operating
correctly before operational telemetry is trusted.

Examples of deployment validation:

- GPS fix acquired
- IMU operational
- Fuel sensor responding
- Communication verified
- Calibration completed

Only validated deployments should transition to Active Operation.

---

# Operational Lifecycle

Operational Intelligence evaluates telemetry independently of Platform
Management.

Operational states are inferred continuously from telemetry rather than
being manually assigned.

Example operational states include:

```text
UNKNOWN
ONLINE
OFFLINE

PARKED
IDLE
MOVING

NORMAL
WARNING
CRITICAL
```

Operational Intelligence owns:

- telemetry interpretation
- state classification
- alert generation
- operational events
- analytics
- investigation

Operational state changes do not modify Platform Management lifecycles.

---

# Lifecycle Independence

Each lifecycle answers a different question.

Manufacturing Lifecycle

"Has this hardware been built?"

Device Lifecycle

"Is this device administratively deployed?"

Sensor Lifecycle

"Is this physical sensor ready for operational use?"

Deployment Lifecycle

"Is this installation ready for production?"

Operational Lifecycle

"What is happening right now?"

Because these questions are independent, changes in one lifecycle do not
necessarily affect another.

Examples:

Replacing a fuel sensor changes the Sensor Lifecycle without replacing
the device.

Suspending a deployment does not erase historical operational events.

A parked vehicle may still be an Active deployment.

A device may be Offline while remaining administratively Active.

---

# Architectural Principle

Platform Management owns configuration.

Operational Intelligence owns interpretation.

Manufacturing owns identity.

Deployment confirms readiness.

Sensors remain modular.

Operational Intelligence consumes resolved configuration but never owns
installation or calibration workflows.

# Next Platform Milestones

Platform Foundation ✅
↓
Motion Intelligence Validation ✅
↓
Cross-Request Movement Intelligence ✅
↓
Transition-Only Investigation Timeline ✅
↓
Modular Sensor Architecture ✅
↓
Calibration Storage Foundation ✅
↓
Telemetry Calibration Pipeline ← Current
↓
KUM Fuel Sensor Integration
↓
Fuel Intelligence Validation
↓
Sensor Adapter Layer
↓
Fuel Intelligence Validation
↓
Sensor Adapter Layer
↓
Firmware Management
↓
Frontend Operational Intelligence Enhancements
↓
Production ORBI Hardware
↓
Production Custom ORBI PCB
↓
Fleet Intelligence Expansion
↓
Generator Intelligence
↓
Cold Chain Intelligence
↓
Payload Intelligence
↓
Energy Monitoring Platform

### Sensor-Agnostic Hardware Strategy

The platform is designed to be hardware and sensor agnostic.

Future hardware integration will follow this architecture:

```text
Physical Sensor
        ↓
Sensor Adapter
        ↓
Normalized Telemetry
        ↓
Sensor Profile
        ↓
Hardware Profile
        ↓
Operational Intelligence
```

This architecture allows multiple hardware vendors and communication
protocols (Modbus, RS485, CAN/J1939, MQTT, LoRaWAN, custom protocols,
and future ORBI hardware) to be supported without changing the
Operational Intelligence layer.

Platform Management remains responsible for provisioning business
relationships:

Organization
↓
Asset
↓
Device
↓
Hardware Profile

Operational Intelligence consumes only normalized telemetry from
registered devices.

The Sensor Adapter Layer will allow the platform to support multiple
hardware vendors and communication protocols while maintaining a
single normalized telemetry model.

Pending:

- ML-assisted anomaly scoring
- Redis/Kafka streaming
- industrial hardware integration

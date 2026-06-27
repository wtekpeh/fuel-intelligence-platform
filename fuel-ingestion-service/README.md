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
````

Then find the spatial intelligence section:

```md
- PostGIS spatial intelligence layer
- GeoJSON geofence APIs
- backend geofence persistence
- operational polygon intelligence
- ST_Contains spatial checks
- replay-aware geofence intelligence
- device-aware geofence assignment foundation
- telemetry position geofence checks
```

Append immediately after it:

```md
- geofence transition detection
- geofence transition event persistence
- enriched geofence transition APIs
- device-filtered geofence transition queries
- investigation-ready geofence transition intelligence
```

---

### Location 4

Find:

```md
Planned spatial intelligence capabilities:

- depot entry/exit events
```

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

## Alert Lifecycle States

Alerts now support a basic operational lifecycle:

```text
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
        ├── Fuel Event Detection
        ├── Device Health Intelligence
        ├── Sensor Health Intelligence
        ├── Device State Engine
        ├── Geofence Intelligence
        ├── Replay Investigation
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
- Device registration
- Device update
- Device reassignment
- Device deactivation
- Automatic sensor provisioning
- Device listing
- Device sensor listing

Current hardware profiles:

```text
GPS_ONLY
FUEL_FULL
```

Current device models:

ORBI-A100
ORBI-GPS-LITE
ORBI-FULL-KIT

Current onboarding workflow:

Create Organization
↓
Create Asset
↓
Select Device Model
↓
Select Hardware Profile
↓
Register Device
↓
Automatically Provision Sensors

## Production Provisioning Model

The platform now separates **Platform Provisioning** from
**Operational Telemetry Ingestion**.

Business relationships are created only through Platform
Management.

Organization
↓
Asset
↓
Device
↓
Device Model
↓
Hardware Profile
↓
Provisioned Sensors

Only after a device has been provisioned will the backend accept:

- telemetry batches
- heartbeats

Unknown devices are rejected and are **not** automatically created.

This transition marks the move from development bootstrap behaviour
to production-ready device provisioning.

Automatic provisioning examples:

```text
GPS_ONLY
→ GPS

FUEL_FULL
→ Fuel
→ GPS
→ Vibration
```

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
GET /api/devices
PATCH /api/devices/{device_id}
PATCH /api/devices/{device_id}/assign-asset
DELETE /api/devices/{device_id}
GET /api/devices/{device_id}/sensors

## Ingestion Validation

Telemetry ingestion now validates every incoming device against the
registered Platform Management inventory.

Current flow:

```text
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
```

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

- LilyGO devices
- GPS-only deployments
- Fuel monitoring deployments
- Fuel + GPS deployments
- Fuel + GPS + Vibration deployments
- Custom PCB hardware
- Additional hardware profiles without database redesign

# Next Platform Milestones

Frontend Platform Management
↓
Provisioning Workspace
↓
Organization Management UI
↓
Asset Management UI
↓
Device Management UI
↓
Provisioning Wizard
↓
Sensor Adapter Layer
↓
Hardware Integration
↓
LilyGO Firmware
↓
Production ORBI Hardware

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

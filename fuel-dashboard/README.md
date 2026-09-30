# ORBI Sensor Intelligence Dashboard

## Overview

The ORBI Sensor Intelligence Dashboard is the React + TypeScript operational
frontend for the ORBI Sensor Intelligence Platform.

The application has evolved beyond its original fuel-monitoring scope into a
device-aware operational intelligence system designed to consume telemetry from
physical ORBI hardware.

The platform separates two major concerns:

```text
Platform Administration
        ↓
Physical ORBI Devices
        ↓
Operational Intelligence
```

Platform Administration manages the business and hardware relationships required
to deploy ORBI devices.

Operational Intelligence consumes telemetry from provisioned devices and provides
live monitoring, investigation, spatial intelligence, replay, analytics, and
operational decision support.

The frontend is designed for:

- desktop operations centres
- tablet-based operational review
- mobile field supervision
- future Capacitor Android/iOS packaging

---

# Technology Stack

Current frontend stack:

- React
- TypeScript
- Vite
- Zustand
- Axios
- WebSocket
- Leaflet
- Turf.js
- CSS

Backend integration:

- Rust
- Axum
- Tokio
- SQLx
- PostgreSQL
- PostGIS

---

# Application Architecture

The operational application follows the hierarchy:

```text
Landing Page
        ↓
Organization Overview
        ↓
Fleet Overview
        ↓
Device Selection
        ↓
Device-Specific Operational Dashboard
```

Operational data is scoped to the selected physical device.

This prevents telemetry, alerts, health events, and investigation data from
different devices from being mixed together in the same operational context.

Current device-aware operational feeds include:

```text
telemetry
alerts
device health
fuel events
device state events
sensor health events
geofence transitions
live WebSocket alerts
analytics where device filtering is supported
```

---

# Shared Operational State Architecture

The frontend uses application-scoped Zustand state and shared orchestration for:

```text
selected operational device
telemetry
alerts
investigation intelligence
analytics intelligence
selected analytics period
map/replay state
```

This prevents individual dashboard components from creating competing copies of
the same operational data.

Shared operational state is consumed across surfaces such as:

```text
Operations
Fleet Overview
Investigation
Map Intelligence
Replay Intelligence
Analytics
```

---

# Telemetry Architecture

ORBI telemetry is capability-aware.

Physical devices can contain multiple sensor capabilities including:

```text
GPS
FUEL
VIBRATION
KILL_SWITCH
```

The operational telemetry read model composes the sensor observations required
by the dashboard into a single device observation.

For the Fuel Intelligence Kit, the current operational composition is:

```text
FUEL observation
      +
GPS observation
      +
VIBRATION observation
      ↓
TelemetryStreamReading
```

The frontend telemetry contract includes:

```text
device_id
fuel_level_litres
latitude
longitude
vibration_level
motion_detected
recorded_at
received_at
```

## Live Telemetry

Live telemetry uses:

```http
GET /api/fuel-readings/recent?device_id={device_id}
```

The dashboard currently polls telemetry every:

```text
5 seconds
```

Recent telemetry is ordered by physical observation time so delayed telemetry
replayed from device storage does not redefine the chronology of the live
physical state.

## Historical Telemetry

Replay Intelligence uses:

```http
GET /api/fuel-readings/history
```

Historical telemetry is composed into one operational replay observation per
physical telemetry timestamp.

This prevents individual GPS, FUEL, and VIBRATION sensor rows from becoming
separate replay frames.

Historical observations created before continuous vibration persistence can
legitimately contain:

```text
vibration_level = null
motion_detected = null
```

The frontend preserves these values as unknown rather than fabricating sensor
measurements.

---

# Vibration and Motion Semantics

Raw vibration telemetry and operational device state are intentionally separate
concepts.

Raw telemetry provides:

```text
vibration_level
motion_detected
```

Raw motion is interpreted as:

```text
true  → motion detected
false → motion not detected
null  → unknown
```

It must not be interpreted directly as:

```text
MOVING
IDLE
PARKED
```

Those states belong to the separate operational-state intelligence layer.

Current operational states are:

```text
MOVING
IDLE
PARKED
OFFLINE
UNKNOWN
```

Operational state is derived by backend intelligence using telemetry evidence
rather than directly exposing the raw `motion_detected` boolean.

---

# Operational Dashboard

Current operational sections include:

```text
Operations
Device Health
Investigation
Map Intelligence
Replay Intelligence
Analytics
```

The dashboard also integrates operational telemetry into Fleet Overview.

---

# Operations

The Operations surface provides:

- connection status
- open alert count
- critical alert count
- resolved alert count
- live telemetry preview
- operational alert monitoring
- incident detail review
- alert acknowledgement
- alert resolution
- navigation into Investigation Intelligence

## Alert Lifecycle

The current backend alert lifecycle is:

```text
OPEN
  ↓
ACKNOWLEDGED
  ↓
RESOLVED
```

Supported actions:

```http
PATCH /api/alerts/{alert_id}/acknowledge
PATCH /api/alerts/{alert_id}/resolve
```

## Live Alert Streaming

The frontend connects to:

```text
/ws/alerts
```

Current WebSocket message types include:

```text
live_alert
recovery_alert
alert_acknowledged
heartbeat
```

WebSocket alerts are device-aware.

The frontend automatically reconnects following temporary network or backend
interruptions.

---

# Device Health

Device health is available as a dedicated operational surface.

Current device health states are:

```text
ONLINE
STALE
OFFLINE
UNKNOWN
```

Device health events are retrieved using:

```http
GET /api/device-health-events?device_id={device_id}
```

Device health intelligence is also incorporated into Investigation and Replay
workflows.

---

# Investigation Intelligence

Investigation Intelligence reconstructs operational events around a selected
physical device.

Current investigation feeds include:

```text
fuel events
device state events
sensor health events
geofence transition events
```

Current APIs:

```http
GET /api/fuel-events?device_id={device_id}

GET /api/device-state-events?device_id={device_id}

GET /api/sensor-health-events?device_id={device_id}

GET /api/geofence-transition-events?device_id={device_id}
```

## Investigation Timeline

Operational events are combined into an investigation timeline.

Current capabilities include:

- fuel event investigation
- device state event investigation
- sensor health investigation
- geofence transition investigation
- clustered event grouping
- operational risk scoring
- investigation detail review
- telemetry integrity interpretation
- operational context explanation
- mobile investigation modal behavior
- alert-to-investigation navigation
- investigation-to-map navigation

## Investigation Clusters

Related operational events can be grouped into investigation clusters.

Current cluster risk classifications are:

```text
LOW
MEDIUM
HIGH
CRITICAL
```

Cluster analysis can consider:

- fuel events
- device state activity
- sensor health anomalies
- geofence activity
- operational severity
- correlated telemetry activity

## Fuel Event Intelligence

Fuel event details can expose:

```text
fuel before
fuel after
fuel difference
duration
event time
detection time
severity
confidence
correlation status
correlation reason
delayed detection status
synchronization delay
location
```

Fuel-event severity, confidence, correlation, and alert severity are separate
backend concepts and must not be conflated by the frontend.

## Investigation Navigation

Current workflow:

```text
Operations Alert
        ↓
View Investigation
        ↓
Investigation Timeline
        ↓
Cluster Prioritization
        ↓
Investigation Detail
        ↓
Operational Context Review
```

Where spatial information is available:

```text
Investigation Detail
        ↓
View on Map
        ↓
Map Intelligence
```

---

# Map Intelligence

Map Intelligence provides the spatial operational surface for selected-device
telemetry and investigation data.

Current capabilities include:

- selected-device live positioning
- telemetry-driven map positioning
- investigation event overlays
- geofence transition overlays
- investigation-to-map synchronization
- map-to-investigation synchronization
- geofence-to-investigation synchronization
- live fuel telemetry
- vibration telemetry visibility
- operational telemetry side intelligence
- geofence rendering
- operational polygon drawing
- responsive map workspace

Current map components include:

```text
src/components/map-intelligence/
├── MapIntelligencePanel.tsx
├── OperationalMap.tsx
├── DeviceMarkerLayer.tsx
├── InvestigationEventLayer.tsx
├── GeofenceTransitionLayer.tsx
├── MapFocusController.tsx
├── GeofenceLayer.tsx
├── GeofenceDrawControl.tsx
└── GeofenceCreationCard.tsx
```

## Investigation Spatial Synchronization

Current workflow:

```text
Investigation Selection
        ↓
selectedTimelineItem
        ↓
MapFocusController
        ↓
Map Fly-To
        ↓
Focused Marker
        ↓
Automatic Popup
```

---

# Geofence Intelligence

Geofences are persisted using PostgreSQL/PostGIS.

Current spatial capabilities include:

- polygon drawing
- GeoJSON extraction
- backend geofence persistence
- PostGIS geometry storage
- `ST_Contains` position checks
- operational zone overlays
- device-aware geofence filtering
- ENTERED_ZONE transitions
- EXITED_ZONE transitions
- geofence transition polling
- investigation integration
- replay integration
- geofence utilization analytics
- zone visit frequency
- most active zone intelligence
- zone concentration classification

Current architecture:

```text
Leaflet Draw
      ↓
GeoJSON
      ↓
Zustand Draw Orchestration
      ↓
Rust Geofence APIs
      ↓
PostgreSQL + PostGIS
      ↓
Spatial Intelligence
      ↓
Operational Investigation
```

Important coordinate rule:

```text
Leaflet  → latitude, longitude
PostGIS  → longitude, latitude
```

Potential future geofence intelligence includes:

- depot zones
- fueling station zones
- restricted operational zones
- safe corridors
- dwell-zone detection
- theft outside safe zones
- refill inside fueling zones
- restricted-zone alerts
- route corridor violations
- route-risk analysis
- unauthorized fueling detection

---

# Replay Intelligence

Replay Intelligence reconstructs historical device operation from telemetry.

Current replay components include:

```text
ReplayControls.tsx
ReplayStatusCard.tsx
ReplayMarkerLayer.tsx
ReplayPlaybackController.tsx
ReplayCameraController.tsx
```

Current capabilities include:

- historical telemetry loading
- today replay
- yesterday replay
- last 7 days replay
- custom date-range replay
- playback controls
- replay speed control
- scrubbing
- camera follow
- telemetry trail progression
- breadcrumb intelligence
- investigation replay
- geofence correlation
- fuel event correlation
- device state correlation
- alert correlation
- device health correlation
- replay event feed
- automatic pause on correlated events
- forensic reconstruction
- investigation synchronization

Current workflow:

```text
Investigation Event
        ↓
View on Map
        ↓
Investigation Replay
        ↓
Historical Telemetry
        ↓
Replay Reconstruction
```

Replay reconstruction can correlate:

```text
Telemetry Position
        ↓
Geofence Context

Telemetry Position
        ↓
Fuel Event Context

Telemetry Position
        ↓
Device State Context

Telemetry Position
        ↓
Alert Context

Telemetry Position
        ↓
Device Health Context
```

---

# Journey Intelligence

Journey Intelligence derives operational movement summaries from historical
telemetry and spatial context.

Current capabilities include:

- journey distance calculation
- journey duration calculation
- replay point counting
- visited-zone detection
- zone visit frequency
- last destination reporting

Current outputs include:

```text
Distance Travelled
Journey Duration
Replay Points
Visited Zones
Zone Visit Counts
Last Destination
```

Spatial distance calculations use Turf.js and GeoJSON LineStrings.

Journey Intelligence is device-scoped and replay-aware.

---

# Analytics Intelligence

Analytics is intentionally separated from raw operational event feeds.

Operational endpoints provide:

```text
raw events
telemetry
investigation evidence
```

Analytics endpoints provide:

```text
aggregated intelligence
trend analysis
operational summaries
```

Current analytics capabilities include:

- Alert Trends
- Geofence Activity Trends
- Device Health Trends
- Geofence Utilization

Current endpoints:

```http
GET /api/analytics/alert-trends

GET /api/analytics/geofence-activity

GET /api/analytics/device-health-trends

GET /api/analytics/geofence-utilization
```

Shared analytics periods:

```text
Last 7 Days
Last 30 Days
Last 90 Days
```

## Analytics Scope

Not every analytics endpoint has the same scope.

Currently:

```text
Alert Trends
→ optionally device-scoped

Geofence Activity
→ optionally device-scoped

Device Health Trends
→ fleet-wide

Geofence Utilization
→ fleet-wide
```

The frontend preserves these backend scope semantics rather than pretending
every analytics surface is selected-device-specific.

---

# Platform Administration

Platform Administration is conceptually separate from Operational Intelligence.

Its responsibilities include:

```text
Organizations
Assets
ORBI Product Catalogue
Hardware Profiles
Device Inventory
Device Verification
Provisioning
Device Lifecycle
Provisioned Devices
```

The existing `orbi-provision` application contains provisioning and
device-management capabilities.

A future integration phase will establish a coherent ORBI Administration
experience so administrators can move between platform administration and
operational intelligence without treating them as unrelated products.

This integration must preserve the architectural separation between:

```text
Administration
→ manages platform and deployment relationships

Operational Intelligence
→ monitors and investigates deployed physical devices
```

---

# ORBI Product Catalogue

Current product direction includes:

| Product | Capabilities |
| --- | --- |
| ORBI GPS Lite | GPS Tracking |
| ORBI GPS Control Kit | GPS Tracking, Remote Kill Switch |
| ORBI Fuel Intelligence Kit | Fuel Monitoring, GPS Tracking, Vibration Detection |
| ORBI Full Intelligence Kit | Fuel Monitoring, GPS Tracking, Vibration Detection, Remote Kill Switch |

---

# Device Inventory and Provisioning

ORBI follows an inventory-first provisioning model.

Physical devices progress through a controlled lifecycle:

```text
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
```

Only eligible devices can proceed into provisioning.

Inventory records can contain:

- ORBI Device Code
- Serial Number
- IMEI
- Product
- Product Code
- Hardware Profile
- Firmware Version
- Manufacturing Status
- Quality Test Status

Conceptual deployment workflow:

```text
Device Inventory
        ↓
Device Verification
        ↓
Organization
        ↓
Asset
        ↓
Provisioning
        ↓
Operational Intelligence
```

---

# Fuel Calibration

The backend contains the physical fuel-calibration domain used by ORBI Fuel
Intelligence devices.

Runtime fuel telemetry is converted from physical KUM ultrasonic measurements
into calibrated fuel quantities using the active production calibration.

## Fuel Calibration Frontend Status

The guided calibration frontend is now implemented against the
server-authoritative calibration workflow.

The current workflow supports:

```text
Select Provisioned FUEL Sensor
↓
Create / Load Calibration Profile
↓
Start Guided Calibration Session
↓
Physically Add or Remove a Known Fuel Quantity
↓
Enter Signed Cumulative Litre Change
↓
Capture Calibration Point
↓
Backend Resolves Latest Persisted Physical KUM Observation
↓
Backend Validates Observation Freshness
↓
Backend Persists Authoritative Calibration Point
↓
Repeat as Required
↓
Establish Absolute Full-Tank Anchor
↓
Complete Session
↓
Publish Validated Runtime Calibration
↓
Explicit Production Approval
```

The frontend displays the latest physical KUM observation, including real-time,
smooth, and raw distance measurements, but it does not submit a browser-selected
distance as authoritative calibration evidence.

The current measurement policy is:

```text
Real-Time Distance
→ authoritative calibration and runtime fuel measurement

Raw Distance
→ physical diagnostic evidence

Smooth Distance
→ diagnostic, trend, and stability evidence
```

The backend remains authoritative for the physical observation, observation
freshness, calibration distance, resolved litres, coverage, confidence,
publication validation, and production activation.

The calibration UI exposes:

- declared tank capacity
- latest physical KUM observation
- real-time, smooth, and raw distance
- signed cumulative fuel change
- captured calibration points
- verified fuel range
- coverage percentage
- calibration confidence
- pause / resume / abandon controls
- full-tank absolute-anchor workflow
- session completion
- publication
- explicit production approval
- stale or missing physical-observation errors

Publication and production approval remain separate actions because publishing a
valid lookup-table candidate does not authorize it for normal runtime fuel
intelligence.

## Automated Stability-Aware Capture — Next Calibration Milestone

The next calibration milestone removes the requirement for the installer to
manually decide the precise moment at which to press **Capture Point**.

The installer will continue to enter the known signed cumulative litre change.
ORBI will then automatically observe incoming physical KUM measurements until the
backend stability engine determines that sufficient stable evidence exists.

Target interaction:

```text
Installer Adds or Removes Known Fuel Quantity
↓
Installer Enters Signed Cumulative Litre Change
↓
Start Measurement
↓
Frontend Displays Live Measurement Animation
↓
Backend Evaluates Incoming KUM Observation Window
↓
WAITING_FOR_TELEMETRY
↓
OBSERVING
↓
SETTLING
↓
STABLE
↓
Backend Automatically Captures Authoritative Real-Time Distance
↓
CAPTURED
↓
Frontend Confirms Calibration Point
```

This is an automation and installer-experience improvement. It does not change
the established calibration mathematics or authoritative measurement policy.

The browser must not independently determine that a measurement is stable.
Instead, it visualizes backend state.

Conceptually:

```text
Physical KUM Telemetry
↓
Backend Stability Engine
├── real-time distance
├── raw distance
├── smooth distance
├── observation timestamps
└── sensor validity
↓
Stability State + Measurement Feedback
↓
Frontend Animation
↓
Automatic Server-Authoritative Capture
```

The animation should represent actual measurement progress rather than an
artificial timer.

Example installer presentation:

```text
Cumulative Fuel
1.00 L

Analyzing Sensor Stability...

Real-Time    20.10 cm
Raw          20.18 cm
Smooth       20.04 cm

Stability
████████████████░░░░  Settling...

        ↓

Measurement Stable ✓

Calibration Point Captured
1.00 L → 20.10 cm
```

Tank installations may differ in geometry, capacity, sensor mounting, liquid
movement, vibration, and ultrasonic behaviour. The stability engine should
therefore evaluate observed signal behaviour rather than rely on one fixed
waiting period for every installation.

Exact stability thresholds will be established and validated against physical
hardware before being treated as production constants.

The automated workflow must preserve the existing manual/server-authoritative
capture foundation while it is introduced incrementally.


---

# Vibration Intelligence

Continuous vibration observations are now persisted under the dedicated
VIBRATION sensor capability and exposed through the composed telemetry read
model.

This provides the frontend foundation for vibration visibility.

Current frontend behavior supports raw vibration telemetry.

Further frontend work may refine how vibration information is presented so that
operators receive useful operational context rather than an unexplained raw
number.

Any future vibration intelligence UI must preserve the distinction between:

```text
raw physical vibration
        ↓
motion evidence
        ↓
operational-state intelligence
```

---

# Responsive Design

The dashboard supports:

- desktop monitoring
- tablet review
- mobile field usage
- future Capacitor packaging

Current responsive behavior includes:

- horizontally scrollable dashboard tabs
- responsive status cards
- mobile alert cards
- incident detail bottom sheet
- mobile investigation modal behavior
- collapsible telemetry
- responsive map workspace

---

# Environment Variables

Production-style example:

```env
VITE_API_BASE_URL=https://rust-api.williamtekpeh.com
VITE_WS_BASE_URL=wss://rust-api.williamtekpeh.com
```

Local development:

```env
VITE_API_BASE_URL=http://127.0.0.1:9000
VITE_WS_BASE_URL=ws://127.0.0.1:9000
```

---

# Current Development Status

## Operational Frontend

Implemented:

- React + TypeScript application
- shared HTTP client
- Zustand operational state
- selected-device operational context
- telemetry polling
- WebSocket alert streaming
- automatic WebSocket reconnect
- alert lifecycle management
- responsive Operations dashboard
- Device Health
- Investigation Intelligence
- Map Intelligence
- Replay Intelligence
- Journey Intelligence
- Geofence Intelligence
- Analytics Intelligence
- Fleet Overview telemetry integration

## Telemetry Reconciliation

Completed:

- device-scoped telemetry
- nullable telemetry handling
- raw motion semantic reconciliation
- operational-state semantic separation
- fuel event severity reconciliation
- fuel confidence reconciliation
- fuel correlation reconciliation
- sensor health nullable-time handling
- alert contract reconciliation
- analytics scope reconciliation
- composed FUEL + GPS + VIBRATION live telemetry
- composed FUEL + GPS + VIBRATION historical telemetry
- continuous vibration persistence integration
- historical pre-vibration compatibility

Current validation checkpoint:

```text
Backend cargo test
→ 245 passed
→ 0 failed

Frontend npm run build
→ successful
```

The telemetry composition has also been runtime-validated using physical ORBI
hardware and persisted PostgreSQL telemetry.

The backend contract that the upcoming calibration UI will consume has also
completed its dedicated guided-calibration regression checkpoint:

```text
65 calibration tests passed
0 failed
```

This includes the server-authoritative physical-observation capture path and
freshness protection required by the redesigned wizard.

---

# Pending Platform Work

Major remaining work includes:

- Automated fuel-calibration stability integration and animated capture workflow
- Vibration Intelligence/UI refinement
- ORBI Administration integration with `orbi-provision`
- Authentication
- Authorization / RBAC
- Keycloak integration
- Firmware Management
- OTA Firmware Updates
- Remote Kill Switch operational integration
- Reporting / export
- Notification integrations
- Capacitor Android/iOS packaging

Later architecture work includes:

- distributed-service boundaries where operationally justified
- broker/event-driven integration where justified
- Kubernetes deployment architecture
- production observability and operational hardening

---

# Next Platform Milestones

The current execution direction is:

```text
Operational Frontend Reconciliation
                ✅
                ↓
Physical Vibration Telemetry Integration
                ✅
                ↓
Guided Fuel Calibration Workflow
                ✅
                ↓
Server-Authoritative Physical Capture
                ✅
                ↓
Physical Calibration → Production Runtime Validation
                ✅
                ↓
Automated Calibration Stability Engine
                ← CURRENT
                ↓
Automated Stability-Aware Point Capture
                ⏳
                ↓
Animated Installer Calibration Experience
                ⏳
                ↓
Final Automated Physical Calibration Validation
                ⏳
                ↓
Vibration Intelligence / UI Refinement
                ⏳
                ↓
ORBI Administration Integration
(orbi-provision)
                ⏳
                ↓
Authentication + Authorization
(Keycloak / RBAC)
                ⏳
                ↓
Firmware / Device Management Expansion
                ↓
Distributed Services
(where justified)
                ↓
Kubernetes
                ↓
Production Platform Hardening
```

The ordering of Administration integration and Keycloak may be refined after the
administration boundaries and authorization requirements are inspected.


---

# Product Direction

ORBI is evolving into a general Sensor Intelligence Platform built around
physical telemetry, operational context, investigation, spatial intelligence,
and device administration.

Current primary intelligence areas include:

```text
Fleet Intelligence
Fuel Intelligence
GPS / Spatial Intelligence
Vibration / Motion Intelligence
Device Health Intelligence
```

Potential future intelligence domains include:

```text
Generator Intelligence
Cold Chain Intelligence
Payload Intelligence
Energy Monitoring
```

The platform should expand into these areas only where supported by real product
requirements and physical sensor capabilities.

---

# Architectural Principle

ORBI separates raw telemetry from derived intelligence.

```text
Physical Sensors
        ↓
Telemetry
        ↓
Operational State
        ↓
Investigation
        ↓
Replay
        ↓
Map / Spatial Context
        ↓
Analytics
        ↓
Operational Decision Support
```

Platform Administration remains responsible for determining:

```text
who owns the device
what asset it belongs to
which hardware capabilities it contains
how it was provisioned
which organization can operate it
```

Operational Intelligence is responsible for determining:

```text
what the device is reporting
what happened
where it happened
when it happened
how the evidence correlates
and what the operator needs to investigate
```

Maintaining this separation is a core architectural principle of the ORBI
Sensor Intelligence Platform.
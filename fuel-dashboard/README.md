# Fuel Dashboard

## Overview

The Fuel Dashboard is the React + TypeScript frontend for the Fuel Intelligence Platform.

It provides a professional operational interface for:

- live fuel telemetry visibility
- operational alert monitoring
- incident acknowledgment
- incident resolution
- device health monitoring
- mobile-friendly field usage

The dashboard is designed to support both:

- desktop operations centres
- mobile field supervisors through future Capacitor Android/iOS packaging

---

# Current Stack

- React
- TypeScript
- Vite
- Zustand
- Axios
- WebSocket
- CSS

---

# Current Features

## Live Operations Dashboard

The main dashboard provides:

- connection status
- open alert count
- critical alert count
- resolved alert count
- live telemetry preview
- operational alert list
- incident detail panel

## Multi-Organization Operational Flow

The frontend is now transitioning from a single global operational dashboard into a multi-organization operational intelligence platform.

Current frontend hierarchy:

```text
Landing Page
→ Organization Overview
→ Fleet Overview
→ Device Selection
→ Device-Specific Operational Dashboard
```

The operational dashboard is now device-aware.

This means dashboard operational data is scoped to the selected physical device instead of globally aggregating all telemetry.

Current device-scoped frontend feeds:

```text
alerts
telemetry
device health
live WebSocket alerts
```

The frontend now passes `device_id` into backend operational APIs.

Current filtered backend endpoints:

```http
GET /api/alerts?device_id={device_id}

GET /api/fuel-readings/recent?device_id={device_id}

GET /api/device-health-events?device_id={device_id}

GET /api/fuel-events?device_id={device_id}

GET /api/device-state-events?device_id={device_id}

GET /api/sensor-health-events?device_id={device_id}
```

This architecture prevents operational conflicts where:

- telemetry from multiple devices mixes together
- alerts from unrelated devices appear in the selected dashboard
- device health becomes operationally ambiguous

WebSocket live alerts are also now filtered using `device_id` so dashboards only receive operational alerts relevant to the selected device context.

This architecture provides the frontend foundation for future features such as:

- investigation timeline replay
- operational reconstruction
- route replay
- device diagnostics
- predictive maintenance
- sensor analytics
- operational heatmaps
- forensic operational investigation

---

# Shared Operational State Architecture

The frontend now uses application-scoped operational orchestration for:

```text
telemetry
investigation intelligence
alerts
selected operational device
```

This prevents duplicated polling and ensures all operational surfaces share synchronized live state.

Current shared operational surfaces:

```text
Dashboard
Fleet Overview
Map Intelligence
Investigation
```

This architecture enables:

- synchronized operational investigation workflows
- future route replay
- future telemetry replay
- fleet-wide operational intelligence
- spatial investigation workflows
- future geofence intelligence
- future operational heatmaps

---

## Alert Workflow

The dashboard supports the current backend alert lifecycle:

`````text
OPEN
→ ACKNOWLEDGED
→ RESOLVED

Supported actions:

PATCH /api/alerts/{alert_id}/acknowledge
PATCH /api/alerts/{alert_id}/resolve
WebSocket Alert Streaming

The dashboard connects to:

/ws/alerts

Supported WebSocket message types:

live_alert
recovery_alert
alert_acknowledged
heartbeat

The frontend automatically reconnects if the backend or network drops.

Live Telemetry Stream

The telemetry stream is read-only and uses polling:

GET /api/fuel-readings/recent

Current polling interval:

5 seconds

Telemetry is collapsed by default to keep the dashboard clean.

Device Health

Device health is available as a separate dashboard tab.

It uses:

GET /api/device-health-events

Current statuses:

ONLINE
STALE
OFFLINE
UNKNOWN
Dashboard Sections

## Investigation Intelligence

The Investigation tab has evolved from a simple event list into an operational investigation and telemetry intelligence workflow.

Current investigation capabilities include:

- fuel event investigation timeline
- clustered operational event grouping
- operational risk scoring
- correlated telemetry interpretation
- investigation detail side panel
- mobile investigation modal behavior
- telemetry integrity interpretation
- operational context explanation
- alert-to-investigation navigation flow

Current investigation feeds:

```text
fuel events
device state events
sensor health events

Current investigation APIs:

GET /api/fuel-events?device_id={device_id}

GET /api/device-state-events?device_id={device_id}

GET /api/sensor-health-events?device_id={device_id}

The investigation system now groups operational telemetry into correlated investigation clusters.

Examples:

theft patterns during idle periods
suspicious refill activity during movement
sensor integrity anomalies during fuel events
correlated operational movement patterns
telemetry clock drift interpretation

Current investigation intelligence features:

Cluster Risk Scoring

Clusters are currently classified as:

LOW
MEDIUM
HIGH
CRITICAL

based on:

fuel theft patterns
refill patterns
sensor health anomalies
operational severity
correlated telemetry activity
Cluster Explanation Engine

The dashboard now generates operator-friendly explanations for investigation clusters.

Examples:

Fuel theft behavior appears operationally consistent with surrounding movement telemetry.
Sensor integrity anomalies detected during suspicious fuel activity.
Potential conflicting fuel activity detected within the same operational window.
Investigation Detail Panel

The investigation detail panel now exposes:

fuel before/after values
fuel delta
confidence scoring
correlation status
operational context
telemetry timestamps
intelligence detection timestamps
delayed synchronization indicators
telemetry clock drift warnings
Cross-Dashboard Navigation

Operations alerts now support direct navigation into Investigation workflows.

Operational flow:

Operations Alert
→ View Investigation
→ Investigation Tab
→ Cluster Prioritization
→ Highlighted Investigation Event
→ Operational Context Review

This creates a connected operational intelligence workflow instead of isolated dashboard sections

Current sections:

Operations
Device Health
Investigation
Analytics

Implemented Dashboard Sections:

- Operations
- Device Health
- Investigation
- Map Intelligence
- Replay Intelligence
- Fleet Overview Operational Telemetry

Planned Phase 2 Features:

- animated telemetry playback
- investigation hotspot clustering
- operational heatmaps
- theft corridor analysis
- live route streaming
- multi-device fleet rendering
- fleet replay intelligence
- advanced investigation filtering
- analytics charts and trends
- predictive operational scoring
- sensor adapter visualization
Responsive Design

---

# Map Intelligence Phase 1

The platform now includes a dedicated Map Intelligence operational surface.

The Map Intelligence system is integrated directly into the operational dashboard architecture and shares the same device-scoped telemetry and investigation state used across the platform.

Current Map Intelligence capabilities include:

- live selected-device spatial rendering
- telemetry-driven device positioning
- investigation event spatial overlays
- investigation-to-map synchronization
- map-to-investigation synchronization
- live fuel telemetry gauge
- operational telemetry side intelligence panel
- responsive operational map workspace

Current map intelligence workflow:

```text
Operations Alert
→ View Investigation
→ Investigation Detail
→ View on Map
→ Map Intelligence
→ Automatic Fly-To Investigation Event
→ Focused Event Popup
```

## Current Map Intelligence Features

### Operational Map Surface

The operational map currently supports:

- OpenStreetMap rendering
- selected device tracking
- investigation event overlays
- investigation event focus synchronization
- telemetry-driven operational positioning
- live operational telemetry side panel

### Investigation Spatial Synchronization

Current synchronization behavior:

```text
Investigation Selection
→ selectedTimelineItem updates
→ MapFocusController reacts
→ map flies to event
→ focused marker enlarges
→ popup opens automatically
```

### Reusable Telemetry Widgets

Reusable telemetry widgets are now shared across:

```text
Fleet Overview
Map Intelligence
```

Current reusable telemetry widgets:

- FuelLevelGauge

### Current Frontend Map Structure

```text
src/components/map-intelligence/
├── MapIntelligencePanel.tsx
├── OperationalMap.tsx
├── DeviceMarkerLayer.tsx
├── InvestigationEventLayer.tsx
├── MapFocusController.tsx
```

Shared telemetry widget structure:

```text
src/components/shared/
└── FuelLevelGauge.tsx
````md
Replay intelligence structure:

```text
src/components/map-intelligence/
├── ReplayControls.tsx
├── ReplayStatusCard.tsx
├── ReplayMarkerLayer.tsx
├── ReplayPlaybackController.tsx
├── ReplayCameraController.tsx

Current replay intelligence capabilities:

replay playback controls
replay speed control
replay scrubbing
replay camera follow
replay telemetry trail progression
replay breadcrumb intelligence
replay event correlation
automatic replay pause on correlated events
replay investigation synchronization
geofence-aware replay context

````md
### Spatial Intelligence & Geofence Operations

Current spatial intelligence capabilities:

- PostGIS-backed geofence persistence
- operational polygon drawing tools
- draw-to-save geofence workflow
- backend GeoJSON delivery
- replay-aware geofence intelligence
- telemetry-aware geofence status
- live PostGIS ST_Contains spatial checks
- operational zone overlays
- device-aware geofence assignment foundation
- replay spatial synchronization
- operational map workspace controls

Current spatial intelligence architecture:

Leaflet Draw Tools
→ GeoJSON extraction
→ Zustand draw orchestration
→ Rust geofence APIs
→ PostgreSQL + PostGIS persistence
→ backend ST_Contains intelligence
→ device-aware geofence filtering
→ replay-aware spatial intelligence
→ operational investigation workflows

````md
### Current Operational Geofence Workflow

```text
draw operational zone
→ modal metadata workflow
→ backend persistence
→ PostGIS geometry storage
→ telemetry position checks
→ replay-aware zone intelligence
→ operational geofence status

Planned geofence intelligence capabilities:

- depot zones
- fueling station zones
- restricted operational zones
- safe corridors
- route-risk analysis
- dwell-zone detection
- theft outside safe-zone detection
- refill-inside-fueling-zone intelligence
- depot entry/exit intelligence
- restricted-zone alerts
- dwell detection
- route corridor violations
- operational hotspot analysis
- replay spatial investigations
- zone transition intelligence
- unauthorized fueling detection

Important coordinate rule:

```text
Leaflet uses latitude, longitude
PostGIS uses longitude, latitude
```
---

Operational draw behavior:

```text
drawing mode
→ replay pauses
→ camera follow pauses
→ map focus synchronization pauses
→ operational workspace stabilizes
```

# Responsive Design

The dashboard is designed for:

desktop monitoring
tablet review
mobile field usage
future Capacitor packaging

Mobile behavior includes:

horizontal dashboard tabs
horizontal status cards
mobile alert cards instead of wide tables
incident detail bottom-sheet panel
collapsible telemetry section
Environment Variables

Example:

VITE_API_BASE_URL=https://rust-api.williamtekpeh.com
VITE_WS_BASE_URL=wss://rust-api.williamtekpeh.com

Local development example:

VITE_API_BASE_URL=http://127.0.0.1:9000
VITE_WS_BASE_URL=ws://127.0.0.1:9000
Current Development Status

Implemented:

React + TypeScript setup
shared API client
alert API module
telemetry API module
device health API module
Zustand alert store
Zustand telemetry store
Zustand device health store
WebSocket alert manager
automatic WebSocket reconnect
live operations dashboard
responsive tabs/menu
telemetry polling
alert table
mobile alert cards
alert detail panel
acknowledge/resolve actions
device health tab
map intelligence operational workspace
telemetry route rendering
breadcrumb telemetry progression
replay playback engine
replay scrubbing
replay investigation correlation
replay camera follow
replay forensic pause workflow
geofence rendering foundation
geofence replay awareness
reusable telemetry widgets
fuel telemetry gauges
fleet operational telemetry widgets

Pending:

landing page / company overview
company/device/sensor selection
alert filters
investigation timeline
analytics charts
authentication
Capacitor Android/iOS packaging
Product Direction

The frontend is being built as an operational fuel monitoring interface first.

The goal is not to build a generic admin dashboard.

The goal is to support:

sensor data
→ operational intelligence
→ live alert workflow
→ investigation
→ resolution

The next major product direction is:

landing page
→ company/device/sensor overview
→ selected operational dashboard

Then commit mentally that every time we add a major frontend capability, we update this README.
`````

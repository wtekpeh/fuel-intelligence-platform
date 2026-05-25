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

## Alert Workflow

The dashboard supports the current backend alert lifecycle:

```text
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

Current sections:

Operations
Device Health
Investigation
Analytics

Implemented:

Operations
Device Health

Planned:

Investigation timeline replay
Analytics charts and trends
Responsive Design

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

Pending:

landing page / company overview
company/device/sensor selection
alert filters
investigation timeline
event replay
map view
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
```

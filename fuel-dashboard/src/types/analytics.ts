export interface AlertTrendSummary {
  total_alerts: number;
  theft_alerts: number;
  refill_alerts: number;
  leak_alerts: number;
  open_alerts: number;
  acknowledged_alerts: number;
  resolved_alerts: number;
}

export interface AlertTrendPoint {
  day: string;
  alert_type: string;
  status: string;
  count: number;
}

export interface AlertTrendsResponse {
  days: number;
  summary: AlertTrendSummary;
  trend: AlertTrendPoint[];
}

export interface GeofenceActivityTrendPoint {
  day: string;
  entries: number;
  exits: number;
}

export interface GeofenceActivityTrendResponse {
  days: number;
  trend: GeofenceActivityTrendPoint[];
}

export interface DeviceHealthTrendDevice {
  device_id: string;
  device_code: string;
  offline_events: number;
  stale_events: number;
  recovery_events: number;
  reliability_issue_count: number;
}

export interface DeviceHealthTrendResponse {
  days: number;
  devices: DeviceHealthTrendDevice[];
}

export interface GeofenceUtilizationZone {
  geofence_name: string;
  visits: number;
}

export interface GeofenceUtilizationResponse {
  days: number;
  zones: GeofenceUtilizationZone[];
}

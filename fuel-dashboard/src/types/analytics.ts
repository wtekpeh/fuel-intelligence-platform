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

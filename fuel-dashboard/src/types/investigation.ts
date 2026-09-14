export interface FuelEvent {
  id: string;
  event_type: string;
  event_time: string;
  detected_at: string;
  fuel_before: number;
  fuel_after: number;
  fuel_difference: number;
  duration_seconds: number;
  latitude: number | null;
  longitude: number | null;
  is_delayed_detection: boolean;
  sync_delay_seconds: number;
  severity: string;
  confidence: string | null;
  correlation_status: string | null;
  correlation_reason: string | null;
  message: string;
}

export interface DeviceStateEvent {
  state: string;
  vibration_level: number | null;
  motion_detected: boolean | null;
  latitude: number | null;
  longitude: number | null;
  recorded_at: string;
}

export interface SensorHealthEvent {
  id: string;
  device_id: string;
  sensor_id: string;
  event_type: string;
  severity: string;
  reason: string;
  first_seen_at: string | null;
  last_seen_at: string | null;
  detected_at: string;
}

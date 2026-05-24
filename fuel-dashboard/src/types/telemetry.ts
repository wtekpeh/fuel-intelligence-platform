export interface TelemetryStreamReading {
  device_id: string;
  fuel_level_litres: number;
  latitude: number | null;
  longitude: number | null;
  vibration_level: number | null;
  motion_detected: boolean | null;
  recorded_at: string;
  received_at: string;
}

export type DeviceHealthStatus = "ONLINE" | "STALE" | "OFFLINE" | "UNKNOWN";

export interface DeviceHealthEvent {
  id: string;
  device_id: string;
  previous_status: DeviceHealthStatus | null;
  new_status: DeviceHealthStatus;
  reason: string;
  detected_at: string;
}

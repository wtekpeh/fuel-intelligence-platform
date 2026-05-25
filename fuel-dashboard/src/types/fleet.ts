export interface OrganizationFleetOverview {
  asset_id: string;
  asset_name: string;
  asset_type: string;
  capacity_litres: number | null;

  device_id: string;
  device_code: string;
  device_status: "ONLINE" | "STALE" | "OFFLINE" | "UNKNOWN";
  last_seen_at: string | null;

  sensor_count: number;
  sensor_types: string[];

  open_alert_count: number;
}

export interface OrganizationOverview {
  organization_id: string;

  organization_name: string;

  industry: string | null;

  asset_count: number;

  device_count: number;

  online_device_count: number;

  stale_device_count: number;

  offline_device_count: number;

  open_alert_count: number;
}

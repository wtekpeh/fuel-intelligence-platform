export interface HardwareProfile {
  id: string;
  profile_code: string;
  name: string;
  description: string;
  created_at: string;
}

export interface HardwareProfileSensor {
  id: string;
  hardware_profile_id: string;
  sensor_type: string;
  unit: string;
  created_at: string;
}

export interface DeviceSummary {
  id: string;
  device_code: string;
  asset_id: string;
  hardware_profile_id: string;
  hardware_profile_code: string;
  hardware_profile_name: string;
  status: string;
  created_at: string;
}

export interface DeviceSensorSummary {
  id: string;
  device_id: string;
  sensor_code: string;
  sensor_type: string;
  unit: string;
  created_at: string;
}

export interface RegisterDeviceRequest {
  asset_id: string;
  device_code: string;
  hardware_profile_id: string;
}

export interface HardwareProfileResponse {
  id: string;
  profileCode: string;
  name: string;
  description?: string | null;
  isActive: boolean;
  createdAt: string;
}

export interface HardwareProfileSensorResponse {
  id: string;
  hardwareProfileId: string;
  sensorType: string;
  unit: string;
}

export interface DeviceModelResponse {
  id: string;
  modelCode: string;
  modelName: string;
  manufacturer?: string | null;
  description?: string | null;
  isActive: boolean;
  createdAt: string;
}

export interface DeviceCatalogueSensor {
  id: string;
  sensorType: string;
  unit: string;
}

export interface DeviceCatalogueHardwareProfile {
  id: string;
  profileCode: string;
  name: string;
  description?: string | null;
  isDefault: boolean;
  sensors: DeviceCatalogueSensor[];
}

export interface DeviceCatalogueModel {
  id: string;
  modelCode: string;
  modelName: string;
  manufacturer?: string | null;
  description?: string | null;
  isActive: boolean;
  profiles: DeviceCatalogueHardwareProfile[];
}

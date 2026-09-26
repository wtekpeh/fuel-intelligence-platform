export type FuelCalibrationProfileStatus =
  | "draft"
  | "progressive"
  | "validated"
  | "production"
  | "superseded";

export type FuelCalibrationConfidence = "low" | "medium" | "high" | "verified";

export type FuelCalibrationSessionStatus =
  | "active"
  | "paused"
  | "completed"
  | "abandoned";

export interface CreateFuelCalibrationProfileRequest {
  tank_capacity_litres: number;
}

export interface StartFuelCalibrationSessionRequest {
  starting_litres: number | null;
}

export interface CaptureFuelCalibrationPointRequest {
  level_cm: number;
  cumulative_change_litres: number;
}

export interface ApplyFuelCalibrationAnchorRequest {
  cumulative_change_litres: number;
  absolute_litres: number;
}

export interface FuelCalibrationProfileMutationResponse {
  profile_id: string;
  message: string;
}

export interface FuelCalibrationSessionMutationResponse {
  session_id: string;
  message: string;
}

export interface FuelCalibrationPointMutationResponse {
  point_id: string;
  message: string;
}

export interface FuelCalibrationSessionPoint {
  id: string;
  level_cm: number;
  cumulative_change_litres: number;
  resolved_litres: number | null;
  captured_at: string;
}

export interface FuelCalibrationSession {
  id: string;
  status: FuelCalibrationSessionStatus;

  started_at: string;
  completed_at: string | null;

  starting_litres: number | null;
  ending_litres: number | null;

  anchor_cumulative_change_litres: number | null;
  anchor_absolute_litres: number | null;
  anchor_established_at: string | null;

  points: FuelCalibrationSessionPoint[];
}

export interface FuelCalibrationProfile {
  id: string;
  sensor_id: string;

  tank_capacity_litres: number;

  status: FuelCalibrationProfileStatus;
  confidence: FuelCalibrationConfidence;

  verified_from_litres: number;
  verified_to_litres: number;
  coverage_percentage: number;

  published_calibration_id: string | null;

  sessions: FuelCalibrationSession[];

  created_at: string;
  updated_at: string;
}

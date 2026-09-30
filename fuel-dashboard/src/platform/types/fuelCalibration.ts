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

/**
 * Backend-authoritative physical stability state used during automatic
 * guided fuel-calibration capture.
 *
 * The frontend does not calculate stability itself. It renders the state
 * produced by the backend from real KUM physical observations.
 */
export type FuelCalibrationStabilityState =
  | "waiting_for_telemetry"
  | "observing"
  | "settling"
  | "stable";

export interface CreateFuelCalibrationProfileRequest {
  tank_capacity_litres: number;
}

export interface StartFuelCalibrationSessionRequest {
  starting_litres: number | null;
}

/**
 * Starts or continues one automatic guided-calibration capture attempt.
 *
 * The installer supplies the known cumulative fuel change.
 *
 * `observation_started_at` is established once when the installer presses
 * "Start Automatic Capture". The same timestamp must then be reused for
 * every poll belonging to that capture attempt.
 *
 * This gives the backend an authoritative lower time boundary so physical
 * KUM observations recorded before the installer started the capture cannot
 * satisfy the stability evaluation.
 *
 * The frontend does not calculate physical stability itself.
 */
export interface CaptureFuelCalibrationPointRequest {
  cumulative_change_litres: number;
  observation_started_at: string;
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

/**
 * Result of one automatic stability evaluation.
 *
 * While the physical fuel measurement is still being evaluated:
 *
 *   captured = false
 *   point_id = null
 *
 * Once the backend determines that the KUM measurement is stable:
 *
 *   state = "stable"
 *   captured = true
 *   point_id = <persisted calibration point UUID>
 *
 * Repeated requests for the same cumulative fuel position remain
 * idempotent and may return the already-existing point ID.
 */
export interface FuelCalibrationAutomaticCaptureResponse {
  state: FuelCalibrationStabilityState;

  sample_count: number;
  observation_duration_seconds: number;

  realtime_range_cm: number | null;
  realtime_slope_cm_per_second: number | null;

  capture_distance_cm: number | null;

  captured: boolean;
  point_id: string | null;

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

/**
 * Latest physical KUM observation before tank-specific distance-to-litres
 * calibration is applied.
 *
 * This remains useful for diagnostics and installer visibility, but automatic
 * calibration capture is controlled by the backend stability evaluator rather
 * than by the frontend selecting this observation directly.
 */
export interface LatestFuelSensorObservation {
  sensor_id: string;
  device_id: string;

  recorded_at: string;

  realtime_distance_cm: number;
  smooth_distance_cm: number;
  raw_distance_cm: number;

  temperature_c: number | null;

  status_1: number | null;
  status_2: number | null;
  raw_data_validity: number | null;

  latitude: number | null;
  longitude: number | null;
}

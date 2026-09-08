const MIN_FUEL_OUTLIER_TOLERANCE_LITRES: f64 = 0.02;

#[derive(Debug, Clone, PartialEq)]
pub enum TelemetryQualityStatus {
    Valid,
    Invalid,
    Suspicious,
}

#[derive(Debug, Clone)]
pub struct FuelTelemetryQualityResult {
    pub status: TelemetryQualityStatus,
    pub reason: Option<String>,
}

pub fn validate_fuel_range(
    fuel_level_litres: f64,
    tank_capacity_litres: f64,
) -> FuelTelemetryQualityResult {
    if !fuel_level_litres.is_finite() {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Invalid,
            reason: Some(format!(
                "Fuel level {} is not a finite value.",
                fuel_level_litres
            )),
        };
    }

    if !tank_capacity_litres.is_finite() || tank_capacity_litres <= 0.0 {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Invalid,
            reason: Some(format!(
                "Tank capacity {} is not a valid positive finite value.",
                tank_capacity_litres
            )),
        };
    }

    if fuel_level_litres < 0.0 {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Invalid,
            reason: Some("Fuel level cannot be negative.".to_string()),
        };
    }

    if fuel_level_litres > tank_capacity_litres {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Invalid,
            reason: Some("Fuel level exceeds tank capacity.".to_string()),
        };
    }

    FuelTelemetryQualityResult {
        status: TelemetryQualityStatus::Valid,
        reason: None,
    }
}

pub fn detect_impossible_fuel_jump(
    previous_fuel_litres: f64,
    current_fuel_litres: f64,
    max_allowed_jump_litres: f64,
) -> FuelTelemetryQualityResult {
    if !previous_fuel_litres.is_finite() {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Invalid,
            reason: Some(format!(
                "Previous fuel level {} is not a finite value.",
                previous_fuel_litres
            )),
        };
    }

    if !current_fuel_litres.is_finite() {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Invalid,
            reason: Some(format!(
                "Current fuel level {} is not a finite value.",
                current_fuel_litres
            )),
        };
    }

    if !max_allowed_jump_litres.is_finite() || max_allowed_jump_litres <= 0.0 {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Invalid,
            reason: Some(format!(
                "Maximum allowed fuel jump {} is not a valid positive finite value.",
                max_allowed_jump_litres
            )),
        };
    }

    let jump = (current_fuel_litres - previous_fuel_litres).abs();

    if jump > max_allowed_jump_litres {
        return FuelTelemetryQualityResult {
            status: TelemetryQualityStatus::Suspicious,
            reason: Some(format!(
                "Fuel jump of {:.2} litres exceeds allowed threshold of {:.2} litres.",
                jump, max_allowed_jump_litres
            )),
        };
    }

    FuelTelemetryQualityResult {
        status: TelemetryQualityStatus::Valid,
        reason: None,
    }
}

pub fn calculate_median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }

    let mut sorted_values = values.to_vec();

    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mid = sorted_values.len() / 2;

    if sorted_values.len() % 2 == 0 {
        Some((sorted_values[mid - 1] + sorted_values[mid]) / 2.0)
    } else {
        Some(sorted_values[mid])
    }
}

pub fn calculate_rolling_median(values: &[f64], window_size: usize) -> Option<f64> {
    if values.is_empty() || window_size == 0 {
        return None;
    }

    let start_index = values.len().saturating_sub(window_size);
    let window = &values[start_index..];

    calculate_median(window)
}

pub fn calculate_iqr(values: &[f64]) -> Option<f64> {
    if values.len() < 4 {
        return None;
    }

    let mut sorted_values = values.to_vec();

    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mid = sorted_values.len() / 2;

    let lower_half = &sorted_values[..mid];

    let upper_half = if sorted_values.len() % 2 == 0 {
        &sorted_values[mid..]
    } else {
        &sorted_values[mid + 1..]
    };

    let q1 = calculate_median(lower_half)?;
    let q3 = calculate_median(upper_half)?;

    Some(q3 - q1)
}

pub fn is_outlier_using_iqr(values: &[f64], candidate: f64, multiplier: f64) -> bool {
    if values.len() < 4 {
        return false;
    }

    let mut sorted_values = values.to_vec();

    sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let mid = sorted_values.len() / 2;

    let lower_half = &sorted_values[..mid];

    let upper_half = if sorted_values.len() % 2 == 0 {
        &sorted_values[mid..]
    } else {
        &sorted_values[mid + 1..]
    };

    let Some(q1) = calculate_median(lower_half) else {
        return false;
    };

    let Some(q3) = calculate_median(upper_half) else {
        return false;
    };

    let iqr = q3 - q1;

    let statistical_tolerance = multiplier * iqr;

    let effective_tolerance = statistical_tolerance.max(MIN_FUEL_OUTLIER_TOLERANCE_LITRES);

    let lower_bound = q1 - effective_tolerance;
    let upper_bound = q3 + effective_tolerance;

    candidate < lower_bound || candidate > upper_bound
}

pub fn count_iqr_outliers(
    baseline_values: &[f64],
    candidate_values: &[f64],
    multiplier: f64,
) -> usize {
    candidate_values
        .iter()
        .filter(|candidate| is_outlier_using_iqr(baseline_values, **candidate, multiplier))
        .count()
}

#[derive(Debug, Clone)]
pub struct FuelQualityWindowSummary {
    pub rolling_median: Option<f64>,
    pub iqr: Option<f64>,
    pub outlier_count: usize,
    pub candidate_count: usize,
}

pub fn evaluate_fuel_quality_window(
    baseline_values: &[f64],
    candidate_values: &[f64],
    rolling_window_size: usize,
    iqr_multiplier: f64,
) -> FuelQualityWindowSummary {
    let rolling_median = calculate_rolling_median(baseline_values, rolling_window_size);

    let iqr = calculate_iqr(baseline_values);

    let outlier_count = count_iqr_outliers(baseline_values, candidate_values, iqr_multiplier);

    FuelQualityWindowSummary {
        rolling_median,
        iqr,
        outlier_count,
        candidate_count: candidate_values.len(),
    }
}

#[test]
fn evaluates_fuel_quality_window() {
    let baseline_values = vec![180.0, 179.0, 178.0, 177.0, 176.0];
    let candidate_values = vec![120.0, 119.0, 118.0];

    let summary = evaluate_fuel_quality_window(&baseline_values, &candidate_values, 3, 1.5);

    assert_eq!(summary.rolling_median, Some(177.0));
    assert_eq!(summary.iqr, Some(3.0));
    assert_eq!(summary.outlier_count, 3);
    assert_eq!(summary.candidate_count, 3);
}

#[test]
fn counts_persistent_iqr_outliers() {
    let baseline_values = vec![180.0, 179.0, 178.0, 177.0, 176.0];
    let candidate_values = vec![120.0, 119.0, 118.0];

    let count = count_iqr_outliers(&baseline_values, &candidate_values, 1.5);

    assert_eq!(count, 3);
}

#[test]
fn ignores_normal_candidate_values() {
    let baseline_values = vec![180.0, 179.0, 178.0, 177.0, 176.0];
    let candidate_values = vec![179.0, 178.0, 177.0];

    let count = count_iqr_outliers(&baseline_values, &candidate_values, 1.5);

    assert_eq!(count, 0);
}

#[test]
fn calculates_iqr_for_valid_values() {
    let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];

    let iqr = calculate_iqr(&values);

    assert_eq!(iqr, Some(30.0));
}

#[test]
fn returns_none_for_iqr_with_too_few_values() {
    let values = vec![10.0, 20.0, 30.0];

    let iqr = calculate_iqr(&values);

    assert_eq!(iqr, None);
}

#[test]
fn calculates_rolling_median_from_last_window() {
    let values = vec![180.0, 179.0, 20.0, 178.0, 177.0];

    let median = calculate_rolling_median(&values, 3);

    assert_eq!(median, Some(177.0));
}

#[test]
fn returns_none_when_rolling_median_values_are_empty() {
    let values: Vec<f64> = vec![];

    let median = calculate_rolling_median(&values, 3);

    assert_eq!(median, None);
}

#[test]
fn returns_none_when_rolling_median_window_is_zero() {
    let values = vec![180.0, 179.0, 178.0];

    let median = calculate_rolling_median(&values, 0);

    assert_eq!(median, None);
}

#[test]
fn calculates_median_for_odd_number_of_values() {
    let values = vec![180.0, 179.0, 20.0, 178.0, 177.0];

    let median = calculate_median(&values);

    assert_eq!(median, Some(178.0));
}

#[test]
fn calculates_median_for_even_number_of_values() {
    let values = vec![180.0, 178.0, 176.0, 174.0];

    let median = calculate_median(&values);

    assert_eq!(median, Some(177.0));
}

#[test]
fn returns_none_for_empty_median_input() {
    let values: Vec<f64> = vec![];

    let median = calculate_median(&values);

    assert_eq!(median, None);
}

#[test]
fn detects_impossible_fuel_jump() {
    let result = detect_impossible_fuel_jump(180.0, 20.0, 50.0);

    assert_eq!(result.status, TelemetryQualityStatus::Suspicious);
}

#[test]
fn accepts_normal_fuel_change() {
    let result = detect_impossible_fuel_jump(180.0, 175.0, 50.0);

    assert_eq!(result.status, TelemetryQualityStatus::Valid);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_fuel_range() {
        let result = validate_fuel_range(120.0, 200.0);

        assert_eq!(result.status, TelemetryQualityStatus::Valid);
        assert_eq!(result.reason, None);
    }

    #[test]
    fn rejects_negative_fuel_level() {
        let result = validate_fuel_range(-5.0, 200.0);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
        assert_eq!(
            result.reason,
            Some("Fuel level cannot be negative.".to_string())
        );
    }

    #[test]
    fn rejects_fuel_above_tank_capacity() {
        let result = validate_fuel_range(250.0, 200.0);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
        assert_eq!(
            result.reason,
            Some("Fuel level exceeds tank capacity.".to_string())
        );
    }

    #[test]
    fn zero_iqr_baseline_accepts_small_sensor_noise() {
        let baseline_values = vec![
            1.2046812749003983,
            1.2046812749003983,
            1.2046812749003983,
            1.2046812749003983,
            1.2046812749003983,
        ];

        let candidate = 1.2006972111553786;

        let is_outlier = is_outlier_using_iqr(&baseline_values, candidate, 1.5);

        assert!(!is_outlier);
    }

    #[test]
    fn zero_iqr_baseline_still_detects_large_fuel_change_as_outlier() {
        let baseline_values = vec![
            1.2046812749003983,
            1.2046812749003983,
            1.2046812749003983,
            1.2046812749003983,
            1.2046812749003983,
        ];

        let candidate = 1.150000;

        let is_outlier = is_outlier_using_iqr(&baseline_values, candidate, 1.5);

        assert!(is_outlier);
    }

    #[test]
    fn rejects_nan_fuel_level() {
        let result = validate_fuel_range(f64::NAN, 200.0);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
    }

    #[test]
    fn rejects_infinite_fuel_level() {
        let result = validate_fuel_range(f64::INFINITY, 200.0);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
    }

    #[test]
    fn rejects_invalid_tank_capacity() {
        let result = validate_fuel_range(100.0, f64::NAN);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
    }

    #[test]
    fn rejects_nan_previous_value_in_jump_detection() {
        let result = detect_impossible_fuel_jump(f64::NAN, 100.0, 50.0);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
    }

    #[test]
    fn rejects_nan_current_value_in_jump_detection() {
        let result = detect_impossible_fuel_jump(100.0, f64::NAN, 50.0);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
    }

    #[test]
    fn rejects_invalid_max_allowed_jump() {
        let result = detect_impossible_fuel_jump(100.0, 120.0, f64::NAN);

        assert_eq!(result.status, TelemetryQualityStatus::Invalid);
    }
}

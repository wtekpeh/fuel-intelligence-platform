use chrono::{DateTime, Utc};

pub fn classify_device_status(
    last_seen_at: Option<DateTime<Utc>>,
    stale_after_seconds: i64,
    offline_after_seconds: i64,
) -> String {
    let Some(last_seen_at) = last_seen_at else {
        return "UNKNOWN".to_string();
    };

    let age_seconds = (Utc::now() - last_seen_at).num_seconds();

    if age_seconds <= stale_after_seconds {
        "ONLINE".to_string()
    } else if age_seconds <= offline_after_seconds {
        "STALE".to_string()
    } else {
        "OFFLINE".to_string()
    }
}

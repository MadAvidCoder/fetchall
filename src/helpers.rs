use chrono::{DateTime, Utc};

pub fn format_relative_time(epoch_seconds: i64) -> String {
    let now = Utc::now();
    let Some(time) = DateTime::from_timestamp(epoch_seconds, 0) else {
        return "Invalid time".to_string();
    };
    let diff = now.signed_duration_since(time);

    let seconds = diff.num_seconds();
    if seconds < 0 {
        return "In the future".to_string();
    }

    let minutes = diff.num_minutes();
    let hours = diff.num_hours();
    let days = diff.num_days();
    let weeks = days / 7;
    let months = days / 30;
    let years = days / 365;
    if seconds < 60 || minutes == 0 {
        "Just now".to_string()
    } else if minutes < 60 || hours == 0 {
        format!("{}m ago", minutes)
    } else if hours < 24 || days == 0 {
        format!("{}h ago", hours)
    } else if days < 7 || weeks == 0 {
        format!("{}d ago", days)
    } else if weeks < 4 || months == 0 {
        format!("{}w ago", weeks)
    } else if months < 12 || years == 0 {
        format!("{}mo ago", months)
    } else {
        format!("{}y ago", years)
    }
}

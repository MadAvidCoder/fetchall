use std::process::Command;

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

pub fn execute_command(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let text = if !stdout.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };

    Some(text.lines().next()?.trim().to_string())
}

pub fn deg_to_cardinal(deg: f64) -> &'static str {
    match deg {
        348.75..=360.0 | 0.0..=11.25 => "N",
        11.25..=33.75 => "NNE",
        33.75..=56.25 => "NE",
        56.25..=78.75 => "ENE",
        78.75..=101.25 => "E",
        101.25..=123.75 => "ESE",
        123.75..=146.25 => "SE",
        146.25..=168.75 => "SSE",
        168.75..=191.25 => "S",
        191.25..=213.75 => "SSW",
        213.75..=236.25 => "SW",
        236.25..=258.75 => "WSW",
        258.75..=281.25 => "W",
        281.25..=303.75 => "WNW",
        303.75..=326.25 => "NW",
        326.25..=348.75 => "NNW",
        _ => unreachable!(),
    }
}

use std::time::{SystemTime, UNIX_EPOCH};

/// Format a Unix timestamp as a human-readable relative time.
pub fn format_relative(ts: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let diff = now - ts;

    if diff < 0 {
        return "in the future".to_string();
    }

    match diff {
        0..=59 => format!("{} second{} ago", diff, plural(diff)),
        60..=3599 => {
            let m = diff / 60;
            format!("{} minute{} ago", m, plural(m))
        }
        3600..=86399 => {
            let h = diff / 3600;
            format!("{} hour{} ago", h, plural(h))
        }
        86400..=2591999 => {
            let d = diff / 86400;
            format!("{} day{} ago", d, plural(d))
        }
        2592000..=31535999 => {
            let mo = diff / 2592000;
            format!("{} month{} ago", mo, plural(mo))
        }
        _ => {
            let y = diff / 31536000;
            format!("{} year{} ago", y, plural(y))
        }
    }
}

/// Format a Unix timestamp as an ISO-8601 date string (UTC).
pub fn format_iso(ts: i64) -> String {
    // Manual UTC date formatting without external deps
    let secs = ts as u64;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    let (year, month, day) = days_to_ymd(days_since_epoch);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, h, m, s
    )
}

/// Return the short hash (first 7 characters).
pub fn short_hash(oid: &str) -> &str {
    let end = oid.len().min(7);
    &oid[..end]
}

fn plural(n: i64) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Convert days since Unix epoch (1970-01-01) to (year, month, day).
fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Simplified Gregorian calendar calculation
    let mut year = 1970u64;
    loop {
        let y_days = if is_leap(year) { 366 } else { 365 };
        if days < y_days {
            break;
        }
        days -= y_days;
        year += 1;
    }
    let month_days = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_short_hash() {
        assert_eq!(short_hash("abc1234def567"), "abc1234");
        assert_eq!(short_hash("abc"), "abc");
    }

    #[test]
    fn test_format_iso_epoch() {
        assert_eq!(format_iso(0), "1970-01-01 00:00:00 UTC");
    }
}

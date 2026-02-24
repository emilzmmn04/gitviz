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
    let ts = ts as i128;
    let days_since_epoch = ts.div_euclid(86_400);
    let time_of_day = ts.rem_euclid(86_400);
    let h = time_of_day / 3_600;
    let m = (time_of_day % 3_600) / 60;
    let s = time_of_day % 60;

    let (year, month, day) = civil_from_days(days_since_epoch);
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

/// Convert days since Unix epoch (1970-01-01) to (year, month, day) in UTC.
/// Uses constant-time Gregorian conversion with integer arithmetic.
fn civil_from_days(days_since_epoch: i128) -> (i128, i128, i128) {
    // Shift from Unix epoch (1970-01-01) to civil epoch (0000-03-01).
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let month = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
    if month <= 2 {
        year += 1;
    }

    (year, month, day)
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

    #[test]
    fn test_format_iso_negative_second() {
        assert_eq!(format_iso(-1), "1969-12-31 23:59:59 UTC");
    }

    #[test]
    fn test_format_iso_extreme_timestamps() {
        let min = format_iso(i64::MIN);
        let max = format_iso(i64::MAX);
        assert!(min.ends_with(" UTC"));
        assert!(max.ends_with(" UTC"));
        assert!(min.contains(':'));
        assert!(max.contains(':'));
    }
}

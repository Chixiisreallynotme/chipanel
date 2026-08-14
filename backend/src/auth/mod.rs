pub mod jwt;
pub mod middleware;
pub mod password;
pub mod profiles;
pub mod tokens;
pub mod users;

/// Format a Unix timestamp as `YYYY-MM-DD HH:MM UTC`.
///
/// Deliberately naive (365-day years, 30-day months) — this exact output is
/// persisted to the on-disk JSON stores and compared lexicographically by
/// `tokens::is_expired`, so changing the format or the arithmetic is a
/// data-migration, not a cleanup.
pub(crate) fn format_timestamp(ts: u64) -> String {
    let days = ts / 86400;
    let seconds_into_day = ts % 86400;
    let hours = seconds_into_day / 3600;
    let minutes = (seconds_into_day % 3600) / 60;

    let year = 1970 + (days / 365);
    let day_of_year = days % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    format!("{:04}-{:02}-{:02} {:02}:{:02} UTC", year, month, day, hours, minutes)
}

/// Current time as an ISO-8601-like `YYYY-MM-DD HH:MM UTC` stamp.
pub(crate) fn now_iso_like() -> String {
    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format_timestamp(now_ts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_iso_like_keeps_the_persisted_stamp_shape() {
        let s = now_iso_like();
        assert_eq!(s.len(), 20, "stamp shape changed: {}", s);
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
        assert_eq!(&s[10..11], " ");
        assert_eq!(&s[13..14], ":");
        assert_eq!(&s[16..], " UTC");
        assert!(s[..4].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn format_timestamp_is_lexicographically_ordered() {
        // is_expired() compares these strings with `>`, so ordering must hold.
        assert!(format_timestamp(2_000_000_000) > format_timestamp(1_000_000_000));
        assert_eq!(format_timestamp(0), "1970-01-01 00:00 UTC");
    }
}

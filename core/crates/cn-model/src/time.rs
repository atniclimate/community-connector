use serde::{Deserialize, Serialize};

/// Milliseconds since the Unix epoch, injected by callers - the core never
/// reads a system clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(pub i64);

/// Parses an ISO-8601 UTC instant (`new Date().toISOString()` shape,
/// `YYYY-MM-DDTHH:MM:SS[.sss]Z`) to Unix milliseconds. Returns `None` on any
/// deviation from that exact shape - a focused parser for the ONE format the
/// browser/JS side emits, deliberately hand-rolled rather than pulling in a date
/// crate (consistent with the CLI's keymat.rs ISO handling).
///
/// Shared home (D-088): the remote intake path emits ISO-STRING timestamps
/// (`form/src/envelope.ts`; Rust `InnerPayload.consent.consent_affirmed_at` is a
/// `String`), while the in-app path emits epoch-ms NUMBERS. The puller uses this
/// to read the relay's `arrived_at`, and the durable owner (cn-ingest
/// `approval.rs`) uses it to coerce a remote submission's ISO consent/capture
/// timestamps into the epoch-ms the `IntakeProvenance` block stores.
pub fn parse_iso8601_utc_to_unix_ms(text: &str) -> Option<i64> {
    let text = text.strip_suffix('Z')?;
    let (date, time) = text.split_once('T')?;

    let mut date_parts = date.split('-');
    let year = parse_fixed(date_parts.next()?, 4)? as i64;
    let month = parse_fixed(date_parts.next()?, 2)?;
    let day = parse_fixed(date_parts.next()?, 2)?;
    if date_parts.next().is_some() {
        return None;
    }

    let (hms, millis) = match time.split_once('.') {
        Some((hms, frac)) => (hms, parse_millis(frac)?),
        None => (time, 0),
    };
    let mut time_parts = hms.split(':');
    let hour = parse_fixed(time_parts.next()?, 2)? as i64;
    let minute = parse_fixed(time_parts.next()?, 2)? as i64;
    let second = parse_fixed(time_parts.next()?, 2)? as i64;
    if time_parts.next().is_some() {
        return None;
    }

    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Allow a leap second (60); reject anything further out of range.
    if hour > 23 || minute > 59 || second > 60 {
        return None;
    }

    let days = days_from_civil(year, month, day);
    // Reject calendar-impossible dates (Feb 31, Feb 29 in a common year, Apr 31,
    // ...). The month/day range check above is month-agnostic, and
    // `days_from_civil` NORMALIZES an impossible date to a valid nearby one (e.g.
    // 2024-02-31 -> 2024-03-02) rather than failing - which would silently yield a
    // WRONG epoch and break the "None on any deviation" contract (D-088: malformed
    // still fails). Round-trip the serial day back to a civil date; if it does not
    // equal the input, the date does not exist.
    if civil_from_days(days) != (year, month, day) {
        return None;
    }
    let secs = days * 86_400 + hour * 3600 + minute * 60 + second;
    Some(secs * 1000 + millis)
}

/// Parses a fixed-width, all-ASCII-digit numeric group (rejects `+`, spaces, and
/// short/long groups so a malformed timestamp fails rather than mis-parses).
fn parse_fixed(group: &str, width: usize) -> Option<u32> {
    if group.len() != width || !group.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    group.parse().ok()
}

/// Parses the fractional-seconds part to whole milliseconds (first three digits,
/// zero-padded). `toISOString` always emits exactly three, but this tolerates
/// 1-9 digits.
fn parse_millis(frac: &str) -> Option<i64> {
    if frac.is_empty() || !frac.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let mut millis = 0i64;
    for index in 0..3 {
        let digit = frac
            .as_bytes()
            .get(index)
            .map_or(0, |byte| (byte - b'0') as i64);
        millis = millis * 10 + digit;
    }
    Some(millis)
}

/// Howard Hinnant's `days_from_civil`: (year, month, day) in the proleptic
/// Gregorian calendar -> days since 1970-01-01.
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = (if year >= 0 { year } else { year - 399 }) / 400;
    let year_of_era = year - era * 400; // [0, 399]
    let month = month as i64;
    let day = day as i64;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1; // [0, 365]
    let doe = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + doy; // [0, 146096]
    era * 146_097 + doe - 719_468
}

/// Howard Hinnant's `civil_from_days`: the exact inverse of [`days_from_civil`]
/// over valid dates (days since 1970-01-01 -> `(year, month, day)`). Used only to
/// round-trip-validate a parsed date: an impossible input maps forward to a
/// serial day whose canonical civil form is a DIFFERENT (valid) date, so the
/// mismatch rejects it. Kept local to this crate rather than reaching into the
/// CLI's `keymat.rs` copy (cn-model has no dependency on the CLI crate).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let year_of_era = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // [0, 399]
    let year = year_of_era + era * 400;
    let doy = doe - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32; // [1, 12]
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iso_parser_pins_known_instants() {
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("1970-01-01T00:00:00Z"),
            Some(0)
        );
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("2000-01-01T00:00:00.000Z"),
            Some(946_684_800_000)
        );
        // +59 days from 2024-01-01 lands on the leap day.
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("2024-02-29T00:00:00.000Z"),
            Some(1_709_164_800_000)
        );
        // Time-of-day plus milliseconds.
        assert_eq!(
            parse_iso8601_utc_to_unix_ms("2000-01-01T13:45:30.999Z"),
            Some(946_684_800_000 + 49_530_999)
        );
    }

    #[test]
    fn iso_parser_rejects_malformed() {
        assert_eq!(parse_iso8601_utc_to_unix_ms(""), None);
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-01-01T00:00:00"), None); // no Z
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-13-01T00:00:00Z"), None); // month 13
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-01-01 00:00:00Z"), None); // space, no T
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-1-01T00:00:00Z"), None); // short month
        assert_eq!(parse_iso8601_utc_to_unix_ms("2000-01-01T25:00:00Z"), None); // hour 25
        // Calendar-impossible dates: in-range day/month, but the date does not
        // exist. days_from_civil would NORMALIZE these into a wrong epoch (R5-1),
        // so the round-trip must reject them rather than return Some.
        assert_eq!(parse_iso8601_utc_to_unix_ms("2023-02-29T00:00:00Z"), None); // Feb 29, common year
        assert_eq!(parse_iso8601_utc_to_unix_ms("2024-02-30T00:00:00Z"), None); // Feb 30
        assert_eq!(parse_iso8601_utc_to_unix_ms("2024-04-31T00:00:00Z"), None); // Apr 31 (30-day month)
    }

    #[test]
    fn days_from_civil_matches_known_epochs() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 1, 1), 10_957);
        assert_eq!(days_from_civil(1969, 12, 31), -1);
    }
}

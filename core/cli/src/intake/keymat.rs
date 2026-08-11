//! Shared key-material helpers for the `cn intake` keygen ceremony family
//! (blueprint docs/blueprints/intake-relay.md section 3;
//! docs/design/facilitator-keygen-ceremony.md sections 4-5).
//!
//! One place for the four subcommands' common concerns, so passphrase handling
//! and the self-test property are proven once rather than four times (mirroring
//! how queue.rs is the shared helper apply.rs builds on): the hidden-echo
//! passphrase read, the single shared self-test vector and its round-trip
//! proof, the printed-backup base32+CRC codec, and ISO-8601 `created_at`
//! stamping.
//!
//! Everything here is CLI-only ceremony/UX tooling. The persisted-format and
//! crypto logic lives in cn-ingest; base32 is deliberately kept out of
//! cn-ingest (blueprint section 9). The CLI holds plaintext key bytes only
//! transiently - the base32 decode buffer and the `from_bytes` input - and
//! relies on being a short-lived offline ceremony step; the durable in-memory
//! `SecretKey` is zeroize-on-drop in cn-ingest.

use std::io::{IsTerminal, Write};

use base32::Alphabet;
use cn_ingest::{
    IngestError, KeyMetadata, Keypair, SECRET_KEY_LEN, SecretKey, generate_keypair, open, seal,
};
use zeroize::Zeroizing;

/// Minimum passphrase word count (ceremony design section 4: a diceware-style
/// phrase of at least 6 words). Enforced at generation so a too-weak passphrase
/// never protects the single most sensitive artifact in the system.
const MIN_PASSPHRASE_WORDS: usize = 6;

/// The single shared self-test vector. `selftest` and `backup verify` both seal
/// and open THIS exact value, so an operator running the ceremony trusts that
/// both commands prove the identical property (blueprint section 3).
const SELFTEST_VECTOR: &[u8] = b"cn-intake-selftest-vector-v1";

/// Stable stderr line prefixes for the printed-backup block. Documented and
/// tested: `cn intake backup verify --from-print` parses these exact prefixes
/// back out of a transcribed sheet, and the keygen tests parse them from stderr.
pub(crate) const PRINT_BACKUP_BASE32_PREFIX: &str = "print-backup base32: ";
pub(crate) const PRINT_BACKUP_CHECK_PREFIX: &str = "print-backup check: ";

/// Reads a fresh passphrase for a NEW key: a hidden-echo prompt, a confirmation
/// prompt, mismatch rejection, and the >= 6-word minimum. Confirmation is not
/// spelled out in the blueprint's one-line `keygen` description, but a lost
/// passphrase is unrecoverable data loss (ceremony section 1), and an
/// unconfirmed typo during generation IS a lost-key event the operator would
/// not discover until too late. On any rejection the caller has written nothing
/// to disk yet - nothing reaches the filesystem.
pub(crate) fn read_new_passphrase(err: &mut dyn Write) -> Result<Zeroizing<String>, String> {
    let first = prompt_secret(err, "Enter a new intake-key passphrase (>= 6 words): ")?;
    let confirm = prompt_secret(err, "Confirm the passphrase: ")?;
    // Deref to compare the inner strings: Zeroizing wraps them so both copies
    // (and confirm) are wiped on drop, matching cn-ingest's zeroized key material.
    if *first != *confirm {
        return Err("passphrase entries did not match; nothing was written".to_string());
    }
    if first.split_whitespace().count() < MIN_PASSPHRASE_WORDS {
        return Err(format!(
            "passphrase must be at least {MIN_PASSPHRASE_WORDS} whitespace-separated words \
             (ceremony design section 4); nothing was written"
        ));
    }
    Ok(first)
}

/// Reads an existing key's passphrase with a single hidden-echo prompt (no
/// confirmation: this unlocks an already-written key, it does not create one).
pub(crate) fn read_existing_passphrase(
    err: &mut dyn Write,
    purpose: &str,
) -> Result<Zeroizing<String>, String> {
    prompt_secret(err, &format!("Enter the passphrase to {purpose}: "))
}

/// Writes `prompt` to the controllable `err` stream, then reads one secret line.
///
/// When stdin is a real terminal (the live ceremony), rpassword reads it from
/// the TTY with echo disabled. When stdin is not a terminal (tests and scripted
/// ceremonies), there is no echo to hide, so read one line from the buffered
/// global stdin - which preserves its buffer across sequential calls, so a
/// passphrase followed by its confirmation reads back as two lines. rpassword's
/// default reads `/dev/tty`/`CONIN$`, not stdin, so it alone cannot serve the
/// non-interactive path; this branch supplies it (I3: a closed stdin is a loud
/// error, never a silent empty passphrase).
fn prompt_secret(err: &mut dyn Write, prompt: &str) -> Result<Zeroizing<String>, String> {
    write!(err, "{prompt}")
        .map_err(|io_err| format!("cannot write passphrase prompt: {io_err}"))?;
    err.flush()
        .map_err(|io_err| format!("cannot flush passphrase prompt: {io_err}"))?;
    if std::io::stdin().is_terminal() {
        rpassword::read_password()
            .map(Zeroizing::new)
            .map_err(|io_err| format!("cannot read passphrase: {io_err}"))
    } else {
        // The untrimmed read buffer holds the passphrase too, so it is zeroized
        // on drop alongside the returned copy.
        let mut line = Zeroizing::new(String::new());
        let read = std::io::stdin()
            .read_line(&mut line)
            .map_err(|io_err| format!("cannot read passphrase from stdin: {io_err}"))?;
        if read == 0 {
            return Err("no passphrase available on stdin (input closed)".to_string());
        }
        Ok(Zeroizing::new(
            line.trim_end_matches(['\r', '\n']).to_string(),
        ))
    }
}

/// Proves the real production decrypt path end to end for `keypair`: seal the
/// shared vector to its public half and open it with the pair. A failed open,
/// or a round trip that does not reproduce the vector, is a content-free crypto
/// failure (I3) - a self-test that cannot prove the property must fail loudly,
/// never pass quietly.
pub(crate) fn round_trip(keypair: &Keypair) -> Result<(), IngestError> {
    let sealed = seal(SELFTEST_VECTOR, &keypair.public);
    let opened = open(&sealed, keypair)?;
    if opened == SELFTEST_VECTOR {
        Ok(())
    } else {
        Err(IngestError::Crypto)
    }
}

/// Uppercase RFC4648 base32, no padding: the standard alphabet for the physical
/// sheet (and its QR). Decoding uppercases and strips whitespace first, so a
/// hand-transcribed lower/upper mix with readability spaces still decodes.
fn print_backup_alphabet() -> Alphabet {
    Alphabet::Rfc4648 { padding: false }
}

/// Encodes a secret key for the printed backup: base32 of the raw 32 bytes plus
/// a CRC-32 check value over those same bytes. Returns `(base32, check-hex)`.
pub(crate) fn encode_print_backup(secret: &SecretKey) -> (String, String) {
    let raw = secret.expose_bytes();
    let base32 = base32::encode(print_backup_alphabet(), raw);
    let check = format!("{:08x}", crc32fast::hash(raw));
    (base32, check)
}

/// Decodes a printed-backup base32 blob under its check line. The check is
/// verified BEFORE any key is reconstructed (I3): malformed base32, a wrong
/// decoded length, or a failed CRC are each rejected loudly, so a typo in a
/// hand-transcribed sheet never yields a silently-wrong key.
pub(crate) fn decode_print_backup(
    base32_text: &str,
    check_text: &str,
) -> Result<SecretKey, String> {
    let expected = u32::from_str_radix(check_text.trim(), 16)
        .map_err(|_| "printed-backup check line is not 8-digit hex".to_string())?;
    let cleaned: String = base32_text
        .chars()
        .filter(|c| !c.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_uppercase();
    let bytes = base32::decode(print_backup_alphabet(), &cleaned).ok_or_else(|| {
        "printed-backup base32 does not decode (transcription error?)".to_string()
    })?;
    if bytes.len() != SECRET_KEY_LEN {
        return Err(format!(
            "printed-backup decodes to {} bytes, expected {SECRET_KEY_LEN}",
            bytes.len()
        ));
    }
    let actual = crc32fast::hash(&bytes);
    if actual != expected {
        return Err(format!(
            "printed-backup check mismatch (computed {actual:08x}, sheet says {expected:08x}); \
             refusing to reconstruct a key from a mistyped backup (I3)"
        ));
    }
    let mut raw = [0u8; SECRET_KEY_LEN];
    raw.copy_from_slice(&bytes);
    Ok(SecretKey::from_bytes(raw))
}

/// Parses a transcribed printed-backup file: scans for the stable
/// `print-backup base32:` and `print-backup check:` lines and decodes the key
/// under its CRC. Used by `cn intake backup verify --from-print`.
pub(crate) fn parse_print_backup_file(contents: &str) -> Result<SecretKey, String> {
    let base32 = find_prefixed(contents, PRINT_BACKUP_BASE32_PREFIX)
        .ok_or_else(|| format!("printed-backup file has no '{PRINT_BACKUP_BASE32_PREFIX}' line"))?;
    let check = find_prefixed(contents, PRINT_BACKUP_CHECK_PREFIX)
        .ok_or_else(|| format!("printed-backup file has no '{PRINT_BACKUP_CHECK_PREFIX}' line"))?;
    decode_print_backup(base32, check)
}

fn find_prefixed<'a>(contents: &'a str, prefix: &str) -> Option<&'a str> {
    contents
        .lines()
        .find_map(|line| line.trim_start().strip_prefix(prefix))
}

/// Stamps a `KeyMetadata` with the current UTC time as ISO-8601
/// (`YYYY-MM-DDTHH:MM:SSZ`). cn-ingest has no clock; the CLI supplies the
/// timestamp. Hand-rolled from millis-since-epoch (below) rather than adding a
/// date crate to this sensitive tool.
pub(crate) fn key_metadata_now() -> Result<KeyMetadata, String> {
    let now_ms = crate::export::unix_now_ms()?;
    Ok(KeyMetadata {
        created_at: format_iso8601_utc(now_ms),
    })
}

/// Generates a fresh ephemeral keypair for a filesystem-free self-test
/// (`selftest --dry`).
pub(crate) fn ephemeral_keypair() -> Keypair {
    generate_keypair()
}

/// Formats millis-since-epoch as `YYYY-MM-DDTHH:MM:SSZ` (UTC, seconds
/// precision). Sub-second millis are truncated.
fn format_iso8601_utc(unix_ms: i64) -> String {
    let secs = unix_ms.div_euclid(1000);
    let days = secs.div_euclid(86_400);
    let secs_of_day = secs.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Howard Hinnant's `civil_from_days`: days-since-1970-01-01 (proleptic
/// Gregorian) -> (year, month, day). `div_euclid` gives the floor division the
/// algorithm needs for negative days.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let year = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    (if month <= 2 { year + 1 } else { year }, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_backup_round_trips() {
        let kp = generate_keypair();
        let (base32, check) = encode_print_backup(&kp.secret);
        let recovered = decode_print_backup(&base32, &check).expect("decode");
        assert_eq!(recovered.expose_bytes(), kp.secret.expose_bytes());
    }

    #[test]
    fn print_backup_tolerates_whitespace_and_case() {
        let kp = generate_keypair();
        let (base32, check) = encode_print_backup(&kp.secret);
        // A hand transcription might lower-case and add readability spaces.
        let messy = format!("{}  {}", &base32[..26].to_lowercase(), &base32[26..]);
        let recovered = decode_print_backup(&messy, &format!("  {check}  ")).expect("decode");
        assert_eq!(recovered.expose_bytes(), kp.secret.expose_bytes());
    }

    #[test]
    fn print_backup_rejects_single_char_corruption() {
        let kp = generate_keypair();
        let (base32, check) = encode_print_backup(&kp.secret);
        // Flip one base32 character to a different valid symbol.
        let mut chars: Vec<char> = base32.chars().collect();
        chars[0] = if chars[0] == 'A' { 'B' } else { 'A' };
        let corrupted: String = chars.into_iter().collect();
        let err = decode_print_backup(&corrupted, &check).expect_err("must reject");
        assert!(err.contains("check mismatch"), "loud CRC rejection: {err}");
    }

    #[test]
    fn print_backup_rejects_truncated_input() {
        let kp = generate_keypair();
        let (base32, check) = encode_print_backup(&kp.secret);
        let err = decode_print_backup(&base32[..base32.len() - 8], &check).expect_err("reject");
        assert!(
            err.contains("bytes, expected") || err.contains("check mismatch"),
            "truncation rejected loudly: {err}"
        );
    }

    #[test]
    fn print_backup_rejects_non_hex_check() {
        let kp = generate_keypair();
        let (base32, _) = encode_print_backup(&kp.secret);
        let err = decode_print_backup(&base32, "not-hex-").expect_err("reject");
        assert!(
            err.contains("8-digit hex"),
            "check-line format enforced: {err}"
        );
    }

    #[test]
    fn parse_print_backup_file_reads_the_labeled_lines() {
        let kp = generate_keypair();
        let (base32, check) = encode_print_backup(&kp.secret);
        let file = format!(
            "some preamble\n{PRINT_BACKUP_BASE32_PREFIX}{base32}\n{PRINT_BACKUP_CHECK_PREFIX}{check}\ntrailer\n"
        );
        let recovered = parse_print_backup_file(&file).expect("parse");
        assert_eq!(recovered.expose_bytes(), kp.secret.expose_bytes());
    }

    #[test]
    fn parse_print_backup_file_reports_missing_lines() {
        let err = parse_print_backup_file("nothing useful here").expect_err("reject");
        assert!(
            err.contains("print-backup base32:"),
            "names the missing line: {err}"
        );
    }

    #[test]
    fn round_trip_proves_a_generated_keypair() {
        let kp = generate_keypair();
        round_trip(&kp).expect("a fresh keypair round-trips the self-test vector");
    }

    #[test]
    fn iso8601_pins_known_epochs() {
        assert_eq!(format_iso8601_utc(0), "1970-01-01T00:00:00Z");
        // 946684800 s = 2000-01-01T00:00:00Z (a widely-memorized anchor).
        assert_eq!(format_iso8601_utc(946_684_800_000), "2000-01-01T00:00:00Z");
        // 1704067200 s = 2024-01-01T00:00:00Z; +59 days lands on the leap day.
        assert_eq!(
            format_iso8601_utc(1_709_164_800_000),
            "2024-02-29T00:00:00Z"
        );
        // Time-of-day and sub-second truncation.
        assert_eq!(
            format_iso8601_utc(946_684_800_000 + (13 * 3600 + 45 * 60 + 30) * 1000 + 999),
            "2000-01-01T13:45:30Z"
        );
    }
}

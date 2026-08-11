//! `cn intake` keygen-ceremony battery (blueprint intake-relay.md section 3
//! step-2 acceptance bar; ceremony design sections 4-5). Every interactive
//! prompt is driven by PIPED stdin (never a real TTY assumption), so the suite
//! runs unattended: a design that blocked on a terminal would hang instead of
//! failing cleanly. All key material here is generated fresh in tempdirs and
//! never leaves them; nothing is committed (I1).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

/// A synthetic 6-word ceremony passphrase (test-only; never operational).
const PASS: &str = "correct horse battery staple river delta";

const BASE32_PREFIX: &str = "print-backup base32: ";
const CHECK_PREFIX: &str = "print-backup check: ";

/// A synthetic printed-backup sheet for the deterministic 32-byte key
/// `[1, 2, ..., 32]` (test-only; never operational), computed once with the same
/// RFC4648-base32 + CRC-32 codec keymat.rs uses, so `backup verify --from-print`
/// decodes it end to end. keygen now WITHHOLDS the plaintext backup block when
/// stderr is not a TTY (R1-3) - which is exactly how this suite spawns `cn` - so
/// the from-print tests can no longer harvest a live block from keygen's stderr
/// and use this fixed sheet instead. Any 32 bytes are a valid X25519 secret, so
/// the recovered pair still round-trips the self-test vector.
const SYNTH_PRINT_BASE32: &str = "AEBAGBAFAYDQQCIKBMGA2DQPCAIREEYUCULBOGAZDINRYHI6D4QA";
const SYNTH_PRINT_CHECK: &str = "87e6ec25";

/// Spawns the real `cn` binary, feeds `stdin_data`, and collects the output.
/// Write errors are ignored: a command that refuses BEFORE reading stdin (a
/// create-only refusal, a usage error) closes the pipe early, which is not a
/// test failure.
fn cn_stdin(args: &[&str], stdin_data: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cn"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("cn spawns");
    let mut stdin = child.stdin.take().expect("child stdin");
    let _ = stdin.write_all(stdin_data.as_bytes());
    drop(stdin); // EOF
    child.wait_with_output().expect("cn completes")
}

/// Spawns the real `cn` binary with a null stdin (for commands that never read
/// a passphrase, or that refuse before prompting).
fn cn(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cn"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("cn runs")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("cn exits with a code")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout is not JSON ({err}); stdout={:?} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            stderr(output)
        )
    })
}

/// Runs a successful keygen into `keys`, returning its (exit-checked) output.
fn keygen(keys: &Path) -> Output {
    let keys_str = keys.to_str().expect("utf8 path");
    let output = cn_stdin(
        &["intake", "keygen", "--out", keys_str],
        &format!("{PASS}\n{PASS}\n"),
    );
    assert_eq!(code(&output), 0, "keygen stderr: {}", stderr(&output));
    output
}

// --- Test 1: round-trip keygen -> selftest -----------------------------------

#[test]
fn keygen_then_selftest_round_trip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    let keys_str = keys.to_str().expect("utf8 path");

    let kg = keygen(&keys);
    let report = stdout_json(&kg);
    assert_eq!(report["command"], "keygen");
    let fingerprint = report["fingerprint"]
        .as_str()
        .expect("fingerprint")
        .to_string();
    assert_eq!(
        fingerprint.split('-').count(),
        8,
        "8 dash groups: {fingerprint}"
    );
    assert!(keys.join("public.json").exists(), "public.json written");
    assert!(keys.join("secret.json").exists(), "secret.json written");
    // The fingerprint banner reaches stderr for hand-copying (ceremony step 8).
    assert!(
        stderr(&kg).contains(&fingerprint),
        "fingerprint banner on stderr"
    );

    let st = cn_stdin(
        &["intake", "selftest", "--key-dir", keys_str],
        &format!("{PASS}\n"),
    );
    assert_eq!(code(&st), 0, "selftest stderr: {}", stderr(&st));
    let sreport = stdout_json(&st);
    assert_eq!(sreport["mode"], "key-dir");
    assert_eq!(sreport["result"], "passed");
    assert_eq!(
        sreport["fingerprint"], fingerprint,
        "selftest certifies the same key keygen reported"
    );
}

// --- Test 2: fingerprint stability and determinism ---------------------------

#[test]
fn fingerprint_matches_keygen_and_is_deterministic() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    let kg = keygen(&keys);
    let fingerprint = stdout_json(&kg)["fingerprint"]
        .as_str()
        .expect("fp")
        .to_string();
    let public = keys.join("public.json");
    let public_str = public.to_str().expect("utf8");

    let first = cn(&["intake", "fingerprint", public_str]);
    assert_eq!(code(&first), 0, "stderr: {}", stderr(&first));
    assert_eq!(
        stdout_json(&first)["fingerprint"],
        fingerprint,
        "fingerprint command agrees with keygen"
    );

    let second = cn(&["intake", "fingerprint", public_str]);
    assert_eq!(
        first.stdout, second.stdout,
        "repeated fingerprint output is byte-identical (deterministic)"
    );
}

// --- Test 3: passphrase-encrypted key round trip (backup verify envelope) -----

#[test]
fn backup_verify_envelope_round_trip() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    let kg = keygen(&keys);
    let fingerprint = stdout_json(&kg)["fingerprint"]
        .as_str()
        .expect("fp")
        .to_string();
    let secret = keys.join("secret.json");
    let secret_str = secret.to_str().expect("utf8");

    let bv = cn_stdin(
        &["intake", "backup", "verify", secret_str],
        &format!("{PASS}\n"),
    );
    assert_eq!(code(&bv), 0, "backup verify stderr: {}", stderr(&bv));
    let report = stdout_json(&bv);
    assert_eq!(report["command"], "backup-verify");
    assert_eq!(report["mode"], "envelope");
    assert_eq!(report["result"], "passed");
    assert_eq!(
        report["fingerprint"], fingerprint,
        "backup verify's own decrypt path reports the same fingerprint"
    );
}

// --- Test 4: malformed / wrong-passphrase rejection --------------------------

#[test]
fn backup_verify_wrong_passphrase_is_content_free() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    keygen(&keys);
    let secret = keys.join("secret.json");
    let secret_str = secret.to_str().expect("utf8");

    let bv = cn_stdin(
        &["intake", "backup", "verify", secret_str],
        "the wrong passphrase entirely\n",
    );
    assert_ne!(code(&bv), 0, "wrong passphrase must fail loudly");
    let err = stderr(&bv);
    assert!(
        err.contains("crypto operation failed"),
        "content-free failure: {err}"
    );
    assert!(
        !err.contains("the wrong passphrase entirely"),
        "the failure never echoes the passphrase: {err}"
    );
    // A failed open produces no I12 success report on stdout.
    assert!(bv.stdout.is_empty(), "nothing partially succeeds to stdout");
}

#[test]
fn backup_verify_corrupt_secret_fails_loudly() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    keygen(&keys);
    // Truncate the secret envelope so it no longer parses.
    let secret = keys.join("secret.json");
    let bytes = std::fs::read(&secret).expect("read secret");
    std::fs::write(&secret, &bytes[..bytes.len() / 2]).expect("truncate secret");

    let bv = cn_stdin(
        &["intake", "backup", "verify", secret.to_str().expect("utf8")],
        &format!("{PASS}\n"),
    );
    assert_ne!(code(&bv), 0, "a corrupt secret file must fail");
    assert!(
        stderr(&bv).contains("cannot open"),
        "loud failure: {}",
        stderr(&bv)
    );
}

#[test]
fn from_print_flipped_checksum_is_rejected_before_reconstruction() {
    let dir = tempfile::tempdir().expect("tempdir");

    // Flip one hex digit of the valid check line: a mistyped sheet.
    let mut check_chars: Vec<char> = SYNTH_PRINT_CHECK.chars().collect();
    check_chars[0] = if check_chars[0] == '0' { '1' } else { '0' };
    let bad_check: String = check_chars.into_iter().collect();

    let sheet = dir.path().join("sheet.txt");
    std::fs::write(
        &sheet,
        format!("{BASE32_PREFIX}{SYNTH_PRINT_BASE32}\n{CHECK_PREFIX}{bad_check}\n"),
    )
    .expect("write sheet");

    let bv = cn(&[
        "intake",
        "backup",
        "verify",
        "--from-print",
        sheet.to_str().expect("utf8"),
    ]);
    assert_ne!(code(&bv), 0, "a flipped checksum must be rejected");
    assert!(
        stderr(&bv).contains("check mismatch"),
        "rejected on the CRC before any key reconstruction: {}",
        stderr(&bv)
    );
}

// --- Additional coverage -----------------------------------------------------

#[test]
fn keygen_is_create_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    keygen(&keys); // first run succeeds

    // Second run refuses (BEFORE prompting - null stdin proves no read).
    let again = cn(&["intake", "keygen", "--out", keys.to_str().expect("utf8")]);
    assert_ne!(code(&again), 0, "keygen refuses to overwrite");
    assert!(
        stderr(&again).contains("already exists"),
        "loud refusal: {}",
        stderr(&again)
    );

    // A directory holding only public.json is still refused, and secret.json
    // is NOT created (never a partial pair).
    let partial = dir.path().join("partial");
    std::fs::create_dir_all(&partial).expect("partial dir");
    std::fs::write(partial.join("public.json"), b"{}").expect("dummy public");
    let refused = cn(&["intake", "keygen", "--out", partial.to_str().expect("utf8")]);
    assert_ne!(code(&refused), 0);
    assert!(
        !partial.join("secret.json").exists(),
        "no partial pair written"
    );
}

#[test]
fn keygen_rejects_short_passphrase_and_writes_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    // Matching confirmation, but only three words.
    let kg = cn_stdin(
        &["intake", "keygen", "--out", keys.to_str().expect("utf8")],
        "one two three\none two three\n",
    );
    assert_ne!(code(&kg), 0, "too-short passphrase is refused");
    assert!(
        stderr(&kg).contains("6"),
        "names the 6-word minimum: {}",
        stderr(&kg)
    );
    assert!(!keys.join("public.json").exists(), "nothing written");
    assert!(!keys.join("secret.json").exists(), "nothing written");
}

#[test]
fn keygen_rejects_mismatched_confirmation_and_writes_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    let kg = cn_stdin(
        &["intake", "keygen", "--out", keys.to_str().expect("utf8")],
        "phrase alpha bravo charlie delta echo\ndifferent alpha bravo charlie delta echo\n",
    );
    assert_ne!(code(&kg), 0, "mismatched confirmation is refused");
    assert!(
        stderr(&kg).contains("did not match"),
        "loud mismatch: {}",
        stderr(&kg)
    );
    assert!(!keys.join("public.json").exists(), "nothing written");
    assert!(!keys.join("secret.json").exists(), "nothing written");
}

#[test]
fn selftest_dry_needs_no_key_dir() {
    let out = cn(&["intake", "selftest", "--dry"]);
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
    let report = stdout_json(&out);
    assert_eq!(report["mode"], "dry");
    assert_eq!(report["result"], "passed");
}

#[test]
fn selftest_requires_exactly_one_mode() {
    assert_eq!(
        code(&cn(&["intake", "selftest"])),
        2,
        "neither mode is a usage error"
    );
    assert_eq!(
        code(&cn(&["intake", "selftest", "--dry", "--key-dir", "x"])),
        2,
        "both modes is a usage error"
    );
}

#[test]
fn selftest_key_dir_rejects_mismatched_pair() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dir_a = dir.path().join("a");
    let dir_b = dir.path().join("b");
    keygen(&dir_a);
    keygen(&dir_b);
    // Swap B's public key over A's: A's secret no longer matches A's public.
    std::fs::copy(dir_b.join("public.json"), dir_a.join("public.json")).expect("swap public");

    let st = cn_stdin(
        &[
            "intake",
            "selftest",
            "--key-dir",
            dir_a.to_str().expect("utf8"),
        ],
        &format!("{PASS}\n"),
    );
    assert_ne!(code(&st), 0, "a mismatched key directory must fail");
    assert!(
        stderr(&st).contains("matching pair"),
        "the cross-check fires loudly: {}",
        stderr(&st)
    );
}

#[test]
fn backup_verify_from_print_round_trip() {
    let dir = tempfile::tempdir().expect("tempdir");

    // A valid synthetic sheet (fixed key `[1..=32]`), independent of keygen's
    // now-withheld stderr block (R1-3).
    let sheet = dir.path().join("sheet.txt");
    std::fs::write(
        &sheet,
        format!("{BASE32_PREFIX}{SYNTH_PRINT_BASE32}\n{CHECK_PREFIX}{SYNTH_PRINT_CHECK}\n"),
    )
    .expect("write sheet");

    let bv = cn(&[
        "intake",
        "backup",
        "verify",
        "--from-print",
        sheet.to_str().expect("utf8"),
    ]);
    assert_eq!(code(&bv), 0, "from-print stderr: {}", stderr(&bv));
    let report = stdout_json(&bv);
    assert_eq!(report["mode"], "print");
    assert_eq!(
        report["result"], "passed",
        "the printed backup decodes and round-trips the self-test vector"
    );
    let fingerprint = report["fingerprint"].as_str().expect("fingerprint");
    assert_eq!(
        fingerprint.split('-').count(),
        8,
        "recovered key fingerprint is 8 dash groups: {fingerprint}"
    );
}

#[test]
fn keygen_never_writes_secret_material_to_a_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let keys = dir.path().join("keys");
    let kg = keygen(&keys);
    let err_text = stderr(&kg);

    // This suite spawns `cn` with a piped (non-TTY) stderr, so keygen WITHHOLDS
    // the plaintext printed-backup block rather than leaking it to a redirected
    // stream (R1-3): the block never reaches stderr, and keygen says why.
    assert!(
        !err_text.contains(BASE32_PREFIX),
        "the plaintext backup block must be withheld on a non-TTY stderr:\n{err_text}"
    );
    assert!(
        err_text.contains("not an interactive terminal"),
        "keygen explains why the printed backup was withheld:\n{err_text}"
    );

    // The passphrase and the printed-backup markers must never appear in any
    // file the tool created (ceremony section 5; I1).
    for name in ["public.json", "secret.json"] {
        let contents = std::fs::read_to_string(keys.join(name)).expect("read key file");
        assert!(
            !contents.contains(PASS),
            "{name} must not contain the passphrase"
        );
        assert!(
            !contents.contains("print-backup"),
            "{name} must not carry the printed-backup block"
        );
    }
}

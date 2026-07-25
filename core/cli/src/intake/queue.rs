//! Queue-root filesystem layer: layout, safety guards, the single-instance
//! lock, the atomic write primitive, and the startup scan (ADR-005 D4).
//!
//! This module moves bytes; what the bytes MEAN is classified by cn-ingest.
//! Layout at the queue root (the contract the app's create-only FSA adapter
//! and the later relay puller both write into):
//!
//! ```text
//! <root>/<record_id>.record.json    immutable payload record
//! <root>/<record_id>.sidecar.json   mutable review sidecar (native-only rewrites)
//! <root>/<record_id>.reviewed       write-once review-begun marker
//! <root>/decisions/<record_id>.<uuid>.json   decision inbox (create-only)
//! <root>/decisions/consumed/        retired decision tombstones
//! <root>/corrupt/                   quarantined checksum-failing pairs
//! <root>/intake.lock                single-instance OS lock file
//! <any>.tmp-<uuid>                  torn atomic-write temps (discarded)
//! ```

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use cn_ingest::{DecisionMessage, FoundSidecar, QueueRecord, ReviewSidecar};

/// Path helper for the fixed queue layout.
pub(crate) struct QueuePaths {
    root: PathBuf,
}

impl QueuePaths {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn record(&self, record_id: &str) -> PathBuf {
        self.root.join(format!("{record_id}.record.json"))
    }

    pub fn sidecar(&self, record_id: &str) -> PathBuf {
        self.root.join(format!("{record_id}.sidecar.json"))
    }

    pub fn marker(&self, record_id: &str) -> PathBuf {
        self.root.join(format!("{record_id}.reviewed"))
    }

    pub fn decisions_dir(&self) -> PathBuf {
        self.root.join("decisions")
    }

    pub fn consumed_dir(&self) -> PathBuf {
        self.decisions_dir().join("consumed")
    }

    pub fn corrupt_dir(&self) -> PathBuf {
        self.root.join("corrupt")
    }

    pub fn lock(&self) -> PathBuf {
        self.root.join("intake.lock")
    }
}

/// Refuses a queue root inside a git worktree or a known cloud-sync path
/// (ADR-005 D4 "outside the worktree" rule; typed error, I3). Best-effort
/// by construction: a stripped marker defeats it, which is why pii-scan
/// tripwires exist as defense in depth.
pub(crate) fn refuse_unsafe_root(root: &Path) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(root)
        .map_err(|io_err| format!("queue root '{}' is not usable: {io_err}", root.display()))?;
    for ancestor in canonical.ancestors() {
        if ancestor.join(".git").exists() {
            return Err(format!(
                "queue root '{}' lies inside the git worktree at '{}'; the canonical queue \
                 root must live outside any worktree (ADR-005 D4)",
                canonical.display(),
                ancestor.display()
            ));
        }
    }
    let lowered = canonical.to_string_lossy().to_lowercase();
    for marker in ["dropbox", "onedrive", "google drive", "googledrive"] {
        if lowered.contains(marker) {
            return Err(format!(
                "queue root '{}' looks like a cloud-synced path ('{marker}'); refusing: no \
                 sync agent may cover the queue (ADR-005 D4)",
                canonical.display()
            ));
        }
    }
    Ok(canonical)
}

/// Held single-instance queue lock. The OS releases the underlying lock when
/// the process exits, so a stale lock FILE never wedges the queue and a held
/// lock proves a live process - the liveness check the ADR requires for any
/// takeover is the lock itself.
pub(crate) struct QueueLock {
    _file: File,
}

pub(crate) fn acquire_lock(paths: &QueuePaths) -> Result<QueueLock, String> {
    let file = File::options()
        .create(true)
        .truncate(false)
        .write(true)
        .open(paths.lock())
        .map_err(|io_err| format!("cannot open queue lock file: {io_err}"))?;
    match fs4::FileExt::try_lock(&file) {
        Ok(()) => Ok(QueueLock { _file: file }),
        Err(fs4::TryLockError::WouldBlock) => Err(
            "another native intake process holds the queue lock; two instances is a \
             refuse-to-run, not a race (ADR-005 D4)"
                .to_string(),
        ),
        Err(err) => Err(format!(
            "queue lock acquisition failed: {}",
            std::io::Error::from(err)
        )),
    }
}

/// Directory-handle flush capability (ADR-005 D4 degraded-platform row):
/// unsupported platforms get ONE recorded WARN per run instead of a
/// silent downgrade; supported platforms record every flush FAILURE
/// loudly (round-1 F3 - no swallowed IO).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DirSyncStatus {
    Supported,
    Unsupported,
}

pub(crate) fn dir_sync_status() -> DirSyncStatus {
    if cfg!(unix) {
        DirSyncStatus::Supported
    } else {
        DirSyncStatus::Unsupported
    }
}

#[cfg(unix)]
fn sync_dir(dir: &Path) -> Result<(), String> {
    File::open(dir)
        .and_then(|handle| handle.sync_all())
        .map_err(|io_err| format!("directory flush of '{}' failed: {io_err}", dir.display()))
}

#[cfg(not(unix))]
fn sync_dir(_dir: &Path) -> Result<(), String> {
    // Windows: no directory-handle flush; the per-run WARN comes from
    // dir_sync_status(). Replacement visibility comes from the rename;
    // power-loss durability rests on the file flush plus startup
    // recovery, exactly as ADR-005 D4 states.
    Ok(())
}

/// The one write primitive (ADR-005 D4 crash-state protocol): exclusive
/// temp in the same directory, flush + fsync, close, atomic rename onto
/// the final path, then a directory flush where the platform allows.
///
/// Platform honesty note (round-1 F3): on Windows this uses
/// `std::fs::rename`, which maps to MoveFileEx with REPLACE_EXISTING but
/// WITHOUT WRITE_THROUGH; the ADR's own qualification applies - "atomic"
/// means replacement visibility, and power-loss durability rests on the
/// file flush plus startup recovery. A flush failure on a supporting
/// platform is RECORDED in the run report, never swallowed.
pub(crate) fn write_atomic(
    final_path: &Path,
    bytes: &[u8],
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let dir = final_path
        .parent()
        .ok_or_else(|| format!("'{}' has no parent directory", final_path.display()))?;
    let file_name = final_path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("'{}' has no usable file name", final_path.display()))?;
    let temp = dir.join(format!("{file_name}.tmp-{}", uuid::Uuid::now_v7()));
    {
        let mut file = File::options()
            .create_new(true)
            .write(true)
            .open(&temp)
            .map_err(|io_err| format!("cannot create temp '{}': {io_err}", temp.display()))?;
        file.write_all(bytes)
            .and_then(|()| file.flush())
            .and_then(|()| file.sync_all())
            .map_err(|io_err| format!("cannot write temp '{}': {io_err}", temp.display()))?;
    }
    fs::rename(&temp, final_path).map_err(|io_err| {
        format!(
            "cannot rename '{}' onto '{}': {io_err}",
            temp.display(),
            final_path.display()
        )
    })?;
    if let Err(flush_warning) = sync_dir(dir) {
        warnings.push(format!(
            "{flush_warning} (recorded, I12; recovery covers the gap)"
        ));
    }
    Ok(())
}

/// Creates the write-once review-begun marker if absent (idempotent;
/// marker-before-first-decision, ADR-005 D4).
pub(crate) fn ensure_marker(
    paths: &QueuePaths,
    record_id: &str,
    warnings: &mut Vec<String>,
) -> Result<(), String> {
    let marker = paths.marker(record_id);
    if marker.exists() {
        return Ok(());
    }
    write_atomic(&marker, b"", warnings)
}

/// One record's files as found on disk (paired with cn-ingest's
/// `FoundRecord` by the apply orchestrator).
pub(crate) struct ScannedRecord {
    pub payload: Option<QueueRecord>,
    pub payload_corrupt: bool,
    pub sidecar: FoundSidecar,
    pub marker_present: bool,
}

impl ScannedRecord {
    fn empty() -> Self {
        Self {
            payload: None,
            payload_corrupt: false,
            sidecar: FoundSidecar::Missing,
            marker_present: false,
        }
    }
}

/// Everything the startup scan observed.
pub(crate) struct ScanOutcome {
    pub records: BTreeMap<String, ScannedRecord>,
    /// Non-fatal IO notes surfaced into the run report (I12; round-1 F3).
    pub io_warnings: Vec<String>,
    /// Parsed unconsumed decision files (path kept for retirement).
    pub decisions: Vec<(PathBuf, DecisionMessage)>,
    /// Decision files that failed to parse or failed version checks: loud,
    /// left in place for facilitator disposition.
    pub unreadable_decisions: Vec<(String, String)>,
    /// Consumed-directory tombstones for startup reconciliation.
    pub tombstones: Vec<(String, Result<DecisionMessage, String>)>,
    pub temp_discarded: usize,
    pub unrecognized: Vec<String>,
}

/// Scans the queue root per the ADR-005 D4 legal-on-disk-states table.
/// Temp files are discarded here (never promoted); every other
/// interpretation is left to `cn_ingest::classify`.
pub(crate) fn scan(paths: &QueuePaths) -> Result<ScanOutcome, String> {
    for dir in [
        paths.root().to_path_buf(),
        paths.decisions_dir(),
        paths.consumed_dir(),
        paths.corrupt_dir(),
    ] {
        fs::create_dir_all(&dir)
            .map_err(|io_err| format!("cannot create '{}': {io_err}", dir.display()))?;
    }

    let mut outcome = ScanOutcome {
        records: BTreeMap::new(),
        io_warnings: Vec::new(),
        decisions: Vec::new(),
        unreadable_decisions: Vec::new(),
        tombstones: Vec::new(),
        temp_discarded: 0,
        unrecognized: Vec::new(),
    };

    for entry in read_dir_files(paths.root())? {
        let name = entry_name(&entry);
        if discard_if_temp(&entry, &name, &mut outcome)? {
            continue;
        }
        if name == "intake.lock" {
            continue;
        }
        if let Some(record_id) = name.strip_suffix(".record.json") {
            let slot = outcome
                .records
                .entry(record_id.to_string())
                .or_insert_with(ScannedRecord::empty);
            match read_verified::<QueueRecord>(&entry, |record| record.verify()) {
                Ok(record) => slot.payload = Some(record),
                Err(_) => slot.payload_corrupt = true,
            }
        } else if let Some(record_id) = name.strip_suffix(".sidecar.json") {
            let slot = outcome
                .records
                .entry(record_id.to_string())
                .or_insert_with(ScannedRecord::empty);
            slot.sidecar = match read_verified::<ReviewSidecar>(&entry, |sidecar| sidecar.verify())
            {
                Ok(sidecar) => FoundSidecar::Valid(Box::new(sidecar)),
                Err(_) => FoundSidecar::Corrupt,
            };
        } else if let Some(record_id) = name.strip_suffix(".reviewed") {
            outcome
                .records
                .entry(record_id.to_string())
                .or_insert_with(ScannedRecord::empty)
                .marker_present = true;
        } else {
            outcome.unrecognized.push(name);
        }
    }

    for entry in read_dir_files(&paths.decisions_dir())? {
        let name = entry_name(&entry);
        if discard_if_temp(&entry, &name, &mut outcome)? {
            continue;
        }
        match read_verified::<DecisionMessage>(&entry, |message| message.verify()) {
            Ok(message) => outcome.decisions.push((entry, message)),
            Err(reason) => outcome.unreadable_decisions.push((name, reason)),
        }
    }

    for entry in read_dir_files(&paths.consumed_dir())? {
        let name = entry_name(&entry);
        outcome.tombstones.push((
            name,
            read_verified::<DecisionMessage>(&entry, |message| message.verify()),
        ));
    }

    Ok(outcome)
}

fn read_dir_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut files = Vec::new();
    let entries = fs::read_dir(dir)
        .map_err(|io_err| format!("cannot read directory '{}': {io_err}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|io_err| format!("cannot list '{}': {io_err}", dir.display()))?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn entry_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn discard_if_temp(path: &Path, name: &str, outcome: &mut ScanOutcome) -> Result<bool, String> {
    if !name.contains(".tmp-") {
        return Ok(false);
    }
    fs::remove_file(path)
        .map_err(|io_err| format!("cannot discard temp '{}': {io_err}", path.display()))?;
    outcome.temp_discarded += 1;
    Ok(true)
}

fn read_verified<T>(
    path: &Path,
    verify: impl Fn(&T) -> Result<(), cn_ingest::IngestError>,
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let bytes =
        fs::read(path).map_err(|io_err| format!("cannot read '{}': {io_err}", path.display()))?;
    let value: T = serde_json::from_slice(&bytes)
        .map_err(|parse_err| format!("cannot parse '{}': {parse_err}", path.display()))?;
    verify(&value).map_err(|verify_err| format!("'{}': {verify_err}", path.display()))?;
    Ok(value)
}

/// Retires a decision file into `decisions/consumed/` (atomic rename;
/// retire-after-durable is the caller's ordering obligation).
pub(crate) fn retire_decision(paths: &QueuePaths, file: &Path) -> Result<(), String> {
    let name = file
        .file_name()
        .ok_or_else(|| format!("decision file '{}' has no name", file.display()))?;
    let target = paths.consumed_dir().join(name);
    fs::rename(file, &target).map_err(|io_err| {
        format!(
            "cannot retire '{}' into '{}': {io_err}",
            file.display(),
            target.display()
        )
    })
}

/// Moves a record's payload and sidecar into `corrupt/` quarantine
/// (retained, never trusted; ADR-005 D4).
pub(crate) fn quarantine_pair(paths: &QueuePaths, record_id: &str) -> Result<(), String> {
    for source in [paths.record(record_id), paths.sidecar(record_id)] {
        if !source.exists() {
            continue;
        }
        let name = source
            .file_name()
            .ok_or_else(|| format!("'{}' has no name", source.display()))?;
        let target = paths.corrupt_dir().join(name);
        fs::rename(&source, &target).map_err(|io_err| {
            format!(
                "cannot quarantine '{}' into '{}': {io_err}",
                source.display(),
                target.display()
            )
        })?;
    }
    Ok(())
}

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crc32fast::Hasher;
use serde::{Deserialize, Serialize};

use crate::fold::{GroupState, StoreFinding, StoreReport};
use crate::op::{Operation, SortKey};

const SNAPSHOT_SCHEMA: &str = "0.1.0";

/// Native append-only operation log.
pub struct OpLog {
    path: PathBuf,
    file: File,
}

impl OpLog {
    /// Opens a log, replaying complete lines and recovering a torn final line.
    pub fn open(path: &Path) -> Result<(Self, Vec<Operation>, StoreReport), StoreError> {
        if !path.exists() {
            File::create(path).map_err(|source| StoreError::io(path, source))?;
        }
        let bytes = std::fs::read(path).map_err(|source| StoreError::io(path, source))?;
        let (ops, report, truncate_to) = parse_log_bytes(path, &bytes)?;
        if let Some(len) = truncate_to {
            OpenOptions::new()
                .write(true)
                .open(path)
                .map_err(|source| StoreError::io(path, source))?
                .set_len(len)
                .map_err(|source| StoreError::io(path, source))?;
        }
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .map_err(|source| StoreError::io(path, source))?;
        Ok((
            Self {
                path: path.to_path_buf(),
                file,
            },
            ops,
            report,
        ))
    }

    /// Appends all operations, then flushes and syncs once for batch durability.
    pub fn append_batch(&mut self, ops: &[Operation]) -> Result<(), StoreError> {
        for op in ops {
            serde_json::to_writer(&mut self.file, op)
                .map_err(|err| StoreError::Serialize(err.to_string()))?;
            self.file
                .write_all(b"\n")
                .map_err(|source| StoreError::io(&self.path, source))?;
        }
        self.file
            .flush()
            .map_err(|source| StoreError::io(&self.path, source))?;
        self.file
            .sync_all()
            .map_err(|source| StoreError::io(&self.path, source))
    }
}

/// Snapshot persistence helper.
pub struct Snapshot;

impl Snapshot {
    /// Saves a checksummed snapshot.
    pub fn save(path: &Path, state: &GroupState) -> Result<(), StoreError> {
        let payload = SnapshotPayload {
            state: SnapshotState::from(state),
            meta: SnapshotMeta::from_state(state),
        };
        let data =
            serde_json::to_vec(&payload).map_err(|err| StoreError::Serialize(err.to_string()))?;
        let envelope = SnapshotEnvelope {
            schema_version: semver::Version::parse(SNAPSHOT_SCHEMA)
                .map_err(|err| StoreError::Serialize(err.to_string()))?,
            crc32: checksum(&data),
            data,
        };
        let bytes =
            serde_json::to_vec(&envelope).map_err(|err| StoreError::Serialize(err.to_string()))?;
        std::fs::write(path, bytes).map_err(|source| StoreError::io(path, source))
    }

    /// Loads a snapshot, returning None when parsing or checksum validation fails.
    pub fn load(path: &Path) -> Result<Option<(GroupState, SnapshotMeta)>, StoreError> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path).map_err(|source| StoreError::io(path, source))?;
        let Ok(envelope) = serde_json::from_slice::<SnapshotEnvelope>(&bytes) else {
            return Ok(None);
        };
        if envelope.schema_version.major != 0 || envelope.schema_version.minor != 1 {
            return Err(StoreError::UnsupportedSchemaVersion);
        }
        if checksum(&envelope.data) != envelope.crc32 {
            return Ok(None);
        }
        let Ok(payload) = serde_json::from_slice::<SnapshotPayload>(&envelope.data) else {
            return Ok(None);
        };
        Ok(Some((payload.state.into_group_state(), payload.meta)))
    }

    /// Loads a snapshot only when its watermark is not ahead of the supplied log tip.
    pub fn load_with_log_tip(
        path: &Path,
        log_tip: Option<&SortKey>,
    ) -> Result<Option<(GroupState, SnapshotMeta)>, StoreError> {
        let Some((state, meta)) = Self::load(path)? else {
            return Ok(None);
        };
        if watermark_ahead(meta.last_sort_key.as_ref(), log_tip) {
            return Ok(None);
        }
        Ok(Some((state, meta)))
    }
}

/// Snapshot watermark metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub last_op_id: Option<cn_model::OpId>,
    pub last_sort_key: Option<SortKey>,
}

impl SnapshotMeta {
    fn from_state(state: &GroupState) -> Self {
        match state.snapshot_watermark() {
            Some((last_op_id, last_sort_key)) => Self {
                last_op_id: Some(last_op_id),
                last_sort_key: Some(last_sort_key),
            },
            None => Self {
                last_op_id: None,
                last_sort_key: None,
            },
        }
    }
}

/// Store persistence errors.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("corrupt log at line {line}")]
    CorruptLog { line: usize },
    #[error("serialization error: {0}")]
    Serialize(String),
    #[error("unsupported schema version")]
    UnsupportedSchemaVersion,
}

impl StoreError {
    fn io(path: &Path, source: std::io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            source,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SnapshotEnvelope {
    schema_version: semver::Version,
    crc32: u32,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct SnapshotPayload {
    state: SnapshotState,
    meta: SnapshotMeta,
}

#[derive(Serialize, Deserialize)]
struct SnapshotState {
    group: Option<cn_model::Group>,
    template: Option<cn_schema::GroupTemplate>,
    entities: Vec<cn_model::Entity>,
    edges: Vec<cn_model::Edge>,
    memberships: Vec<cn_model::Membership>,
    trust_grants: Vec<cn_model::TrustGrant>,
    stories: Vec<cn_model::Story>,
}

impl From<&GroupState> for SnapshotState {
    fn from(state: &GroupState) -> Self {
        Self {
            group: state.group.clone(),
            template: state.template.clone(),
            entities: state.entities.values().cloned().collect(),
            edges: state.edges.values().cloned().collect(),
            memberships: state.memberships.values().cloned().collect(),
            trust_grants: state.trust_grants.values().cloned().collect(),
            stories: state.stories.values().cloned().collect(),
        }
    }
}

impl SnapshotState {
    fn into_group_state(self) -> GroupState {
        GroupState::from_snapshot_parts(
            self.group,
            self.template,
            self.entities
                .into_iter()
                .map(|item| (item.id, item))
                .collect(),
            self.edges.into_iter().map(|item| (item.id, item)).collect(),
            self.memberships
                .into_iter()
                .map(|item| (item.id, item))
                .collect(),
            self.trust_grants
                .into_iter()
                .map(|item| (item.id, item))
                .collect(),
            self.stories
                .into_iter()
                .map(|item| (item.id, item))
                .collect(),
        )
    }
}

fn parse_log_bytes(
    path: &Path,
    bytes: &[u8],
) -> Result<(Vec<Operation>, StoreReport, Option<u64>), StoreError> {
    let mut ops = Vec::new();
    let mut report = StoreReport::default();
    let final_start = bytes.iter().rposition(|byte| *byte == b'\n').map(|i| i + 1);
    let complete = final_start.map_or(bytes, |end| &bytes[..end]);
    for (index, line) in complete.split_inclusive(|byte| *byte == b'\n').enumerate() {
        let trimmed = line.strip_suffix(b"\n").unwrap_or(line);
        if trimmed.is_empty() {
            continue;
        }
        parse_line(trimmed, index + 1, &mut ops)?;
    }
    if bytes.last().is_some_and(|byte| *byte != b'\n') {
        let start = final_start.unwrap_or(0);
        let line_no = bytes[..start].iter().filter(|byte| **byte == b'\n').count() + 1;
        let final_line = &bytes[start..];
        if serde_json::from_slice::<Operation>(final_line).is_err() {
            report.warnings.push(StoreFinding {
                code: "TornLineRecovered".to_string(),
                message: format!("recovered torn final line in {}", path.display()),
            });
            return Ok((ops, report, Some(start as u64)));
        }
        ops.push(
            serde_json::from_slice(final_line)
                .map_err(|_| StoreError::CorruptLog { line: line_no })?,
        );
    }
    Ok((ops, report, None))
}

fn parse_line(line: &[u8], line_no: usize, ops: &mut Vec<Operation>) -> Result<(), StoreError> {
    match serde_json::from_slice(line) {
        Ok(op) => {
            ops.push(op);
            Ok(())
        }
        Err(_) => Err(StoreError::CorruptLog { line: line_no }),
    }
}

fn checksum(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}

fn watermark_ahead(watermark: Option<&SortKey>, log_tip: Option<&SortKey>) -> bool {
    match (watermark, log_tip) {
        (Some(_), None) => true,
        (Some(watermark), Some(log_tip)) => watermark > log_tip,
        (None, _) => false,
    }
}

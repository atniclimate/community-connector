//! Version discipline and canonical digests for the queue formats (I7).

use sha2::{Digest, Sha256};

/// Current queue record/sidecar/decision format version (ADR-005 D4).
pub const QUEUE_RECORD_VERSION: &str = "0.1.0";

/// Typed ingest failure (I3: loud, never silent).
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum IngestError {
    #[error("unknown queue format major version {found}")]
    UnknownMajorVersion { found: semver::Version },
    #[error("checksum mismatch for {what}")]
    ChecksumMismatch { what: String },
    #[error("sidecar binding mismatch for record {record_id}")]
    BindingMismatch { record_id: String },
    #[error("serialization error: {0}")]
    Serialize(String),
}

/// SHA-256 (lowercase hex) over a value's canonical serde_json bytes.
/// serde_json's default map is ordered (BTreeMap), so object keys are
/// sorted and the serialization is canonical for our purposes.
pub fn canonical_digest<T: serde::Serialize>(value: &T) -> Result<String, IngestError> {
    let bytes = serde_json::to_vec(value).map_err(|err| IngestError::Serialize(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Rejects unknown MAJOR versions loudly; unknown minor is tolerated
/// (ignore-and-preserve is handled by each format's extras map).
pub fn check_major(version: &semver::Version) -> Result<(), IngestError> {
    let current = semver::Version::parse(QUEUE_RECORD_VERSION).expect("valid const");
    if version.major != current.major {
        return Err(IngestError::UnknownMajorVersion {
            found: version.clone(),
        });
    }
    Ok(())
}

/// Generates a UUIDv7 string (record ids, decision ids, preassigned op ids).
pub fn new_uuid_v7() -> String {
    uuid::Uuid::now_v7().to_string()
}

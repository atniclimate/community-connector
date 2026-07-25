//! Transport and semantic dedup classification (ADR-005 D4, round-1
//! amendment). `submission_id` is client-controlled and therefore cannot
//! be the only key: the transport layer (re-pull safety) is checked first,
//! then the semantic layer. Conflicts are DISTINCT typed outcomes the UI
//! must surface for facilitator disposition - never a drop.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::record::{QueueRecord, SubmissionSource};

/// The two dedup keys of one staged record, extracted for comparison.
/// Transport fields exist only for remote records (source discriminant).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DedupKey {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receipt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ciphertext_hash: Option<String>,
    /// Client-asserted submission id (may be empty; empty never matches).
    pub submission_id: String,
    /// Canonical digest of the inner payload.
    pub payload_hash: String,
}

impl DedupKey {
    /// Extracts both keys from a staged record.
    pub fn from_record(record: &QueueRecord) -> Self {
        let (receipt_id, ciphertext_hash) = match &record.source {
            SubmissionSource::Remote {
                receipt_id,
                ciphertext_hash,
                ..
            } => (Some(receipt_id.clone()), Some(ciphertext_hash.clone())),
            SubmissionSource::InApp {} => (None, None),
        };
        Self {
            receipt_id,
            ciphertext_hash,
            submission_id: record
                .payload
                .get("submission_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            payload_hash: record.payload_hash.clone(),
        }
    }
}

/// Typed dedup outcome (ADR-005 D4). Replays are recorded no-ops (I12);
/// conflicts demand facilitator attention and are never silently dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DedupVerdict {
    Fresh,
    /// Same `(receipt_id, ciphertext_hash)`: a re-pulled receipt; recorded
    /// no-op.
    TransportReplay,
    /// Same `receipt_id` with a DIFFERENT ciphertext hash: relay-side
    /// substitution or a platform fault - loud transport-integrity
    /// conflict (I3), never a no-op; the relay copy is retained.
    TransportConflict,
    /// Same `(submission_id, payload_hash)`: the same submission staged
    /// twice; recorded no-op.
    SemanticReplay,
    /// Same `submission_id` with a DIFFERENT payload hash: surfaced for
    /// facilitator disposition, never a drop (ADR-005 D4).
    Conflict,
}

/// Classifies a new record's keys against the existing records' keys.
/// Transport first, then semantic; an empty `submission_id` never matches
/// (client-controlled - two blank ids are not the same submission).
pub fn classify_dedup(new: &DedupKey, existing: &[DedupKey]) -> DedupVerdict {
    for key in existing {
        if let (Some(receipt), Some(existing_receipt)) = (&new.receipt_id, &key.receipt_id)
            && receipt == existing_receipt
        {
            return if new.ciphertext_hash == key.ciphertext_hash {
                DedupVerdict::TransportReplay
            } else {
                DedupVerdict::TransportConflict
            };
        }
    }
    for key in existing {
        if !new.submission_id.is_empty() && new.submission_id == key.submission_id {
            return if new.payload_hash == key.payload_hash {
                DedupVerdict::SemanticReplay
            } else {
                DedupVerdict::Conflict
            };
        }
    }
    DedupVerdict::Fresh
}

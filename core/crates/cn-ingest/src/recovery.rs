//! Startup recovery classification: the ADR-005 D4 legal-on-disk-states
//! table as pure logic. The caller (native `cn intake apply` - the app
//! never mutates, so it never recovers) executes actions through its own
//! I/O; the invariants live here: never promote a temp, never reconstruct
//! a decision history, marker-before-first-decision, marker
//! create-if-absent idempotent.

use crate::record::{QueueRecord, ReviewSidecar, ReviewState};
use crate::version::IngestError;

/// What the recovery scan observed for one record id (the caller reads the
/// filesystem; parse failures arrive as `Corrupt`).
#[derive(Debug, Clone, PartialEq)]
pub enum FoundSidecar {
    Missing,
    /// Parsed and checksum-verified.
    Valid(Box<ReviewSidecar>),
    /// Present but unreadable, checksum-failing, or binding-mismatched.
    Corrupt,
}

/// One record's observed on-disk state.
#[derive(Debug, Clone, PartialEq)]
pub struct FoundRecord {
    pub record_id: String,
    /// None when the payload file is missing or checksum-failing.
    pub payload: Option<QueueRecord>,
    pub payload_corrupt: bool,
    pub sidecar: FoundSidecar,
    /// The write-once review-begun marker file.
    pub marker_present: bool,
    /// Unconsumed decision files targeting this record.
    pub pending_decisions: usize,
}

/// Deterministic recovery action per the ADR-005 D4 table.
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction {
    /// Reconstruct a `pending` sidecar (initial state ONLY - a decision
    /// history is never reconstructed).
    ReconstructPendingSidecar { record_id: String },
    /// Loud halt: marker present without a readable sidecar means lost
    /// decision state; nothing is guessed.
    HaltLostDecisionState { record_id: String },
    /// Move the pair to corrupt/ quarantine; retain any relay receipt
    /// (local durability unproven - the upstream copy is preserved).
    QuarantineCorrupt { record_id: String },
    /// Loud halt: sidecar binding mismatch; both files retained for
    /// facilitator investigation, no automatic repair.
    HaltBindingMismatch { record_id: String },
    /// Run the approval recovery rule before any other queue work.
    RunApprovalRecovery { record_id: String },
    /// Benign transient: marker present with unconsumed decisions; normal
    /// admission proceeds.
    ProceedWithAdmission { record_id: String },
    /// Marker-without-history anomaly: stays pending, reported (I12); the
    /// facilitator re-decides if a decision was intended.
    ReportMarkerAnomaly { record_id: String },
    /// Nothing to do for this record.
    None { record_id: String },
}

/// Discards are separate from per-record actions: a temp file is NEVER
/// promoted; its content, if it mattered, is re-creatable.
pub fn classify_temp_file() -> &'static str {
    "discard"
}

/// Classifies one record's on-disk state into its deterministic action.
pub fn classify(found: &FoundRecord) -> Result<RecoveryAction, IngestError> {
    let record_id = found.record_id.clone();

    // Corrupt payload or corrupt sidecar: quarantine the pair.
    if found.payload_corrupt || found.sidecar == FoundSidecar::Corrupt {
        return Ok(RecoveryAction::QuarantineCorrupt { record_id });
    }

    let Some(payload) = &found.payload else {
        // Sidecar without payload is a binding-level investigation case.
        return Ok(match found.sidecar {
            FoundSidecar::Missing => RecoveryAction::None { record_id },
            _ => RecoveryAction::HaltBindingMismatch { record_id },
        });
    };

    match &found.sidecar {
        FoundSidecar::Missing => {
            if found.marker_present {
                // Lost decision state: history is never recreated.
                Ok(RecoveryAction::HaltLostDecisionState { record_id })
            } else {
                Ok(RecoveryAction::ReconstructPendingSidecar { record_id })
            }
        }
        FoundSidecar::Valid(sidecar) => {
            if sidecar.verify_pair(payload).is_err() {
                return Ok(RecoveryAction::HaltBindingMismatch { record_id });
            }
            match sidecar.review_state {
                ReviewState::ApprovedIntent => {
                    Ok(RecoveryAction::RunApprovalRecovery { record_id })
                }
                ReviewState::Pending if found.marker_present => {
                    let has_decision_history = sidecar
                        .history
                        .iter()
                        .any(|entry| matches!(entry, crate::record::HistoryEntry::Decision { .. }));
                    if found.pending_decisions > 0 {
                        Ok(RecoveryAction::ProceedWithAdmission { record_id })
                    } else if has_decision_history {
                        // Marker matches recorded history: normal state.
                        Ok(RecoveryAction::None { record_id })
                    } else {
                        Ok(RecoveryAction::ReportMarkerAnomaly { record_id })
                    }
                }
                _ => Ok(RecoveryAction::None { record_id }),
            }
        }
        FoundSidecar::Corrupt => unreachable!("handled above"),
    }
}

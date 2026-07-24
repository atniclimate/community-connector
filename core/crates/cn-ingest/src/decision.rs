//! Decision-inbox messages and the admission table (ADR-005 D4, rounds 4-7).
//!
//! The browser creates decision files (create-only, unique names); native
//! apply admits them through this pure table: binding checks, decision-id
//! dedup by message digest, decision-generation+state CAS, and legal
//! transitions. Same-digest replays are WRITELESS (the original durable
//! entry is the retirement proof); stale/illegal decisions get durable
//! audit entries that never advance the generation.

use serde::{Deserialize, Serialize};

use cn_model::Timestamp;

use crate::record::{DecisionOutcome, HistoryEntry, ReviewSidecar, ReviewState};
use crate::version::{IngestError, QUEUE_RECORD_VERSION, canonical_digest, check_major};

/// Decision type (ADR-005 D4; every type's exact effect is defined).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    /// pending -> approved_intent (with the plan, in the same write).
    Approve,
    /// pending -> rejected.
    Reject,
    /// State REMAINS pending; the generation advance invalidates older
    /// views.
    SetAsideNote { note: String },
    /// failed -> pending: the only exit from the terminal state.
    ClearFailed,
}

/// Decision-inbox message (versioned per I7). The body carries identity;
/// the filename never does.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DecisionMessage {
    /// Queue format version (unknown MAJOR rejected loudly).
    pub queue_record_version: semver::Version,
    /// Body-carried decision id (UUIDv7).
    pub decision_id: String,
    /// Target payload record.
    pub record_id: String,
    /// Binding: the payload record's record_checksum value.
    pub payload_digest: String,
    /// CAS premise: the state the wizard observed.
    pub expected_review_state: ReviewState,
    /// CAS premise: the semantic generation the wizard observed.
    pub expected_decision_generation: u64,
    /// The decision.
    pub decision: DecisionType,
    /// Reviewer identity.
    pub reviewer: String,
    /// Decision time (wizard clock).
    pub decided_at: Timestamp,
}

impl DecisionMessage {
    /// Canonical digest of the whole message (dedup evidence).
    pub fn message_digest(&self) -> Result<String, IngestError> {
        canonical_digest(self)
    }

    /// Verifies version major.
    pub fn verify(&self) -> Result<(), IngestError> {
        check_major(&self.queue_record_version)
    }
}

/// Admission verdict for one decision message against one sidecar.
#[derive(Debug, Clone, PartialEq)]
pub enum AdmissionVerdict {
    /// Admit: apply `effect` to the sidecar in ONE atomic rewrite that also
    /// appends the decision event (and, for Approve, the plan + intent).
    Admit {
        entry: HistoryEntry,
        resulting_state: ReviewState,
        resulting_generation: u64,
    },
    /// Same id + same digest already durable: retire against the original
    /// entry, WRITE NOTHING (round 6: retry bookkeeping moves no counter).
    WritelessReplay,
    /// Same id + DIFFERENT message digest: loud conflict (I3), never a
    /// replay.
    IdReuseConflict,
    /// CAS mismatch: append a durable `stale` decision event (generation
    /// unchanged), retire unapplied.
    Stale { entry: HistoryEntry },
    /// Illegal transition: append a durable `illegal` decision event
    /// (generation unchanged), retire unapplied.
    Illegal { entry: HistoryEntry },
    /// Binding failure between message and sidecar/payload.
    BindingMismatch,
}

/// The admission table (ADR-005 D4). Pure: the caller executes the verdict
/// through its own I/O inside the native critical section.
pub fn admit(
    sidecar: &ReviewSidecar,
    message: &DecisionMessage,
) -> Result<AdmissionVerdict, IngestError> {
    message.verify()?;
    sidecar.verify()?;
    if message.record_id != sidecar.record_id || message.payload_digest != sidecar.payload_digest {
        return Ok(AdmissionVerdict::BindingMismatch);
    }
    let digest = message.message_digest()?;

    // Dedup against DECISION events only.
    for entry in &sidecar.history {
        if let HistoryEntry::Decision {
            decision_id,
            message_digest,
            ..
        } = entry
            && *decision_id == message.decision_id
        {
            return Ok(if *message_digest == digest {
                AdmissionVerdict::WritelessReplay
            } else {
                AdmissionVerdict::IdReuseConflict
            });
        }
    }

    let make_entry = |resulting_state: ReviewState,
                      resulting_generation: u64,
                      outcome: DecisionOutcome| HistoryEntry::Decision {
        decision_id: message.decision_id.clone(),
        message_digest: digest.clone(),
        decision_type: message.decision.clone(),
        prior_state: sidecar.review_state,
        prior_generation: sidecar.decision_generation,
        resulting_state,
        resulting_generation,
        reviewer: message.reviewer.clone(),
        decided_at: message.decided_at,
        reason: match &message.decision {
            DecisionType::SetAsideNote { note } => Some(note.clone()),
            _ => None,
        },
        outcome,
    };

    // CAS: generation AND state (the generation is what closes the ABA
    // cycle; audit entries never advance it, so recording staleness cannot
    // invalidate a decision authored against the current generation).
    if message.expected_decision_generation != sidecar.decision_generation
        || message.expected_review_state != sidecar.review_state
    {
        return Ok(AdmissionVerdict::Stale {
            entry: make_entry(
                sidecar.review_state,
                sidecar.decision_generation,
                DecisionOutcome::Stale,
            ),
        });
    }

    // Legal transitions only.
    let next_state = match (&message.decision, sidecar.review_state) {
        (DecisionType::Approve, ReviewState::Pending) => ReviewState::ApprovedIntent,
        (DecisionType::Reject, ReviewState::Pending) => ReviewState::Rejected,
        (DecisionType::SetAsideNote { .. }, ReviewState::Pending) => ReviewState::Pending,
        (DecisionType::ClearFailed, ReviewState::Failed) => ReviewState::Pending,
        _ => {
            return Ok(AdmissionVerdict::Illegal {
                entry: make_entry(
                    sidecar.review_state,
                    sidecar.decision_generation,
                    DecisionOutcome::Illegal,
                ),
            });
        }
    };

    // Every ADMITTED decision advances the generation - note-only included.
    let next_generation = sidecar.decision_generation + 1;
    Ok(AdmissionVerdict::Admit {
        entry: make_entry(next_state, next_generation, DecisionOutcome::Admitted),
        resulting_state: next_state,
        resulting_generation: next_generation,
    })
}

/// Deterministic inbox processing order: (decided_at, decision_id).
pub fn processing_order(messages: &mut [DecisionMessage]) {
    messages.sort_by(|a, b| {
        (a.decided_at, a.decision_id.as_str()).cmp(&(b.decided_at, b.decision_id.as_str()))
    });
}

/// Applies an admission verdict's sidecar effect (Admit/Stale/Illegal) and
/// re-seals. WritelessReplay and conflicts write nothing by contract.
pub fn apply_verdict(
    sidecar: &mut ReviewSidecar,
    verdict: &AdmissionVerdict,
) -> Result<(), IngestError> {
    match verdict {
        AdmissionVerdict::Admit {
            entry,
            resulting_state,
            resulting_generation,
        } => {
            sidecar.history.push(entry.clone());
            sidecar.review_state = *resulting_state;
            sidecar.decision_generation = *resulting_generation;
            sidecar.seal_rewrite()
        }
        AdmissionVerdict::Stale { entry } | AdmissionVerdict::Illegal { entry } => {
            sidecar.history.push(entry.clone());
            sidecar.seal_rewrite()
        }
        AdmissionVerdict::WritelessReplay
        | AdmissionVerdict::IdReuseConflict
        | AdmissionVerdict::BindingMismatch => Ok(()),
    }
}

/// Records an approval-transaction event (never a dedup target); each is a
/// state change and advances the decision generation (ADR-005 D4).
pub fn record_transaction(
    sidecar: &mut ReviewSidecar,
    linked_decision_id: &str,
    kind: crate::record::TransactionKind,
    resulting_state: ReviewState,
    at: Timestamp,
    detail: Option<String>,
) -> Result<(), IngestError> {
    sidecar.history.push(HistoryEntry::Transaction {
        linked_decision_id: linked_decision_id.to_string(),
        kind,
        prior_state: sidecar.review_state,
        resulting_state,
        at,
        detail,
    });
    sidecar.review_state = resulting_state;
    sidecar.decision_generation += 1;
    sidecar.seal_rewrite()
}

/// Current decision-message version helper for builders.
pub fn decision_message_version() -> semver::Version {
    semver::Version::parse(QUEUE_RECORD_VERSION).expect("valid const")
}

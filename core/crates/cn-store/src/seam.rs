//! Idempotent durable batch append for intake approval (ADR-005 D4).
//!
//! Classifies preassigned op ids against the DURABLE LOG (not the fold
//! seen-set), preflights authorization and fold acceptance on a shadow
//! clone in batch order, appends only absent ops with one fsync, then
//! folds - all inside the caller's native critical section. Recovery
//! under a durable approval intent completes without re-authorization:
//! the intent marker, not post-crash authority, governs (ADR-005 D4).

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use cn_model::OpId;

use crate::authz::{Authorizer, AuthzDenial};
use crate::fold::{GroupState, StoreReport};
use crate::log::{OpLog, StoreError};
use crate::op::Operation;

/// Canonical digest of an operation: SHA-256 (lowercase hex) over its
/// sorted-key canonical serialization (via a `serde_json::Value` tree,
/// whose objects are BTreeMaps - matching cn-ingest's `canonical_digest`
/// byte domain exactly; ADR-005 D4 sorted-keys rule). Both the approval
/// plan and durable-log classification compute digests over the
/// in-memory `Operation`, so the domain is identical on both sides.
pub fn canonical_op_digest(op: &Operation) -> Result<String, StoreError> {
    let tree = serde_json::to_value(op).map_err(|err| StoreError::Serialize(err.to_string()))?;
    let bytes = serde_json::to_vec(&tree).map_err(|err| StoreError::Serialize(err.to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

/// Durable-log membership index: op id -> canonical digest. Built from the
/// ops replayed at `OpLog::open` and maintained by the seam on append; this
/// is the durable-presence authority, distinct from the fold seen-set.
#[derive(Debug, Default)]
pub struct DurableOpIndex {
    map: BTreeMap<OpId, String>,
}

impl DurableOpIndex {
    /// Indexes the full durable log contents.
    pub fn from_ops(ops: &[Operation]) -> Result<Self, StoreError> {
        let mut map = BTreeMap::new();
        for op in ops {
            map.insert(op.op_id, canonical_op_digest(op)?);
        }
        Ok(Self { map })
    }

    /// Number of durable ops indexed.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// True when the durable log holds no ops.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// True when the given op id is durable (any digest). Recovery mode
    /// selection depends on this: ALL-absent plans re-run the FULL
    /// first-attempt step (preflight included) per ADR-005 D4; only a
    /// batch with durable presence completes under the intent marker.
    pub fn contains(&self, op_id: &OpId) -> bool {
        self.map.contains_key(op_id)
    }
}

/// One planned batch entry: the op plus its preassigned canonical digest
/// from the approval plan.
#[derive(Debug, Clone)]
pub struct BatchEntry {
    pub op: Operation,
    pub digest: String,
}

/// First attempts preflight authorization and fold acceptance.
/// `RecoveryUnderIntent` completes a PARTIALLY APPENDED batch without
/// re-authorization (ADR-005 D4: the durable intent marker, not
/// post-crash authority, governs completion of what first-attempt
/// preflight already authorized). An ALL-ABSENT plan at recovery must
/// NOT use it - the ADR's rule for that pattern is "re-run step 2",
/// which includes preflight; callers select the mode via
/// `DurableOpIndex::contains`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchMode {
    FirstAttempt,
    RecoveryUnderIntent,
}

/// Per-op durable outcome of a successful batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpDisposition {
    AbsentAppended,
    PresentSameDigest,
}

/// Typed batch failure. NOTHING was appended for any of these.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum BatchFailure {
    /// The same op id is durable with different bytes (ADR-005
    /// `durable_conflict`).
    #[error("op {op_id} is durable with conflicting bytes")]
    DigestConflict { op_id: OpId },
    /// Durable presence is not a contiguous batch-order prefix (ADR-005
    /// `durable_inconsistency`: a hole or out-of-order presence).
    #[error("durable presence at {op_id} is not a contiguous prefix")]
    DurableInconsistency { op_id: OpId },
    /// The supplied digest does not match the supplied op's bytes - the
    /// plan itself is corrupt.
    #[error("plan digest mismatch for op {op_id}")]
    PlanDigestMismatch { op_id: OpId },
    /// Preflight authorization denial (first attempt only; ADR-005
    /// `preflight_failed`).
    #[error("op {op_id} denied in preflight")]
    Denied { op_id: OpId, denial: AuthzDenial },
    /// Preflight fold acceptance failure (first attempt only; ADR-005
    /// `preflight_failed`).
    #[error("op {op_id} would quarantine in preflight")]
    WouldQuarantine { op_id: OpId },
    /// Underlying store failure.
    #[error("store failure: {0}")]
    Store(String),
}

/// Idempotent durable batch append (ADR-005 D4).
///
/// On success every batch op is durable exactly once and folded into
/// `state`; the returned dispositions say which ops this call appended.
/// On failure nothing was appended and `state` is untouched.
pub fn append_batch_idempotent<A: Authorizer>(
    log: &mut OpLog,
    index: &mut DurableOpIndex,
    state: &mut GroupState,
    authorizer: &A,
    batch: &[BatchEntry],
    mode: BatchMode,
    report: &mut StoreReport,
) -> Result<Vec<(OpId, OpDisposition)>, BatchFailure> {
    // Plan honesty + durable classification + contiguous-prefix check.
    let mut dispositions = Vec::with_capacity(batch.len());
    let mut seen_absent = false;
    for entry in batch {
        let computed =
            canonical_op_digest(&entry.op).map_err(|err| BatchFailure::Store(err.to_string()))?;
        if computed != entry.digest {
            return Err(BatchFailure::PlanDigestMismatch {
                op_id: entry.op.op_id,
            });
        }
        match index.map.get(&entry.op.op_id) {
            Some(existing) if *existing == entry.digest => {
                if seen_absent {
                    return Err(BatchFailure::DurableInconsistency {
                        op_id: entry.op.op_id,
                    });
                }
                dispositions.push((entry.op.op_id, OpDisposition::PresentSameDigest));
            }
            Some(_) => {
                return Err(BatchFailure::DigestConflict {
                    op_id: entry.op.op_id,
                });
            }
            None => {
                seen_absent = true;
                dispositions.push((entry.op.op_id, OpDisposition::AbsentAppended));
            }
        }
    }
    let absent: Vec<&BatchEntry> = batch
        .iter()
        .zip(dispositions.iter())
        .filter(|(_, (_, disposition))| *disposition == OpDisposition::AbsentAppended)
        .map(|(entry, _)| entry)
        .collect();

    // Shadow preflight in batch order (first attempt only): the clone
    // already holds every durable op's effect, so an absent op depending
    // on the present prefix authorizes correctly.
    if mode == BatchMode::FirstAttempt {
        let mut shadow = state.clone();
        let mut scratch = StoreReport::default();
        for entry in &absent {
            authorizer
                .authorize(&shadow, &entry.op)
                .map_err(|denial| BatchFailure::Denied {
                    op_id: entry.op.op_id,
                    denial,
                })?;
            let before = scratch.quarantined.len();
            shadow.apply(entry.op.clone(), &mut scratch);
            if scratch.quarantined[before..]
                .iter()
                .any(|q| q.op_id == entry.op.op_id)
            {
                return Err(BatchFailure::WouldQuarantine {
                    op_id: entry.op.op_id,
                });
            }
        }
    }

    // Durable append: absent ops only, one fsync.
    let absent_ops: Vec<Operation> = absent.iter().map(|entry| entry.op.clone()).collect();
    if !absent_ops.is_empty() {
        log.append_batch(&absent_ops)
            .map_err(|err| BatchFailure::Store(err.to_string()))?;
    }

    // Index + real fold. Same ops against the same state as the preflight
    // inside one critical section: the fold outcome cannot diverge.
    for entry in &absent {
        index.map.insert(entry.op.op_id, entry.digest.clone());
        state.apply(entry.op.clone(), report);
    }
    Ok(dispositions)
}

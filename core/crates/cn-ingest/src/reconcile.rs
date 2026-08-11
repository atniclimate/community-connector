//! Receipt-ledger reconciliation classification for the remote puller
//! (blueprint docs/blueprints/intake-relay.md section 6.1 post-loop; ADR-005 D6
//! "Classification precedence"). After the main pull loop, every ledger receipt
//! is classified into one of five DISJOINT states by STRICT PRECEDENCE: local
//! durable facts (strongly consistent) outrank the eventually-consistent relay
//! observations. This module is that precedence as pure logic - one enum, one
//! function over plain data. "now" is folded into an explicit `age` input,
//! never read from a system clock here (a classifier that read the clock would
//! be non-deterministic and untestable).
//!
//! Scope is ONLY the per-receipt classifier. The run-report aggregation
//! (oldest-pending age, half-TTL warning, expired count) belongs to the
//! puller's post-loop report (blueprint step 7), which needs the TTL config and
//! every receipt in hand; it is deliberately not here. The ADR's "re-checked
//! once next run before alarming" for alerts is emergent from calling this same
//! stateless classifier again on a later run - nothing is tracked internally.
//!
//! No file or network I/O (ADR-005 D1 module fence): the caller (the CLI
//! puller) gathers the local and relay observations; this crate only decides.

use serde::{Deserialize, Serialize};

/// One receipt's classification in the D6 precedence partition. Serialized into
/// the puller run report (I12), so integrity alerts and expiries are visible,
/// never dropped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptClass {
    /// A local transport record exists for this receipt: already pulled and
    /// staged. A strongly-consistent local fact - outranks every relay
    /// observation.
    Staged,
    /// A local delete-journal entry exists: this puller already deleted the
    /// blob (the relay may still list it under eventual consistency).
    DeletedByMe,
    /// No local record, and the relay still holds the blob: awaiting a pull.
    Unpulled,
    /// No local record, blob absent, and the receipt is younger than the
    /// TTL + margin threshold: an AMBIGUOUS integrity alert (possible relay-
    /// side deletion or a failed-blob-write orphan ledger entry). Re-checked
    /// next run before alarming - that recheck is just re-calling this.
    IntegrityAlert,
    /// No local record, blob absent, and the receipt has reached the
    /// TTL + margin threshold: the blob self-drained on schedule. Counted;
    /// drives the broadcast re-solicit.
    Expired,
}

/// What the puller observed for one ledger receipt: the two strongly-consistent
/// LOCAL facts, the eventually-consistent relay blob presence, and the
/// receipt's age relative to the TTL + margin threshold. The caller folds "now"
/// into `age` - it is never read here.
#[derive(Debug, Clone, PartialEq)]
pub struct ReceiptObservation {
    /// A local queue record carries this receipt id (strongly consistent).
    pub local_transport_record: bool,
    /// A local delete-journal entry records this puller deleting the blob.
    pub local_delete_journal: bool,
    /// The relay still lists/serves the ciphertext blob (eventually
    /// consistent - consulted ONLY when no local fact applies).
    pub relay_blob_present: bool,
    /// Receipt age in the caller's chosen unit, or `None` when the age is
    /// unknown (e.g. a missing/unreadable ledger arrival timestamp). Same unit
    /// as `ttl_margin_threshold`.
    pub age: Option<i64>,
    /// The blob-TTL + consistency-margin threshold, in the same unit as `age`.
    pub ttl_margin_threshold: i64,
}

/// Classifies one ledger receipt by the ADR-005 D6 precedence. Evaluated in
/// this order the five states are a disjoint partition: local durable facts
/// first, then the eventually-consistent relay observations. A receipt that is
/// BOTH locally staged AND blob-absent-past-TTL classifies [`ReceiptClass::Staged`],
/// never [`ReceiptClass::Expired`] - local durability wins.
pub fn classify_receipt(obs: &ReceiptObservation) -> ReceiptClass {
    // Strongly-consistent LOCAL facts take precedence (ADR-005 D6).
    if obs.local_transport_record {
        return ReceiptClass::Staged;
    }
    if obs.local_delete_journal {
        return ReceiptClass::DeletedByMe;
    }
    // No local record: consult the eventually-consistent relay observations.
    if obs.relay_blob_present {
        return ReceiptClass::Unpulled;
    }
    // Blob absent. Expiry is a claim that needs evidence the observation window
    // has fully elapsed; without an age we cannot make it, so an unknown age
    // stays an alert (D-082). With an age, the boundary counts as expired: the
    // threshold already bakes in the consistency margin, so `>=` treats a
    // receipt exactly at the threshold as drained rather than perpetually
    // alerting (D-082).
    match obs.age {
        Some(age) if age >= obs.ttl_margin_threshold => ReceiptClass::Expired,
        Some(_) => ReceiptClass::IntegrityAlert,
        None => ReceiptClass::IntegrityAlert,
    }
}

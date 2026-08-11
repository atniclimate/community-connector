//! Consent-text digest recognition for the remote puller (blueprint
//! docs/blueprints/intake-relay.md section 6.1 step 9; ADR-005 D3 consent
//! block). After a submission decrypts, its `consent_text_digest` is checked
//! against the puller config's `known_consent_digests` (blueprint 6.3). An
//! UNRECOGNIZED digest is a WARNING, never a halt: the deployed form may have
//! been redeployed with fresh consent text between a submitter's affirmation
//! and this pull, so a digest we do not recognize is expected and non-fatal.
//! The outcome is therefore an exhaustively-matched enum, never a `Result` and
//! never a bool - "unrecognized" must be structurally impossible to silently
//! drop (I3).
//!
//! Pure comparison over plain strings: no file or network I/O (ADR-005 D1
//! module fence). The caller (the `cn intake pull` CLI) reads the known-digest
//! list from off-repo config and passes it in; this crate never touches the
//! network.
//!
//! This is NOT the `consent_affirmed == true` gate (already enforced loudly by
//! [`crate::InnerPayload::parse`] and [`crate::stage_remote_record`]), and NOT
//! the digest-SHAPE warning in submission validation (D-078.3, "does this look
//! like a SHA-256 hex string"). This asks the distinct question: "is this
//! digest one we recognize as currently deployed?"

use serde::{Deserialize, Serialize};

/// Whether a submission's consent-text digest is one the puller recognizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentDigestVerdict {
    /// The digest matches a known deployed consent-text digest.
    Known,
    /// Unrecognized digest: a WARNING surfaced in the run report (I12), never
    /// a halt (blueprint 6.1 step 9). The form may have redeployed between the
    /// submitter's affirmation and this pull.
    Unknown,
}

/// Checks a submission's `consent_text_digest` against the puller's known
/// deployed digests (blueprint 6.3 `known_consent_digests`). Exact string
/// equality: the config stores canonical lowercase hex and the submission
/// carries the same, so folding case would hide a real mismatch rather than
/// surface it. An empty known list (the realistic pre-D-023 pilot state, before
/// any consent text is signed off) yields [`ConsentDigestVerdict::Unknown`] for
/// every digest - never a panic or error.
pub fn check_consent_digest(
    consent_text_digest: &str,
    known_digests: &[String],
) -> ConsentDigestVerdict {
    if known_digests
        .iter()
        .any(|known| known == consent_text_digest)
    {
        ConsentDigestVerdict::Known
    } else {
        ConsentDigestVerdict::Unknown
    }
}

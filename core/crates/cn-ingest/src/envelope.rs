//! Remote-intake envelope formats (ADR-005 D3; docs/blueprints/intake-relay.md
//! section 2). The remote path has two encryption layers, and both cross this
//! crate as PURE data:
//!
//! - [`OuterEnvelope`]: cleartext metadata plus base64 ciphertext - exactly
//!   what the relay stores and can never read.
//! - [`InnerPayload`]: the decrypted form submission the puller stages.
//!
//! This module is types, parsing, validation, and the pure conversion from a
//! parsed inner payload to a [`QueueRecord`] with [`SubmissionSource::Remote`].
//! NO crypto and NO network code ever enter here (ADR-005 D1 module fence): the
//! ciphertext is an OPAQUE base64 string that the crypto binding (blueprint
//! phase A) and the puller (phase D) own end to end. Decoding it here would
//! prematurely pin a base64 variant the crypto binding must fix alongside
//! libsodium, so the envelope carries the ciphertext verbatim.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use cn_model::Timestamp;

use crate::record::{QueueRecord, SubmissionSource};
use crate::version::IngestError;

/// Current outer-envelope format version (I7). Its OWN version line, distinct
/// from the queue record version: conflating independent version spaces is the
/// bug the round-2 fixes closed, so envelope parsing never checks against
/// `QUEUE_RECORD_VERSION`.
pub const INTAKE_ENVELOPE_VERSION: &str = "0.1.0";

/// Current inner-payload (submission) format version (I7). Its own version line.
pub const SUBMISSION_VERSION: &str = "0.1.0";

/// A conservative default outer-envelope size cap in bytes for the
/// relay-storage boundary (ADR-005 D6: single-digit KB blobs). This is NOT the
/// authority - the relay enforces its own deploy-runbook cap and the puller
/// passes an explicit value; this default exists so callers and tests have a
/// sane baseline.
pub const DEFAULT_MAX_ENVELOPE_BYTES: usize = 8 * 1024;

/// Rejects unknown MAJOR versions loudly against a format's OWN current version
/// line; unknown minor is tolerated (each format's `extras` map preserves it).
/// Mirrors `version::check_major`, but takes the current version explicitly so
/// the envelope formats never borrow the queue record's version space.
fn check_format_major(version: &semver::Version, current: &str) -> Result<(), IngestError> {
    let current = semver::Version::parse(current).expect("valid const");
    if version.major != current.major {
        return Err(IngestError::UnknownMajorVersion {
            found: version.clone(),
        });
    }
    Ok(())
}

/// Outer envelope: cleartext metadata plus the sealed box, base64-encoded. This
/// is the ONLY artifact the relay stores; it holds no readable personal data
/// (ADR-005 D3). The relay never decrypts, and this type never tries to.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OuterEnvelope {
    /// Outer-envelope format version (unknown MAJOR rejected loudly, I7).
    pub intake_envelope_version: semver::Version,
    /// Fingerprint of the recipient key the submitter sealed to - a routing
    /// hint only; the puller verifies the real recipient by decryption.
    pub recipient_key_fingerprint: String,
    /// Base64-encoded libsodium sealed box. Opaque here: decoded only by the
    /// puller's crypto binding.
    pub ciphertext: String,
    /// Unknown-minor fields preserved across read/write (I7).
    #[serde(flatten)]
    pub extras: BTreeMap<String, Value>,
}

impl OuterEnvelope {
    /// Parses an outer envelope from bytes. Size is validated against `max_size`
    /// BEFORE any parsing (ADR-005 D6): a hostile blob must be rejected before
    /// it drives the parser, so the size gate precedes `from_slice`. Unknown
    /// MAJOR is rejected; unknown minor is preserved in [`OuterEnvelope::extras`].
    pub fn parse(bytes: &[u8], max_size: usize) -> Result<Self, IngestError> {
        if bytes.len() > max_size {
            return Err(IngestError::OversizedPayload {
                max: max_size,
                actual: bytes.len(),
            });
        }
        let envelope: OuterEnvelope =
            serde_json::from_slice(bytes).map_err(|err| IngestError::Serialize(err.to_string()))?;
        check_format_major(&envelope.intake_envelope_version, INTAKE_ENVELOPE_VERSION)?;
        Ok(envelope)
    }
}

/// Inner payload: the decrypted form submission the puller stages. Every field
/// here is a client assertion (source_asserted), including the consent block -
/// the durable owner treats it accordingly (ADR-005 D4).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InnerPayload {
    /// Submission format version (unknown MAJOR rejected loudly, I7).
    pub submission_version: semver::Version,
    /// Client-generated submission UUID: the semantic dedup key (ADR-005 D4).
    pub submission_id: String,
    /// Deployed form version string (provenance).
    pub form_version: String,
    /// Consent affirmation block (D-030).
    pub consent: ConsentBlock,
    /// Client ISO timestamp of capture (submitter clock; never trusted for
    /// ordering).
    pub captured_at: String,
    /// Payload-carried kind, present when the form offered a kind picker
    /// (D-069). Absent means the puller falls back to the default pilot kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Form field values (typed per the group template; opaque map here).
    pub fields: BTreeMap<String, Value>,
    /// Unknown-minor fields preserved across read/write (I7).
    #[serde(flatten)]
    pub extras: BTreeMap<String, Value>,
}

impl InnerPayload {
    /// Parses a decrypted inner payload. Unknown MAJOR rejected; unknown minor
    /// preserved. `consent_affirmed == false` is a LOUD failure (I3): the form
    /// prevents it structurally (D-030 checkbox gate), so seeing it means a
    /// hand-crafted payload that must never stage silently.
    pub fn parse(bytes: &[u8]) -> Result<Self, IngestError> {
        let payload: InnerPayload =
            serde_json::from_slice(bytes).map_err(|err| IngestError::Serialize(err.to_string()))?;
        check_format_major(&payload.submission_version, SUBMISSION_VERSION)?;
        if !payload.consent.consent_affirmed {
            return Err(IngestError::ConsentNotAffirmed);
        }
        Ok(payload)
    }
}

/// Consent affirmation block (ADR-005 D3, D-030). `consent_affirmed` MUST be
/// true to stage; the form's checkbox gate guarantees it and every downstream
/// boundary re-checks (defense in depth). This block is a fixed, security-
/// critical shape - it carries no `extras` map by design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsentBlock {
    /// Digest of the exact consent text the submitter affirmed.
    pub consent_text_digest: String,
    /// The affirmation itself. MUST be true to stage (D-030).
    pub consent_affirmed: bool,
    /// Client ISO timestamp of affirmation.
    pub consent_affirmed_at: String,
}

/// The pull-time context the puller supplies when staging a remote record.
/// These are the remote-only evidence fields of [`SubmissionSource::Remote`];
/// NONE of them come from the (untrusted) inner payload - they are the puller's
/// own observations (receipt id, the key that actually opened the box, the
/// ciphertext hash it computed, its own pull clock).
pub struct RemoteContext {
    /// Recipient fingerprint claimed by the outer envelope (routing hint).
    pub claimed_fingerprint: String,
    /// Outer-envelope version as received (string form, provenance).
    pub envelope_version: String,
    /// Fingerprint of the key that actually decrypted the box.
    pub key_used: String,
    /// Relay receipt id.
    pub receipt_id: String,
    /// Relay-reported arrival time, when available.
    pub relay_received_at: Option<Timestamp>,
    /// Local pull time (puller clock).
    pub pulled_at: Timestamp,
    /// SHA-256 over the ciphertext, computed before decryption.
    pub ciphertext_hash: String,
}

/// Converts a parsed inner payload into a staged [`QueueRecord`] with
/// [`SubmissionSource::Remote`]. Pure: no I/O, no crypto. The record's payload
/// is the inner payload VERBATIM (`serde_json::to_value`), preserving every
/// field including unknown-minor extras, so dedup, kind resolution, and
/// approval planning read exactly what the submitter sent. Consent is
/// re-checked here because staging is the gate to the queue: a hand-crafted
/// [`InnerPayload`] that skipped [`InnerPayload::parse`] must not slip through
/// (D-030).
pub fn stage_remote_record(
    inner: &InnerPayload,
    ctx: RemoteContext,
    record_id: String,
    staged_at: Timestamp,
) -> Result<QueueRecord, IngestError> {
    if !inner.consent.consent_affirmed {
        return Err(IngestError::ConsentNotAffirmed);
    }
    let payload =
        serde_json::to_value(inner).map_err(|err| IngestError::Serialize(err.to_string()))?;
    let source = SubmissionSource::Remote {
        claimed_fingerprint: ctx.claimed_fingerprint,
        envelope_version: ctx.envelope_version,
        key_used: ctx.key_used,
        receipt_id: ctx.receipt_id,
        relay_received_at: ctx.relay_received_at,
        pulled_at: ctx.pulled_at,
        ciphertext_hash: ctx.ciphertext_hash,
    };
    QueueRecord::new(record_id, staged_at, source, payload)
}

use serde::{Deserialize, Deserializer, Serialize};

use crate::{CustodyEventId, ModelError, OpId, PersonId, Timestamp, model_schema_version};

/// Origin of a modeled value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    /// The value was reported by the person it describes.
    SelfReported,
    /// The value was imported from a named source.
    Ingested {
        /// Source name or identifier.
        source: String,
    },
    /// The value was derived from operation inputs.
    Derived {
        /// Operation inputs used to derive this value.
        inputs: Vec<OpId>,
    },
    /// The value was authored directly.
    Authored,
}

/// Actor reference for provenance and custody records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorRef {
    /// Human actor.
    Human(PersonId),
    /// Automated agent actor.
    Agent {
        /// Agent identifier.
        agent_id: String,
    },
}

/// Custody event action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CustodyAction {
    /// Created action.
    Created,
    /// Imported action.
    Imported,
    /// Migrated action.
    Migrated,
    /// Corrected action.
    Corrected,
    /// Exported action.
    Exported,
}

/// Append-only custody event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustodyEvent {
    /// Custody event identifier.
    pub id: CustodyEventId,
    /// Custody action.
    pub action: CustodyAction,
    /// Event timestamp.
    pub at: Timestamp,
    /// Event actor.
    pub actor: ActorRef,
    /// Optional custody note.
    pub note: Option<String>,
}

/// Current intake provenance block version (ADR-005 D5, I7).
pub const INTAKE_BLOCK_VERSION: &str = "0.1.0";

/// Intake linkage carried on entities/edges produced by an approved
/// intake submission (ADR-005 D5). Identifier and consent fields marked
/// `source_asserted` are client-controlled claims from the anonymous
/// submission path, recorded verbatim and never treated as verified fact;
/// the digests and record id are locally computed at approval-plan time.
/// This block - including the affirmative consent assertion - is the
/// audit evidence that survives the recorded close-of-window purge sweep.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntakeProvenance {
    /// Intake block format version (unknown MAJOR is rejected on read).
    pub intake_block_version: semver::Version,
    /// Queue record id (puller/wizard-generated UUIDv7).
    pub record_id: String,
    /// Relay receipt id (remote submissions only).
    pub receipt_id: Option<String>,
    /// Client-generated submission UUID (source_asserted; semantic dedup key).
    pub submission_id: String,
    /// Form version the submitter claims they saw (source_asserted).
    pub form_version: String,
    /// Digest of the consent statement displayed (source_asserted).
    pub consent_text_digest: String,
    /// Affirmative consent assertion (source_asserted; must be true to send).
    pub consent_affirmed: bool,
    /// Claimed consent time (source_asserted; never used for ordering).
    pub consent_affirmed_at: Timestamp,
    /// Canonical digest of the decrypted inner payload.
    pub payload_digest: String,
    /// Pre-link approval batch digest (ADR-005 D5 projection).
    pub batch_digest: String,
}

/// Provenance envelope attached to modeled data.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProvenanceEnvelope {
    origin: Origin,
    recorded_by: ActorRef,
    responsible_human: PersonId,
    recorded_at: Timestamp,
    custody: Vec<CustodyEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    intake: Option<IntakeProvenance>,
    schema_version: semver::Version,
}

impl ProvenanceEnvelope {
    /// Creates a provenance envelope with validated accountability.
    pub fn new(
        origin: Origin,
        recorded_by: ActorRef,
        responsible_human: PersonId,
        recorded_at: Timestamp,
    ) -> Result<Self, ModelError> {
        validate_accountability(&recorded_by, responsible_human)?;
        Ok(Self {
            origin,
            recorded_by,
            responsible_human,
            recorded_at,
            custody: Vec::new(),
            intake: None,
            schema_version: model_schema_version(),
        })
    }

    /// Attaches the intake linkage block (ADR-005 D5). Set by the approval
    /// plan when it builds intake-created modeled values; absent everywhere
    /// else.
    pub fn set_intake(&mut self, intake: IntakeProvenance) {
        self.intake = Some(intake);
    }

    /// Returns the intake linkage block when this value came through the
    /// intake pipeline.
    pub fn intake(&self) -> Option<&IntakeProvenance> {
        self.intake.as_ref()
    }

    /// Appends a custody event.
    pub fn append_custody(&mut self, event: CustodyEvent) {
        self.custody.push(event);
    }

    /// Returns custody events in append order.
    pub fn custody(&self) -> &[CustodyEvent] {
        &self.custody
    }

    /// Returns the origin.
    pub fn origin(&self) -> &Origin {
        &self.origin
    }

    /// Returns the recording actor.
    pub fn recorded_by(&self) -> &ActorRef {
        &self.recorded_by
    }

    /// Returns the accountable human.
    pub fn responsible_human(&self) -> PersonId {
        self.responsible_human
    }

    /// Returns the recorded timestamp.
    pub fn recorded_at(&self) -> Timestamp {
        self.recorded_at
    }

    /// Returns the envelope schema version.
    pub fn schema_version(&self) -> &semver::Version {
        &self.schema_version
    }
}

#[derive(Deserialize)]
struct RawEnvelope {
    origin: Origin,
    recorded_by: ActorRef,
    responsible_human: PersonId,
    recorded_at: Timestamp,
    custody: Vec<CustodyEvent>,
    #[serde(default)]
    intake: Option<IntakeProvenance>,
    schema_version: semver::Version,
}

impl<'de> Deserialize<'de> for ProvenanceEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawEnvelope::deserialize(deserializer)?;
        validate_accountability(&raw.recorded_by, raw.responsible_human)
            .map_err(serde::de::Error::custom)?;
        if let Some(intake) = &raw.intake {
            let current =
                semver::Version::parse(INTAKE_BLOCK_VERSION).map_err(serde::de::Error::custom)?;
            if intake.intake_block_version.major != current.major {
                return Err(serde::de::Error::custom(format!(
                    "unknown intake block major version {}",
                    intake.intake_block_version
                )));
            }
        }
        Ok(Self {
            origin: raw.origin,
            recorded_by: raw.recorded_by,
            responsible_human: raw.responsible_human,
            recorded_at: raw.recorded_at,
            custody: raw.custody,
            intake: raw.intake,
            schema_version: raw.schema_version,
        })
    }
}

fn validate_accountability(
    recorded_by: &ActorRef,
    responsible_human: PersonId,
) -> Result<(), ModelError> {
    if let ActorRef::Human(person) = recorded_by
        && *person != responsible_human
    {
        return Err(ModelError::ResponsibleHumanMismatch);
    }
    Ok(())
}

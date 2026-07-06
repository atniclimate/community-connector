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

/// Provenance envelope attached to modeled data.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ProvenanceEnvelope {
    origin: Origin,
    recorded_by: ActorRef,
    responsible_human: PersonId,
    recorded_at: Timestamp,
    custody: Vec<CustodyEvent>,
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
            schema_version: model_schema_version(),
        })
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
        Ok(Self {
            origin: raw.origin,
            recorded_by: raw.recorded_by,
            responsible_human: raw.responsible_human,
            recorded_at: raw.recorded_at,
            custody: raw.custody,
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

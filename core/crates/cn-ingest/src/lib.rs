//! cn-ingest: the intake pipeline's filesystem-free domain logic
//! (ADR-005 D4/D5; docs/blueprints/intake-pipeline.md).
//!
//! All I/O stays in callers (the app over FSA creates records and decision
//! files; native `cn intake apply` owns every mutation). This crate models
//! the persisted queue formats, the decision-inbox admission rules, and the
//! crash-recovery classification as pure, exhaustively testable logic. No
//! network code may ever enter this crate (ADR-005 D1 module fence).

pub mod decision;
pub mod record;
pub mod recovery;
mod version;

pub use decision::*;
pub use record::*;
pub use recovery::*;
pub use version::{IngestError, QUEUE_RECORD_VERSION, canonical_digest, new_uuid_v7};

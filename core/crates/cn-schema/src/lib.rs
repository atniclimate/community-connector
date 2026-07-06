//! Group template parsing and semantic validation.

mod report;
mod types;
mod validate;

pub use report::{Finding, FindingCode, ValidationReport};
pub use types::{
    AttrDef, AttrType, ColorRole, EdgeKindDef, GroupTemplate, HexColor, Shape, Theme, Weighted,
};
pub use validate::{parse_template, validate_edge, validate_entity};

/// Errors raised while structurally parsing template JSON.
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    /// Template JSON is structurally invalid for the template shape.
    #[error("invalid template json: {0}")]
    Parse(String),
}

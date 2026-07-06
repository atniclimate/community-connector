use serde::Serialize;

/// Machine-readable validation report for schema and instance checks.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct ValidationReport {
    /// Rejecting validation findings.
    pub errors: Vec<Finding>,
    /// Visible non-rejecting validation findings.
    pub warnings: Vec<Finding>,
}

impl ValidationReport {
    pub(crate) fn error(&mut self, code: FindingCode, path: impl Into<String>, message: String) {
        self.errors.push(Finding {
            code,
            message,
            path: path.into(),
        });
    }

    pub(crate) fn warning(&mut self, code: FindingCode, path: impl Into<String>, message: String) {
        self.warnings.push(Finding {
            code,
            message,
            path: path.into(),
        });
    }
}

/// Single validation finding.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding {
    /// Stable machine-readable finding code.
    pub code: FindingCode,
    /// Human-readable finding message.
    pub message: String,
    /// JSON-pointer-like path to the offending value.
    pub path: String,
}

/// Stable validation finding codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FindingCode {
    /// Template schema version is not accepted by cn-model.
    UnsupportedSchemaVersion,
    /// Template declares no entity kinds.
    NoKinds,
    /// Template repeats an entity kind id.
    DuplicateKindId,
    /// A kind repeats an attribute id.
    DuplicateAttrId,
    /// Template repeats an edge kind id.
    DuplicateEdgeKindId,
    /// Enum attribute does not declare values.
    EnumValuesMissing,
    /// Enum attribute declares an empty values list.
    EnumValuesEmpty,
    /// Enum attribute repeats a value.
    EnumValuesDuplicate,
    /// Non-enum attribute declares values.
    ValuesOnNonEnum,
    /// Non-link attribute declares a format.
    FormatOnNonLink,
    /// Edge kind references an unknown entity kind.
    UnknownKindRef,
    /// Theme declares a role no kind uses.
    ThemeRoleUnknown,
    /// Kind color role has no theme entry.
    ColorRoleUnthemed,
    /// Template declares more than the design brief kind limit.
    TooManyKinds,
    /// Entity or edge kind is not declared by the template.
    KindMismatch,
    /// Attribute value variant does not match its template type.
    AttrTypeMismatch,
    /// Entity attribute is not declared by its kind.
    UnknownAttr,
    /// Required entity attribute is absent.
    RequiredAttrMissing,
    /// Entity enum value is outside the template values.
    EnumValueNotInTemplate,
    /// Edge carries a weight where weights are forbidden.
    WeightForbidden,
    /// Edge lacks a weight where weights are required.
    WeightRequired,
    /// Edge endpoint kind is outside the edge-kind policy.
    EndpointKindNotAllowed,
}

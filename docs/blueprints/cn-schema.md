# Blueprint: cn-schema (Phase 2, authored by director)

Implements group-template parsing and validation in Rust, the semantic twin
of `schemas/group-template.schema.json` (which stays authoritative for
non-Rust consumers) plus SEMANTIC checks JSON Schema cannot express. Source
ADRs: ADR-001 D2/D4/D9 + A-B5/A-B6 amendments; invariants I3, I7, I12.

## Dependencies

```toml
[dependencies]
cn-model = { path = "../cn-model" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
semver = { version = "1", features = ["serde"] }
thiserror = "2"
```

## Types (all serde with `deny_unknown_fields` - parity with
additionalProperties: false)

```rust
pub struct GroupTemplate {
    pub schema_version: semver::Version,     // accepted iff cn_model::accepts_schema
    pub template_id: TemplateId,             // cn-model newtype
    pub name: String,
    pub description: String,
    #[serde(default)] pub vocabulary: BTreeMap<String, String>,
    pub kinds: Vec<KindDef>,                 // min 1 (semantic check)
    pub edge_kinds: Vec<EdgeKindDef>,
    #[serde(default)] pub theme: Option<Theme>,
}

pub struct KindDef {
    pub id: KindId, pub label: String,
    pub shape: Shape, pub color_role: ColorRole,
    pub attributes: Vec<AttrDef>,
}

#[serde(rename_all = "lowercase")]
pub enum Shape { Sphere, Cube, Octahedron, Tetrahedron, Torus, Cone }

pub struct ColorRole(String);   // validated ^kind-[1-9][0-9]*$

pub struct AttrDef {
    pub id: AttrId, #[serde(rename = "type")] pub attr_type: AttrType,
    #[serde(default)] pub required: bool,
    #[serde(default)] pub values: Option<Vec<String>>,   // iff attr_type == Enum
    #[serde(default)] pub format: Option<LinkFormat>,    // iff attr_type == Link (reuse cn-model LinkFormat)
    #[serde(default = "default_visibility_group")] pub default_visibility: Circle,  // default Group
}

#[serde(rename_all = "lowercase")]
pub enum AttrType { Text, Number, Enum, Tags, Date, Geo, Link, Media }

pub struct EdgeKindDef {
    pub id: KindId, pub label: String,
    pub from: Vec<KindId>, pub to: Vec<KindId>,   // min 1 each
    pub directed: bool,
    #[serde(default)] pub weighted: Weighted,     // default Forbidden (ADR-001)
}

#[serde(rename_all = "lowercase")]
pub enum Weighted { Required, Optional, #[default] Forbidden }

pub struct Theme { pub mode: String, pub roles: BTreeMap<String, HexColor> }
pub struct HexColor(String);    // validated ^#[0-9a-f]{6}$
```

## Validation report (I12; machine-readable)

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidationReport { pub errors: Vec<Finding>, pub warnings: Vec<Finding> }
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Finding { pub code: FindingCode, pub message: String, pub path: String }
#[derive(...)] pub enum FindingCode {
    UnsupportedSchemaVersion, NoKinds, DuplicateKindId, DuplicateAttrId,
    DuplicateEdgeKindId, EnumValuesMissing, EnumValuesEmpty, EnumValuesDuplicate,
    ValuesOnNonEnum, FormatOnNonLink, UnknownKindRef, ThemeRoleUnknown,
    ColorRoleUnthemed, TooManyKinds, KindMismatch, AttrTypeMismatch,
    UnknownAttr, RequiredAttrMissing, EnumValueNotInTemplate,
    WeightForbidden, WeightRequired, EndpointKindNotAllowed,
}
```

NOTE (recorded design debt): report types will migrate to a shared crate when
cn-store/cn-ingest need them; cn-api unifies reports at the boundary. Do not
preemptively move them.

## Functions

```rust
/// Parse + fully validate a template document. Structural failures (bad
/// JSON, unknown fields, bad newtype formats) -> Err(SchemaError). Semantic
/// findings -> the report; errors non-empty means the template is REJECTED
/// by callers, warnings ride along (I12: visible, never dropped).
pub fn parse_template(json: &str) -> Result<(GroupTemplate, ValidationReport), SchemaError>;
```

Semantic checks producing report entries:
- schema_version not accepted (UnsupportedSchemaVersion - error)
- kinds empty (NoKinds - error); duplicate kind ids, duplicate attr ids per
  kind, duplicate edge-kind ids (errors)
- values: missing/empty/duplicate when Enum (errors); present on non-Enum
  (ValuesOnNonEnum - error); format on non-Link (error)
- edge from/to referencing unknown kind ids (UnknownKindRef - error; STRICTER
  than the JSON Schema, deliberately)
- theme.roles key not matching any kind's color_role (ThemeRoleUnknown -
  warning); kind color_role with no theme entry when a theme IS present
  (ColorRoleUnthemed - warning)
- more than 8 kinds (TooManyKinds - WARNING, design brief rule)

```rust
/// Instance validation against a validated template (used by ingest/store).
pub fn validate_entity(tpl: &GroupTemplate, entity: &cn_model::Entity) -> ValidationReport;
pub fn validate_edge(tpl: &GroupTemplate, edge: &cn_model::Edge,
                     from_kind: &KindId, to_kind: &KindId) -> ValidationReport;
```

- validate_entity: kind exists (KindMismatch), every attribute id declared
  (UnknownAttr), AttributeValue variant matches AttrType (AttrTypeMismatch),
  Enum value in template values (EnumValueNotInTemplate), required attrs
  present (RequiredAttrMissing).
- validate_edge: edge kind exists, endpoint kinds allowed by from/to
  (EndpointKindNotAllowed), weight vs Weighted policy (WeightForbidden /
  WeightRequired).

```rust
#[derive(Debug, thiserror::Error)]
pub enum SchemaError {
    #[error("invalid template json: {0}")] Parse(String),   // wraps serde_json error text
}
```

## Test obligations

1. Both fixture templates (include_str! from
   `../../../../fixtures/templates/`) parse with ZERO errors; assert the
   exact warning set (expected: empty).
2. Negative structural: unknown top-level field rejected; bad shape value;
   bad hex color; bad color_role pattern; bad slug ids.
3. Negative semantic: enum without values; values on text; format on number;
   edge referencing unknown kind; duplicate kind id; 9 kinds -> TooManyKinds
   warning (and zero errors if otherwise valid).
4. Instance validation: entity with undeclared attr; wrong value type; enum
   symbol outside template; required missing; edge weight on Forbidden;
   missing weight on Required; endpoint kind not allowed.
5. Weighted default is Forbidden when field absent (fixture edges without
   `weighted` reject weights).

Verification identical to cn-model: fmt, clippy -D warnings, tests, plus
`cargo build --target wasm32-unknown-unknown -p cn-schema`.

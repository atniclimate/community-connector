use std::collections::{BTreeMap, BTreeSet};

use cn_model::{AttrId, AttributeValue, Edge, Entity, KindId, accepts_schema};

use crate::report::{FindingCode, ValidationReport};
use crate::types::{AttrDef, AttrType, EdgeKindDef, GroupTemplate, Weighted};

/// Parse and fully validate a template document.
pub fn parse_template(json: &str) -> Result<(GroupTemplate, ValidationReport), crate::SchemaError> {
    let template: GroupTemplate =
        serde_json::from_str(json).map_err(|err| crate::SchemaError::Parse(err.to_string()))?;
    let report = validate_template(&template);
    Ok((template, report))
}

/// Validate an entity instance against a validated template.
pub fn validate_entity(tpl: &GroupTemplate, entity: &Entity) -> ValidationReport {
    let mut report = ValidationReport::default();
    let Some(kind) = tpl.kinds.iter().find(|kind| kind.id == entity.kind) else {
        report.error(
            FindingCode::KindMismatch,
            "/kind",
            format!("entity kind '{}' is not declared", entity.kind),
        );
        return report;
    };

    let defs: BTreeMap<&AttrId, &AttrDef> =
        kind.attributes.iter().map(|def| (&def.id, def)).collect();
    validate_entity_attributes(entity, &defs, &mut report);
    validate_required_attributes(kind.attributes.as_slice(), entity, &mut report);
    report
}

/// Validate an edge instance against a validated template.
pub fn validate_edge(
    tpl: &GroupTemplate,
    edge: &Edge,
    from_kind: &KindId,
    to_kind: &KindId,
) -> ValidationReport {
    let mut report = ValidationReport::default();
    let Some(edge_kind) = tpl.edge_kinds.iter().find(|kind| kind.id == edge.kind) else {
        report.error(
            FindingCode::KindMismatch,
            "/kind",
            format!("edge kind '{}' is not declared", edge.kind),
        );
        return report;
    };

    validate_edge_endpoints(edge_kind, from_kind, to_kind, &mut report);
    validate_edge_weight(edge_kind, edge, &mut report);
    report
}

fn validate_template(tpl: &GroupTemplate) -> ValidationReport {
    let mut report = ValidationReport::default();
    validate_schema_version(tpl, &mut report);
    validate_kinds(tpl, &mut report);
    validate_edge_kinds(tpl, &mut report);
    validate_theme(tpl, &mut report);
    report
}

fn validate_schema_version(tpl: &GroupTemplate, report: &mut ValidationReport) {
    if !accepts_schema(&tpl.schema_version) {
        report.error(
            FindingCode::UnsupportedSchemaVersion,
            "/schema_version",
            format!("schema version '{}' is not supported", tpl.schema_version),
        );
    }
}

fn validate_kinds(tpl: &GroupTemplate, report: &mut ValidationReport) {
    if tpl.kinds.is_empty() {
        report.error(
            FindingCode::NoKinds,
            "/kinds",
            "template must declare at least one kind".to_string(),
        );
    }
    if tpl.kinds.len() > 8 {
        report.warning(
            FindingCode::TooManyKinds,
            "/kinds",
            "template declares more than 8 kinds".to_string(),
        );
    }

    let mut ids = BTreeSet::new();
    for (index, kind) in tpl.kinds.iter().enumerate() {
        if !ids.insert(&kind.id) {
            report.error(
                FindingCode::DuplicateKindId,
                format!("/kinds/{index}/id"),
                format!("duplicate kind id '{}'", kind.id),
            );
        }
        validate_kind_attrs(index, kind.attributes.as_slice(), report);
    }
}

fn validate_kind_attrs(kind_index: usize, attrs: &[AttrDef], report: &mut ValidationReport) {
    let mut ids = BTreeSet::new();
    for (attr_index, attr) in attrs.iter().enumerate() {
        if !ids.insert(&attr.id) {
            report.error(
                FindingCode::DuplicateAttrId,
                format!("/kinds/{kind_index}/attributes/{attr_index}/id"),
                format!("duplicate attribute id '{}'", attr.id),
            );
        }
        validate_attr_values(kind_index, attr_index, attr, report);
        validate_attr_format(kind_index, attr_index, attr, report);
    }
}

fn validate_attr_values(
    kind_index: usize,
    attr_index: usize,
    attr: &AttrDef,
    report: &mut ValidationReport,
) {
    match (&attr.attr_type, &attr.values) {
        (AttrType::Enum, None) => report.error(
            FindingCode::EnumValuesMissing,
            format!("/kinds/{kind_index}/attributes/{attr_index}/values"),
            format!("enum attribute '{}' must declare values", attr.id),
        ),
        (AttrType::Enum, Some(values)) if values.is_empty() => report.error(
            FindingCode::EnumValuesEmpty,
            format!("/kinds/{kind_index}/attributes/{attr_index}/values"),
            format!("enum attribute '{}' must declare non-empty values", attr.id),
        ),
        (AttrType::Enum, Some(values)) => {
            validate_enum_duplicates(kind_index, attr_index, attr, values.as_slice(), report)
        }
        (_, Some(_)) => report.error(
            FindingCode::ValuesOnNonEnum,
            format!("/kinds/{kind_index}/attributes/{attr_index}/values"),
            format!("non-enum attribute '{}' must not declare values", attr.id),
        ),
        (_, None) => {}
    }
}

fn validate_enum_duplicates(
    kind_index: usize,
    attr_index: usize,
    attr: &AttrDef,
    values: &[String],
    report: &mut ValidationReport,
) {
    let mut seen = BTreeSet::new();
    for (value_index, value) in values.iter().enumerate() {
        if !seen.insert(value) {
            report.error(
                FindingCode::EnumValuesDuplicate,
                format!("/kinds/{kind_index}/attributes/{attr_index}/values/{value_index}"),
                format!("enum attribute '{}' repeats value '{}'", attr.id, value),
            );
        }
    }
}

fn validate_attr_format(
    kind_index: usize,
    attr_index: usize,
    attr: &AttrDef,
    report: &mut ValidationReport,
) {
    if attr.attr_type != AttrType::Link && attr.format.is_some() {
        report.error(
            FindingCode::FormatOnNonLink,
            format!("/kinds/{kind_index}/attributes/{attr_index}/format"),
            format!("non-link attribute '{}' must not declare format", attr.id),
        );
    }
}

fn validate_edge_kinds(tpl: &GroupTemplate, report: &mut ValidationReport) {
    let kind_ids: BTreeSet<&KindId> = tpl.kinds.iter().map(|kind| &kind.id).collect();
    let mut edge_ids = BTreeSet::new();
    for (edge_index, edge) in tpl.edge_kinds.iter().enumerate() {
        if !edge_ids.insert(&edge.id) {
            report.error(
                FindingCode::DuplicateEdgeKindId,
                format!("/edge_kinds/{edge_index}/id"),
                format!("duplicate edge kind id '{}'", edge.id),
            );
        }
        validate_kind_refs(edge_index, "from", edge.from.as_slice(), &kind_ids, report);
        validate_kind_refs(edge_index, "to", edge.to.as_slice(), &kind_ids, report);
    }
}

fn validate_kind_refs(
    edge_index: usize,
    field: &str,
    refs: &[KindId],
    kind_ids: &BTreeSet<&KindId>,
    report: &mut ValidationReport,
) {
    if refs.is_empty() {
        report.error(
            FindingCode::UnknownKindRef,
            format!("/edge_kinds/{edge_index}/{field}"),
            format!("edge kind {field} must include at least one kind"),
        );
    }
    for (ref_index, kind_id) in refs.iter().enumerate() {
        if !kind_ids.contains(kind_id) {
            report.error(
                FindingCode::UnknownKindRef,
                format!("/edge_kinds/{edge_index}/{field}/{ref_index}"),
                format!("edge kind references unknown kind '{kind_id}'"),
            );
        }
    }
}

fn validate_theme(tpl: &GroupTemplate, report: &mut ValidationReport) {
    let Some(theme) = &tpl.theme else {
        return;
    };
    let color_roles: BTreeSet<&str> = tpl
        .kinds
        .iter()
        .map(|kind| kind.color_role.as_str())
        .collect();

    for role in theme.roles.keys() {
        if !color_roles.contains(role.as_str()) {
            report.warning(
                FindingCode::ThemeRoleUnknown,
                format!("/theme/roles/{role}"),
                format!("theme role '{role}' is not used by any kind"),
            );
        }
    }
    for (index, kind) in tpl.kinds.iter().enumerate() {
        if !theme.roles.contains_key(kind.color_role.as_str()) {
            report.warning(
                FindingCode::ColorRoleUnthemed,
                format!("/kinds/{index}/color_role"),
                format!("kind color role '{}' has no theme entry", kind.color_role),
            );
        }
    }
}

fn validate_entity_attributes(
    entity: &Entity,
    defs: &BTreeMap<&AttrId, &AttrDef>,
    report: &mut ValidationReport,
) {
    for (attr_id, attr) in &entity.attributes {
        let Some(def) = defs.get(attr_id) else {
            report.error(
                FindingCode::UnknownAttr,
                format!("/attributes/{attr_id}"),
                format!("attribute '{attr_id}' is not declared"),
            );
            continue;
        };
        validate_attribute_value(attr_id, def, &attr.value, report);
    }
}

fn validate_attribute_value(
    attr_id: &AttrId,
    def: &AttrDef,
    value: &AttributeValue,
    report: &mut ValidationReport,
) {
    if !value_matches_type(value, def.attr_type) {
        report.error(
            FindingCode::AttrTypeMismatch,
            format!("/attributes/{attr_id}/value"),
            format!("attribute '{attr_id}' value does not match template type"),
        );
        return;
    }
    if let (AttributeValue::Enum(value), Some(values)) = (value, &def.values)
        && !values.contains(value)
    {
        report.error(
            FindingCode::EnumValueNotInTemplate,
            format!("/attributes/{attr_id}/value"),
            format!("enum value '{value}' is not declared for attribute '{attr_id}'"),
        );
    }
}

fn validate_required_attributes(attrs: &[AttrDef], entity: &Entity, report: &mut ValidationReport) {
    for attr in attrs.iter().filter(|attr| attr.required) {
        if !entity.attributes.contains_key(&attr.id) {
            report.error(
                FindingCode::RequiredAttrMissing,
                format!("/attributes/{}", attr.id),
                format!("required attribute '{}' is missing", attr.id),
            );
        }
    }
}

fn validate_edge_endpoints(
    edge_kind: &EdgeKindDef,
    from_kind: &KindId,
    to_kind: &KindId,
    report: &mut ValidationReport,
) {
    if !edge_kind.from.contains(from_kind) {
        report.error(
            FindingCode::EndpointKindNotAllowed,
            "/from",
            format!(
                "from kind '{from_kind}' is not allowed for edge '{}'",
                edge_kind.id
            ),
        );
    }
    if !edge_kind.to.contains(to_kind) {
        report.error(
            FindingCode::EndpointKindNotAllowed,
            "/to",
            format!(
                "to kind '{to_kind}' is not allowed for edge '{}'",
                edge_kind.id
            ),
        );
    }
}

fn validate_edge_weight(edge_kind: &EdgeKindDef, edge: &Edge, report: &mut ValidationReport) {
    match (edge_kind.weighted, edge.weight) {
        (Weighted::Forbidden, Some(_)) => report.error(
            FindingCode::WeightForbidden,
            "/weight",
            format!("edge kind '{}' forbids weights", edge_kind.id),
        ),
        (Weighted::Required, None) => report.error(
            FindingCode::WeightRequired,
            "/weight",
            format!("edge kind '{}' requires a weight", edge_kind.id),
        ),
        (Weighted::Required | Weighted::Optional | Weighted::Forbidden, _) => {}
    }
}

fn value_matches_type(value: &AttributeValue, attr_type: AttrType) -> bool {
    matches!(
        (value, attr_type),
        (AttributeValue::Text(_), AttrType::Text)
            | (AttributeValue::Number(_), AttrType::Number)
            | (AttributeValue::Enum(_), AttrType::Enum)
            | (AttributeValue::Tags(_), AttrType::Tags)
            | (AttributeValue::Date(_), AttrType::Date)
            | (AttributeValue::Geo(_), AttrType::Geo)
            | (AttributeValue::Link(_), AttrType::Link)
            | (AttributeValue::Media(_), AttrType::Media)
    )
}

use std::collections::BTreeMap;

use cn_model::{
    ActorRef, AttrId, AttributeInstance, AttributeValue, Circle, Edge, EdgeId, Entity, EntityId,
    GroupId, KindId, Origin, PersonId, ProvenanceEnvelope, SensitivityTier, Timestamp,
};
use cn_schema::{FindingCode, ValidationReport, parse_template, validate_edge, validate_entity};
use serde_json::{Value, json};

const FISHERIES: &str =
    include_str!("../../../../fixtures/templates/fisheries-committee.template.json");
const RESEARCH: &str =
    include_str!("../../../../fixtures/templates/research-network.template.json");

fn timestamp(value: i64) -> Timestamp {
    Timestamp(value)
}

fn person_id(value: i64) -> PersonId {
    PersonId::new(timestamp(value))
}

fn provenance() -> ProvenanceEnvelope {
    let human = person_id(1);
    ProvenanceEnvelope::new(
        Origin::SelfReported,
        ActorRef::Human(human),
        human,
        timestamp(10),
    )
    .expect("valid provenance")
}

fn attr(value: AttributeValue) -> AttributeInstance {
    AttributeInstance::new(value, Circle::Group, provenance())
}

fn empty_entity(kind: &str) -> Entity {
    Entity::new(
        EntityId::new(timestamp(1)),
        GroupId::new(timestamp(2)),
        KindId::new(kind).expect("valid kind id"),
        Circle::Group,
        provenance(),
        SensitivityTier::T2,
    )
}

fn edge(kind: &str) -> Edge {
    Edge::new(
        EdgeId::new(timestamp(3)),
        GroupId::new(timestamp(2)),
        KindId::new(kind).expect("valid kind id"),
        EntityId::new(timestamp(4)),
        EntityId::new(timestamp(5)),
        true,
        Circle::Group,
        provenance(),
        SensitivityTier::T2,
    )
}

fn parse_value(value: Value) -> ValidationReport {
    let json = serde_json::to_string(&value).expect("serialize template");
    parse_template(&json).expect("parse template").1
}

fn report_codes(report: &ValidationReport) -> Vec<FindingCode> {
    report.errors.iter().map(|finding| finding.code).collect()
}

#[test]
fn fixture_templates_parse_with_no_findings() {
    for fixture in [FISHERIES, RESEARCH] {
        let (_template, report) = parse_template(fixture).expect("fixture parses");
        assert!(report.errors.is_empty());
        assert_eq!(report.warnings, Vec::new());
    }
}

#[test]
fn structural_failures_are_parse_errors() {
    let mut unknown_field: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    unknown_field["extra"] = json!(true);
    assert!(parse_template(&unknown_field.to_string()).is_err());

    let mut bad_shape: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    bad_shape["kinds"][0]["shape"] = json!("pyramid");
    assert!(parse_template(&bad_shape.to_string()).is_err());

    let mut bad_hex: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    bad_hex["theme"]["roles"]["kind-1"] = json!("#ABCDEF");
    assert!(parse_template(&bad_hex.to_string()).is_err());

    let mut bad_color_role: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    bad_color_role["kinds"][0]["color_role"] = json!("kind-0");
    assert!(parse_template(&bad_color_role.to_string()).is_err());

    let mut bad_slug: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    bad_slug["kinds"][0]["id"] = json!("Bad Slug");
    assert!(parse_template(&bad_slug.to_string()).is_err());
}

#[test]
fn semantic_template_failures_are_reported() {
    let mut missing_enum_values: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    missing_enum_values["kinds"][0]["attributes"][4]
        .as_object_mut()
        .expect("attribute object")
        .remove("values");
    assert_eq!(
        report_codes(&parse_value(missing_enum_values)),
        vec![FindingCode::EnumValuesMissing]
    );

    let mut values_on_text: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    values_on_text["kinds"][0]["attributes"][0]["values"] = json!(["name"]);
    assert_eq!(
        report_codes(&parse_value(values_on_text)),
        vec![FindingCode::ValuesOnNonEnum]
    );

    let mut format_on_number: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    format_on_number["kinds"][2]["attributes"][1]["format"] = json!("url");
    assert_eq!(
        report_codes(&parse_value(format_on_number)),
        vec![FindingCode::FormatOnNonLink]
    );

    let mut unknown_kind_ref: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    unknown_kind_ref["edge_kinds"][0]["from"] = json!(["missing_kind"]);
    assert_eq!(
        report_codes(&parse_value(unknown_kind_ref)),
        vec![FindingCode::UnknownKindRef]
    );

    let mut duplicate_kind: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    let duplicate = duplicate_kind["kinds"][0].clone();
    duplicate_kind["kinds"]
        .as_array_mut()
        .expect("kinds array")
        .push(duplicate);
    assert_eq!(
        report_codes(&parse_value(duplicate_kind)),
        vec![FindingCode::DuplicateKindId]
    );
}

#[test]
fn nine_kinds_warns_without_rejecting() {
    let mut template: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    template["theme"] = Value::Null;
    for n in 7..=9 {
        let mut kind = template["kinds"][0].clone();
        kind["id"] = json!(format!("extra_kind_{n}"));
        kind["color_role"] = json!(format!("kind-{n}"));
        template["kinds"]
            .as_array_mut()
            .expect("kinds array")
            .push(kind);
    }

    let report = parse_value(template);
    assert!(report.errors.is_empty());
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].code, FindingCode::TooManyKinds);
}

#[test]
fn instance_entity_validation_reports_expected_errors() {
    let (template, report) = parse_template(RESEARCH).expect("fixture parses");
    assert!(report.errors.is_empty());

    let mut entity = empty_entity("person");
    entity.attributes = BTreeMap::from([
        (
            AttrId::new("unknown_attr").expect("valid attr id"),
            attr(AttributeValue::Text("synthetic".to_string())),
        ),
        (
            AttrId::new("display_name").expect("valid attr id"),
            attr(AttributeValue::Number(42.0)),
        ),
        (
            AttrId::new("career_stage").expect("valid attr id"),
            attr(AttributeValue::Enum("outside".to_string())),
        ),
    ]);
    assert_eq!(
        report_codes(&validate_entity(&template, &entity)),
        vec![
            FindingCode::EnumValueNotInTemplate,
            FindingCode::AttrTypeMismatch,
            FindingCode::UnknownAttr,
        ]
    );

    let missing_required = empty_entity("person");
    assert_eq!(
        report_codes(&validate_entity(&template, &missing_required)),
        vec![FindingCode::RequiredAttrMissing]
    );
}

#[test]
fn instance_edge_validation_reports_expected_errors() {
    let (template, report) = parse_template(RESEARCH).expect("fixture parses");
    assert!(report.errors.is_empty());

    let mut forbidden_weight = edge("authored");
    forbidden_weight
        .set_weight(Some(1.0))
        .expect("finite edge weight");
    assert_eq!(
        report_codes(&validate_edge(
            &template,
            &forbidden_weight,
            &KindId::new("person").expect("valid kind id"),
            &KindId::new("publication").expect("valid kind id"),
        )),
        vec![FindingCode::WeightForbidden]
    );

    let mut required_template: Value = serde_json::from_str(RESEARCH).expect("fixture json");
    required_template["edge_kinds"][1]["weighted"] = json!("required");
    let json = serde_json::to_string(&required_template).expect("serialize template");
    let (required_template, report) = parse_template(&json).expect("template parses");
    assert!(report.errors.is_empty());
    assert_eq!(
        report_codes(&validate_edge(
            &required_template,
            &edge("authored"),
            &KindId::new("person").expect("valid kind id"),
            &KindId::new("publication").expect("valid kind id"),
        )),
        vec![FindingCode::WeightRequired]
    );

    assert_eq!(
        report_codes(&validate_edge(
            &template,
            &edge("authored"),
            &KindId::new("organization").expect("valid kind id"),
            &KindId::new("publication").expect("valid kind id"),
        )),
        vec![FindingCode::EndpointKindNotAllowed]
    );
}

#[test]
fn weighted_default_is_forbidden_when_absent() {
    let (template, report) = parse_template(FISHERIES).expect("fixture parses");
    assert!(report.errors.is_empty());

    let mut weighted_edge = edge("has_skill");
    weighted_edge
        .set_weight(Some(1.0))
        .expect("finite edge weight");
    assert_eq!(
        report_codes(&validate_edge(
            &template,
            &weighted_edge,
            &KindId::new("person").expect("valid kind id"),
            &KindId::new("skill_resource").expect("valid kind id"),
        )),
        vec![FindingCode::WeightForbidden]
    );
}

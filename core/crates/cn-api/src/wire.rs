use std::panic::{AssertUnwindSafe, catch_unwind};
use std::str::FromStr;

use cn_model::{EntityId, GroupId, accepts_schema};
use cn_perm::ViewerContext;
use cn_store::Operation;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use crate::error::{ApiError, ErrorCode, ErrorEnvelope};

pub(crate) fn respond<T, F>(f: F) -> String
where
    T: Serialize,
    F: FnOnce() -> Result<T, ApiError>,
{
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(value)) => serialize_envelope(json!({ "ok": value })),
        Ok(Err(err)) => serialize_envelope(json!({ "err": ErrorEnvelope::from(err) })),
        Err(_) => serialize_envelope(json!({
            "err": ErrorEnvelope::from(ApiError::internal("internal invariant violation"))
        })),
    }
}

pub(crate) fn parse_json<T: DeserializeOwned>(json: &str) -> Result<T, ApiError> {
    serde_json::from_str(json).map_err(|err| ApiError::invalid_json(err.to_string()))
}

pub(crate) fn parse_viewer(json: &str) -> Result<ViewerContext, ApiError> {
    serde_json::from_str(json).map_err(|err| ApiError::invalid_viewer(err.to_string()))
}

pub(crate) fn parse_group_id(value: &str) -> Result<GroupId, ApiError> {
    GroupId::from_str(value).map_err(|err| ApiError::invalid_json(err.to_string()))
}

pub(crate) fn parse_group_id_for_lookup(value: &str) -> Result<GroupId, ApiError> {
    GroupId::from_str(value).map_err(|_| ApiError::not_found())
}

pub(crate) fn parse_entity_id(value: &str) -> Result<EntityId, ApiError> {
    EntityId::from_str(value).map_err(|_| ApiError::not_found())
}

pub(crate) fn parse_ops_jsonl(chunk: &str) -> Result<Vec<Operation>, ApiError> {
    let mut ops = Vec::new();
    for (index, line) in chunk.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let op = serde_json::from_str(trimmed).map_err(|err| {
            ApiError::with_details(
                ErrorCode::InvalidJson,
                err.to_string(),
                json!({ "line": index.saturating_add(1) }),
            )
        })?;
        ops.push(op);
    }
    Ok(ops)
}

pub(crate) fn reject_template_report(report: &cn_schema::ValidationReport) -> Result<(), ApiError> {
    if report
        .errors
        .iter()
        .any(|finding| finding.code == cn_schema::FindingCode::UnsupportedSchemaVersion)
    {
        return Err(ApiError::with_details(
            ErrorCode::UnsupportedSchemaVersion,
            "unsupported template schema version",
            json!({ "report": report }),
        ));
    }
    if !report.errors.is_empty() {
        return Err(ApiError::with_details(
            ErrorCode::InvalidJson,
            "template validation failed",
            json!({ "report": report }),
        ));
    }
    Ok(())
}

pub(crate) fn reject_unsupported_ops(ops: &[Operation]) -> Result<(), ApiError> {
    if ops
        .iter()
        .any(|op| !accepts_schema(&op.schema_version) || !accepts_schema(&op.template_version))
    {
        return Err(ApiError::new(
            ErrorCode::UnsupportedSchemaVersion,
            "unsupported operation schema version",
        ));
    }
    Ok(())
}

fn serialize_envelope(value: Value) -> String {
    match serde_json::to_string(&value) {
        Ok(json) => json,
        Err(err) => format!(
            r#"{{"err":{{"code":"internal","message":"{}","details":{{}}}}}}"#,
            json_escape(&err.to_string())
        ),
    }
}

fn json_escape(value: &str) -> String {
    value
        .chars()
        .flat_map(|c| match c {
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

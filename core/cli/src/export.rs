//! `cn export`: adapter over the existing cn-api load + export-snapshot path.
//!
//! Loads a fixture ops log through `cn_api::Api` (the same begin/chunk/commit
//! sequence the app uses) and prints the export-snapshot envelope verbatim
//! for one viewer scope. The only work done here is argument marshalling;
//! every permission and export decision stays in the core (I2).

use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::Exit;

const USAGE: &str = "usage: cn export --template <template.json> --ops <ops.jsonl> \
--group <group-uuid> --viewer <anonymous|person:<person-uuid>>";

struct ExportArgs {
    template: String,
    ops: String,
    group: String,
    viewer_json: String,
}

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    let parsed = match parse_args(args) {
        Ok(parsed) => parsed,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            writeln!(err, "{USAGE}")?;
            return Ok(Exit::Usage);
        }
    };
    let template_json = match std::fs::read_to_string(&parsed.template) {
        Ok(json) => json,
        Err(io_err) => {
            writeln!(err, "error: cannot read '{}': {io_err}", parsed.template)?;
            return Ok(Exit::Failure);
        }
    };
    let ops_jsonl = match std::fs::read_to_string(&parsed.ops) {
        Ok(jsonl) => jsonl,
        Err(io_err) => {
            writeln!(err, "error: cannot read '{}': {io_err}", parsed.ops)?;
            return Ok(Exit::Failure);
        }
    };
    let now_ms = match unix_now_ms() {
        Ok(ms) => ms,
        Err(message) => {
            writeln!(err, "error: {message}")?;
            return Ok(Exit::Failure);
        }
    };

    let mut api = cn_api::Api::new();
    let begin = api.load_group_begin(&parsed.group, &parsed.viewer_json, &template_json);
    if let Some(exit) = check_step(err, "load_group_begin", &begin)? {
        return Ok(exit);
    }
    let chunk = api.load_ops_chunk(&parsed.group, &ops_jsonl);
    if let Some(exit) = check_step(err, "load_ops_chunk", &chunk)? {
        return Ok(exit);
    }
    let commit = api.load_group_commit(&parsed.group, now_ms);
    if let Some(exit) = check_step(err, "load_group_commit", &commit)? {
        return Ok(exit);
    }

    let envelope = api.export_snapshot(&parsed.group, &parsed.viewer_json, "{}");
    writeln!(out, "{envelope}")?;
    match serde_json::from_str::<serde_json::Value>(&envelope) {
        Ok(value) if value.get("err").is_none() => Ok(Exit::Ok),
        Ok(_) => {
            writeln!(err, "error: export_snapshot failed (envelope above)")?;
            Ok(Exit::Failure)
        }
        Err(parse_err) => {
            writeln!(
                err,
                "error: export_snapshot returned an unreadable envelope: {parse_err}"
            )?;
            Ok(Exit::Failure)
        }
    }
}

fn parse_args(args: &[String]) -> Result<ExportArgs, String> {
    let mut template = None;
    let mut ops = None;
    let mut group = None;
    let mut viewer = None;
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let slot = match flag.as_str() {
            "--template" => &mut template,
            "--ops" => &mut ops,
            "--group" => &mut group,
            "--viewer" => &mut viewer,
            other => return Err(format!("unknown export argument '{other}'")),
        };
        let value = iter.next().ok_or_else(|| format!("{flag} needs a value"))?;
        if slot.replace(value.clone()).is_some() {
            return Err(format!("{flag} given more than once"));
        }
    }
    Ok(ExportArgs {
        template: template.ok_or_else(|| "--template is required".to_string())?,
        ops: ops.ok_or_else(|| "--ops is required".to_string())?,
        group: group.ok_or_else(|| "--group is required".to_string())?,
        viewer_json: viewer_scope_json(&viewer.ok_or_else(|| "--viewer is required".to_string())?)?,
    })
}

/// Marshals the CLI viewer-scope argument into the existing ViewerContext
/// JSON shape. Person ids are passed through untouched; cn-api validates
/// them (no duplicated validation semantics here).
fn viewer_scope_json(scope: &str) -> Result<String, String> {
    if scope == "anonymous" {
        return Ok(json!({ "kind": "anonymous" }).to_string());
    }
    if let Some(person) = scope.strip_prefix("person:") {
        if person.is_empty() {
            return Err("viewer scope 'person:' needs a person uuid".to_string());
        }
        return Ok(json!({ "kind": "person", "person": person }).to_string());
    }
    Err(format!(
        "viewer scope '{scope}' is not 'anonymous' or 'person:<person-uuid>'"
    ))
}

/// Checks one load-step envelope; on failure writes the step name and the
/// full envelope to stderr and yields the failure exit (I3: the core error
/// is surfaced verbatim, never swallowed).
fn check_step(err: &mut dyn Write, step: &str, envelope: &str) -> std::io::Result<Option<Exit>> {
    match serde_json::from_str::<serde_json::Value>(envelope) {
        Ok(value) if value.get("err").is_none() => Ok(None),
        Ok(_) => {
            writeln!(err, "error: {step} failed: {envelope}")?;
            Ok(Some(Exit::Failure))
        }
        Err(parse_err) => {
            writeln!(
                err,
                "error: {step} returned an unreadable envelope ({parse_err}): {envelope}"
            )?;
            Ok(Some(Exit::Failure))
        }
    }
}

fn unix_now_ms() -> Result<i64, String> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|clock_err| format!("system clock is before the unix epoch: {clock_err}"))?;
    i64::try_from(elapsed.as_millis())
        .map_err(|range_err| format!("system time overflows the op clock: {range_err}"))
}

//! `cn validate`: adapter over the existing cn-schema template validation.
//!
//! Prints the machine-readable `ValidationReport` JSON exactly as cn-schema
//! serializes it (I12); this crate adds no validation semantics of its own.

use std::io::Write;

use crate::Exit;

const USAGE: &str = "usage: cn validate <template.json>";

pub(crate) fn run(
    args: &[String],
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> std::io::Result<Exit> {
    let [path] = args else {
        writeln!(err, "{USAGE}")?;
        return Ok(Exit::Usage);
    };
    let json = match std::fs::read_to_string(path) {
        Ok(json) => json,
        Err(io_err) => {
            writeln!(err, "error: cannot read '{path}': {io_err}")?;
            return Ok(Exit::Failure);
        }
    };
    let (_template, report) = match cn_schema::parse_template(&json) {
        Ok(parsed) => parsed,
        Err(parse_err) => {
            writeln!(err, "error: {parse_err}")?;
            return Ok(Exit::Failure);
        }
    };
    let rendered = match serde_json::to_string_pretty(&report) {
        Ok(rendered) => rendered,
        Err(ser_err) => {
            writeln!(err, "error: cannot serialize validation report: {ser_err}")?;
            return Ok(Exit::Failure);
        }
    };
    writeln!(out, "{rendered}")?;
    Ok(if report.errors.is_empty() {
        Exit::Ok
    } else {
        Exit::Failure
    })
}

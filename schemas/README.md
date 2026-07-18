# Persisted-format JSON Schemas

Versioned JSON Schemas for every persisted format (I7). Each schema carries an
explicit version in its `$id`, and every instance carries a `schema_version`
so readers can reject unknown majors loudly with a typed error (I3). While a
format's major is 0, the minor is the compatibility line, matching
`cn_model::accepts_schema`: 0.1-line readers accept `0.1.x` only. A breaking
change bumps the major and mints a new `$id`.

These schemas document the ratified formats as written and read by the Rust
core - the core remains the source of truth; the schemas are the
machine-checkable contract used by fixture validation and external tooling.

| Schema | $id | Documents |
|---|---|---|
| `group-template.schema.json` | `.../group-template/0.1.0` | Authored group-template documents (`fixtures/templates/*.template.json`), the R2 extensible-attribute contract parsed by `cn-schema`. |
| `op-log.schema.json` | `.../op-log/0.1.0` | One `cn-store` operation record (the line format of `fixtures/groups/*.ops.jsonl`) and the `cn-api` `export_snapshot` envelope. Root is a `oneOf` of the two; use `#/$defs/operation` or `#/$defs/export_snapshot` to target one shape. |
| `story-path.schema.json` | `.../story-path/0.1.0` | A curated story path (`cn_model::Story`), the R7 stories-are-data format. Referenced by `op-log.schema.json` for `StoryCreate`/`StoryUpdate` payloads. |
| `snapshot-envelope.schema.json` | `.../snapshot-envelope/0.1.0` | The data envelope embedded in the single-file offline snapshot (P2.3, D-044.5): explicit baked `viewer_scope` (anonymous or group-member only), the exact `viewer_context`, the embedded export, and resolved theme tokens. |
| `theme-tokens.schema.json` | `.../theme-tokens/0.1.0` | Resolved theme token output with its adjustment report. |

Cross-file `$ref`s (`op-log` -> `story-path`; `snapshot-envelope` -> `op-log`)
resolve by `$id`, so validators must register all schemas in this directory
before compiling - `app/scripts/validate-templates.mjs` (run as
`npm run validate:templates` from `app/`) does exactly that and validates the
fixture templates, every op-log fixture line, embedded stories, and embedded
authored templates against these schemas.

Planned, not yet authored: the intake contract schema (lands with Phase 5
P5.3) and Codex output contract schemas.

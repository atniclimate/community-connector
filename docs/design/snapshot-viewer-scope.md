# Snapshot Baked Viewer Scope (P2.5)

Status: DECIDED for the v0.1 line (Wave 2, Lane B2). Records which viewer's
permission-filtered projection is baked into the single-file offline snapshot
(R8), why, and the acceptance predicate that makes the choice testable
(D-044.6). Cross-refs: integration plan 2026-07-06 section 6.4,
schemas/snapshot-envelope.schema.json, ADR-001 (circles), ADR-002 A-B8 and
ADR-001 A-B2 (export gate), docs/design/snapshot-ledger.md (D-044.5).

## Decision

The baked viewer scope for v0.1 snapshot artifacts is **Anonymous**
(`viewer_context: { "kind": "anonymous" }`, `viewer_scope: "anonymous"` in the
envelope). The generator (`app/scripts/embed-snapshot-data.ts`) bakes only
this scope; it has no code path for any other viewer.

Scope is recorded per artifact in the envelope itself, and every artifact
carries exactly one fixture's projection (D-044.5):
`app/dist/snapshot.<fixture-id>.html`.

## Evidence and rationale

1. **Public-room safety (integration plan 6.4).** The shipped snapshot is the
   assembly-reveal artifact: it is shown on a projector and passed around as a
   file. Section 6.4 requires the baked projection be "the Anonymous/
   Group-member view, never the facilitator's or any Trusted/Private-bearing
   view" and that the scope be stated explicitly in the build. Anonymous is
   the narrower of the two admissible scopes: public-circle values only.

2. **Group-member scope is not safely generatable today (load-bearing).**
   ADR-001's circle semantics as implemented in cn-perm give every person
   viewer their *self* reach on records they own: `project()` resolves the
   admissible circle per owner (`cn-perm/src/projection.rs`,
   `admissible_circle` with the owner argument), so any `{ "kind": "person" }`
   bake embeds that person's own private- and trusted-circle values, and any
   trusted-peer grants extend reach further. There is no member-without-self
   `ViewerContext`. The fixture smoke viewer (`...3e9`) is additionally the
   GroupCreate actor and a governance member. Baking any concrete person
   therefore exceeds group-member reach, which the envelope schema's own
   description forbids the generator to do. Until cn-perm grows a first-class
   observer-member viewer (see Follow-ups), Anonymous is the only scope whose
   projection provably contains nothing above the audience's circle.

3. **Defense in depth from the export gate.** The envelope's export payload
   comes exclusively from `cn-api export_snapshot`, which applies the
   narrowing-only export gate (ADR-001 A-B2, ADR-002 A-B8): effective-tier T3
   values are excluded even for owners. Anonymous scope + export gate =
   public circle, sub-T3 only. This is the safest artifact that can exist.

4. **Empirical check (2026-07-17).** Against the pre-public-layer fixtures,
   the anonymous export of both fixture groups was empty (0 entities): every
   generated entity carried `presence_visibility: "group"`. This confirmed
   the scope machinery filters correctly, and that meaningful anonymous
   artifacts are a *fixture data* question, not a scope question (below).

## Consequences per fixture

- **research-network** carries public-record kinds (organization,
  publication, species - label attribute `default_visibility: "public"` in
  the template). `app/scripts/generate-demo-ops.mjs --public-layer` gives
  entities of exactly those kinds public presence and makes edges between two
  such entities public. Persons and every attribute-instance visibility are
  untouched. Result: the anonymous artifact projects a real constellation
  (60 entities, 13 edges as of this writing) while all person data stays at
  group circle and above.
- **fisheries-committee** is a deliberately closed group: no attribute in its
  template is public, by design. Its anonymous view is truthfully empty, so
  **no fisheries snapshot artifact exists in v0.1**. The embed step skips it
  loudly in auto mode and fails the build if it is requested explicitly. This
  is the correct privacy outcome, not a defect: a closed group has no public
  face. P4.4 ("both fixture groups load and render from the snapshot")
  therefore depends on one of the Follow-ups below.

## Handling rule

An anonymous-scope artifact is safe for any audience, including public rooms
and onward file sharing. If a future artifact is ever baked at group-member
scope, it must be handled as group-internal material - same handling as any
group-circle data. The envelope's `viewer_scope` field is the record of which
rule applies.

## Acceptance predicate (D-044.6 - for the browser-gate/acceptance lane)

Definitions, for a fixture ops log `F`, built artifact HTML `H`, and the
embedded envelope `E` (the `#cn-snapshot-envelope` JSON, whose export payload
was computed by cn-perm - the app and the test never re-derive permissions,
I2):

- `S_all`: every string that appears as an attribute *value* (including each
  element of tags values, link URLs, media refs), story title, or story
  narration in any op of `F`, with length >= 4.
- `S_pub`: every string value reachable anywhere in `E` (this includes the
  serialized template, so template vocabulary - kind labels, enum values,
  attribute ids - is never a false positive).
- `S_leak = S_all \ S_pub` (strings cn-perm chose not to export).

The artifact passes when ALL of the following hold:

1. `E` parses, validates against `schemas/snapshot-envelope.schema.json`, and
   has `viewer_scope == "anonymous"` with `viewer_context.kind == "anonymous"`.
2. `E.export.projection.entities.length > 0` (the empty-viewer tripwire).
3. **No member of `S_leak` occurs as a substring of `H`.** This is the "no
   attribute value above the baked scope appears anywhere in the serialized
   HTML" requirement, made independent of circle bookkeeping: anything the
   permission engine filtered must be absent, whatever the reason it was
   filtered (circle, tier, lifecycle).
4. `H` loads offline (file://, no dev server, no network) and renders a
   non-zero projected entity count - P2.4's browser test.

Known limits, stated so the test does not overclaim: the length-4 floor
excludes trivially colliding short strings (numbers, "2026"); a private value
that is *literally identical* to some public value is not detectable by
string difference - the real guarantee is that the payload is produced by
`cn-api export_snapshot` (cn-perm) for the anonymous viewer, and the string
sweep is a tripwire on top of it. A verification run of this predicate on
2026-07-17 found 259 value strings in the research-network ops, 130 present
in the envelope, 129 filtered, and 0 leaks in the artifact.

## Follow-ups (parked, with owners)

- **Observer-member viewer in cn-perm** (recommended path to a bakeable
  group-member scope and to a fisheries artifact for P4.4): a ViewerContext
  with group-circle reach and no self/trusted escalation. Permission-adjacent:
  grind HIGH plus a mandatory adversarial Codex round (CLAUDE.md), and an
  envelope-schema minor bump to admit the new context shape.
- **Alternative for fisheries only**: the data owner marks selected content
  public (a template/data decision, not a scope decision). For the real pilot
  this authority is ATNI Climate's (G1/D-030); for synthetic fixtures it is a
  fixture-design decision.

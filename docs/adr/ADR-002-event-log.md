# ADR-002: Event-Sourced Operation Log and Network Readiness

- Status: accepted (amended after adversarial rounds 1 and 2; round budget spent)
- Date: 2026-07-06
- Phase: 1
- Drivers: R5 (network-ready, not networked), R10/I6 (provenance), I3 (no
  silent failure), I7 (versioned formats), I12 (validation reports); inherits
  two hard requirements from ADR-001 round 1: operations idempotent by
  operation id, custody events with stable ids and an ordering rule.

## Context

Communities will eventually interlink over protocols that do not exist yet
(choosing them is a human gate). We must pick a persistence shape NOW that a
future sync layer can use WITHOUT rearchitecting, while serving a purely local
app today. The unit of exchange must also serve audit (who changed what, when,
under whose responsibility), which R10 demands anyway.

## Decision

### D1. Mutations are appended operations; state is a fold

Every domain mutation is an immutable `Operation` appended to a per-group log.
Current state = deterministic fold over the log. There is no in-place mutation
path in cn-store; cn-wasm and the CLI submit operations, never edited state.

### D2. Operation shape

```
Operation {
  op_id: OpId,                  // UUIDv7, assigned at creation, never reused
  group_id: GroupId,
  actor: ActorRef,              // human or software agent
  responsible_human: PersonId,  // required when actor is non-human (I6/R10)
  recorded_at: Timestamp,       // wall clock + monotonic counter (see D5)
  kind: OpKind,
  payload: kind-specific, schema-versioned
  schema_version: SemVer,       // of the op format itself (I7)
}
```

### D3. Operation granularity (from ADR-001 D4 and round-1 advisory)

OpKinds, initial set: `EntityCreate`, `EntityArchive`, `AttributeSet`,
`AttributeRemove`, `VisibilitySet`, `TierSet` (governance-gated),
`EdgeCreate`, `EdgeWeightSet`, `EdgeArchive`, `MembershipAdd`,
`MembershipRemove`, `TrustGrantCreate`, `TrustGrantRevoke`, `CustodyAppend`,
`StoryCreate`, `StoryUpdate`, `StoryArchive`, `TemplateMigrationApply`.
Attribute-level ops (not whole-entity snapshots) so a future sync exchanges
minimal, permission-classifiable units: an `AttributeSet` carries the
attribute's circle and tier, so the export/sync gate (ADR-001 A-B2) can filter
ops individually. Deletes are archives: the log never loses history; archived
objects drop out of the fold's live state.

### D4. Idempotency and replay determinism (inherited requirement)

The fold keeps a seen-set of op_ids: re-applying a duplicate op_id is a no-op
by construction. The fold is a pure function of the op MULTISET plus the order
rule in D5 - duplicated and out-of-order delivery converge to the same state.
Application errors (op references a missing entity, template violation) are
typed results recorded in the validation report (I12) and quarantine the op;
they never panic the fold and never get silently dropped (I3).

### D5. Total order, defined now, merged later

Local order is append order. For future multi-writer merge, every op carries
`recorded_at = (hlc: hybrid logical clock, actor_id, op_id)` and the canonical
order is lexicographic over that triple - total, deterministic, and computable
offline. Conflict SEMANTICS under merge: last-writer-wins per
(object, field) under the canonical order, with one reserved hook: OpKinds may
declare a custom merge in a future ADR (this is where CRDT semantics could
slot in per-field if ever needed). We deliberately do NOT design cross-group
identity, transport handshakes, or trust bootstrapping - human-gated network
decisions.

### D6. Custody events (inherited requirement)

`CustodyAppend { custody_event_id: UUIDv7, envelope_target, event }` - custody
events have their own stable ids; the custody vector's canonical order is the
op canonical order (D5), making append-only convergent under merge and
duplicate-safe (D4).

### D7. Storage layout (cn-store)

Per group: `ops.jsonl` (append-only, one op per line, fsync on append) plus
`snapshot.json` (a fold cache with the op_id watermark it folds up to,
schema-versioned). The log is ground truth; snapshots are derived, verifiable
(refold and compare), and deletable. Readers reject unknown MAJOR versions of
either format loudly (I7). Log lines never rewritten; compaction, if ever
needed, is a new ADR.

### D8. SyncTransport (cn-sync)

```
trait SyncTransport {
    fn peer_descriptor(&self) -> PeerDescriptor;      // opaque identity handle
    fn offer(&mut self, since: Watermark) -> Result<OpEnvelopeBatch, SyncError>;
    fn accept(&mut self, batch: OpEnvelopeBatch) -> Result<Ack, SyncError>;
}
```

Ships with exactly one implementation: `LocalLoopback` (moves op batches
between local stores; used by tests and the fixture round-trip). The trait is
the seam; nothing else in the codebase may reference a concrete transport.
`OpEnvelopeBatch` passes through the tier/export gate (ADR-001 A-B2) BEFORE
serialization - T3 ops are structurally unreachable by any transport.

### D9. What the fold feeds

The fold produces the raw in-memory state that ONLY cn-perm may read (ADR-001
A-B1). cn-graph and the app see projections. The wasm boundary (ADR-003)
exposes: submit_operation, get_projection(viewer), get_validation_report.

## Options considered and rejected

1. **CRDTs now** - rejected: pays replicated-datatype complexity before any
   network exists or requirements are known; D5's reserved per-OpKind merge
   hook keeps the door open at the point where CRDTs would actually plug in.
2. **Snapshot-only persistence (predecessor pattern)** - rejected: no unit of
   exchange for R5, no audit trail for R10, and merge becomes diff-guessing.
3. **Event-source everything including UI state** - rejected: scope creep;
   only domain mutations are ops. UI state machine (I4) is app-local.
4. **Global singleton log (not per-group)** - rejected: groups are the
   sovereignty and sync boundary (R1, tiers); a per-group log means a group's
   data can be exchanged, archived, or withheld as a unit.

## Consequences

- Positive: sync becomes "ship ops through a trait"; audit is structural;
  every state is reproducible; validation reports get exact op provenance.
- Negative: fold cost on load - mitigated by snapshots (D7); measured in
  Phase 2 alongside the ADR-001 memory gate.
- Negative: attribute-level ops make bulk imports chatty (thousands of ops) -
  accepted; cn-ingest batches appends and the log format is line-oriented
  precisely so bulk appends stream.

## Open questions (Phase 2 may refine without a new ADR)

- HLC implementation detail (single-node now; the type is opaque to callers).
- Snapshot cadence policy (every N ops vs on-close) - performance question,
  not architectural.

## Amendments (adversarial round 1, 2026-07-06)

Codex review verdict REDESIGN - direction affirmed, semantics incomplete.
Director rulings, amended in:

### A-B1. Lifecycle is a field; liveness is derived

Archive is not a special op class: every object carries a `lifecycle` field
(`active | archived`) mutated by `EntityArchive`/`EntityUnarchive` (added) /
`EdgeArchive`/`StoryArchive`, converging under the same LWW-per-(object,
field) rule as everything else. Ops on archived objects still fold (the record
keeps updating); LIVENESS is derived purely from the lifecycle field at
projection time. No dominance special-case, so convergence is preserved.

### A-B2. Cross-object validity is evaluated at read time

The fold is purely structural; cross-object SEMANTIC validity (a TrustGrant
whose grantor lost membership, an edge whose endpoint is archived) is
evaluated by cn-perm at projection time against current state. Such records
are INERT, not deleted - they reactivate if the referenced state returns.
No authorization decision is ever baked into fold order.

### A-B3. Ops validate against their declared template version

Every op carries the `template_version` it was authored against. The fold
validates an op against THAT version, then maps its effect forward through
the migration chain's directives (ADR-001 A-B6). A concurrent
`AttributeSet(old_name)` sorting after a rename migration remains valid per
its declared version and lands renamed. Ops declaring an unknown or future
template version are quarantined with a typed report entry.

### A-B4. Quarantine folds to fixpoint

The fold definition is: apply ops in canonical order, then re-examine the
quarantine set in canonical order, repeating until a pass admits nothing new
(fixpoint). Deterministic in the op multiset, so late-arriving dependencies
converge identically across peers. Quarantined ops persist in the log and in
the validation report (I12) until admitted.

Round-2 clarification: admission timing can never affect LWW outcomes. Every
field write carries its op's sort_key, and a write lands only if its sort_key
exceeds the sort_key currently recorded for that field - so an op admitted
late from quarantine with a lower sort_key cannot overwrite a higher-sort
write that already landed. Canonical order governs values; passes only govern
admission.

### A-B5. Export gate: per-OpKind disclosure and dependency closure

Every OpKind declares a disclosure classification covering ALL fields it can
reveal (existence, kind, endpoints, owner, roles, story membership, actor and
responsible_human metadata - advisory 4 folded in). An op exports to a
destination context only if every object it references admits that context
under ADR-001 rules. Exported batches must be dependency-closed AFTER
filtering (creates precede references); suppressed ops leave no placeholders,
gaps, or counts. The concrete per-OpKind table lands with cn-sync's
implementation spec, but the gate contract is normative now.

### A-B6. Write-path authorization is cn-perm's

cn-store exposes exactly one write path: `submit(ops)` which calls cn-perm
authorization per op BEFORE append. Unauthorized ops are rejected with typed
errors surfaced in the validation report (I2, I3). `VisibilitySet` requires
the value's owner. `TierSet` is authorized iff EITHER the submitter holds the
group's governance role (may assign any tier within community policy) OR the
submitter owns the value AND the new tier is strictly more restrictive than
the current effective tier (owner tighten-only, ADR-001 A-B3/D7). Any other
TierSet is rejected with a typed error.

### A-B7. Durability semantics, typed

One fsync per submitted BATCH, not per op (bulk ingest streams). Every line
ends with a newline; on open, a torn final line (missing newline or invalid
JSON) is truncated and reported as a typed WARN (I3, I12). The snapshot
watermark must be <= the durable log tip; a snapshot ahead of the log is
discarded with a typed WARN and state refolds from the log. Round-2 addition:
snapshot.json carries a content checksum, and a torn, unparseable, or
checksum-failing snapshot - regardless of watermark - is likewise discarded
with a typed WARN and a full refold (I3, I12). Log lines are never rewritten.

### A-B8. SyncTransport is an explicitly provisional v0 seam

The trait becomes a request/response exchange over versioned opaque frames
plus a capability descriptor:

```
trait SyncTransport {
    fn capabilities(&self) -> PeerCapabilities;   // versioned, extensible
    fn exchange(&mut self, frame: SyncFrame) -> Result<SyncFrame, SyncError>;
}
```

`SyncFrame` is a schema-versioned envelope (I7); the local loopback adapter
implements one frame kind (op-batch offer/accept by watermark). The
architectural guarantee R5 relies on is the SEAM (nothing outside cn-sync may
reference a transport or frame internals) plus op-format stability - NOT the
trait's arity, which the future protocol ADR (human-gated) is expected to
revise cheaply within the one crate. Per-tier partial sync is a declared
capability requirement for that ADR, wired to A-B5's gate.

### Advisories folded in

- The fold ALWAYS applies canonical order, including locally; append order is
  arrival detail, never fold order.
- Op shape splits `recorded_at: Timestamp` (display/audit) from
  `sort_key: (hlc, actor_id, op_id)` (canonical order).
- `CustodyAppend` on an archived target is valid (audit continues); on an
  unknown target it quarantines like any dependency miss.
- Version policy: readers reject unknown MAJOR versions loudly and accept
  unknown MINOR fields with ignore-and-preserve semantics (round-trip safe).

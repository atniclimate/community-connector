# ADR-002: Event-Sourced Operation Log and Network Readiness

- Status: draft - awaiting adversarial round 1
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

- HLC implementation detail (single-node now: wall clock + monotonic counter
  suffices; the type is opaque to callers).
- Snapshot cadence policy (every N ops vs on-close) - performance question,
  not architectural.

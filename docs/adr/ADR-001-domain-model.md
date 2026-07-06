# ADR-001: Domain Model

- Status: accepted (amended after adversarial round 1; see Amendments)
- Date: 2026-07-06
- Phase: 1
- Drivers: R1 (multi-group), R2 (template-defined kinds/attributes), R4
  (per-attribute sharing circles), R5 (network-ready ids and op exchange),
  R6 (graph queries), R10 (provenance + tier everywhere), invariants I2, I3,
  I6, I7, I12

## Context

cn-model is the single source of truth for what data IS. Every other crate
(schema validation, permissions, graph queries, storage, ingest, wasm boundary)
consumes these types. The predecessor hardcoded entity kinds, colors, and config
in code, silently swallowed provenance failures, and had no per-person sharing
control; those failures are structural motivations here, not just history.

## Decision

### D1. Identity

Every first-class object - Entity, Edge, Group, Membership, TrustGrant,
Operation, Story - gets a UUIDv7 id at creation. Ids are immutable, never
reused, and globally unique so future network exchange (R5) needs no renaming.

### D2. Kinds are data, not code

Entity kinds and edge kinds are template-defined identifiers validated by
cn-schema against the group's template. Rust represents a kind as a newtype
over an interned string (`KindId`), never an enum. A tribal fisheries committee
adds "fishing site" by editing JSON; the core never recompiles for vocabulary.

### D3. Entity shape

```
Entity {
  id: EntityId,                 // UUIDv7
  group_id: GroupId,
  kind: KindId,                 // must exist in the group template
  attributes: Map<AttrId, AttributeInstance>,
  owner: Option<PersonId>,      // personal-mode record ownership (R4)
  presence_visibility: Circle,  // can the viewer know this node exists
  provenance: ProvenanceEnvelope,  // required at construction (I6)
  tier: SensitivityTier,           // required at construction (I6)
  schema_version: SemVer,
}
```

There is no `Default` and no constructor that omits provenance or tier (I6).

### D4. Typed attributes: closed value types, open vocabulary

`AttributeValue` is a closed enum over the R2 type system: `Text`, `Number`,
`Enum(symbol)`, `Tags(set)`, `Date`, `Geo(point | region)`, `Link(url)`,
`MediaRef(opaque id)`. Adding a new value TYPE is a code change plus an ADR
amendment; which attributes exist, on which kinds, with what constraints, is
entirely template data. Validation failures are typed errors surfaced in the
validation report (I3, I12), never coerced.

Every attribute VALUE carries its own envelope:

```
AttributeInstance {
  value: AttributeValue,
  visibility: Circle,              // private|trusted|group|network|public
  provenance: ProvenanceEnvelope,  // self-reported vs ingested can differ per field
}
```

### D5. Circles, grants, and the core/template boundary

`Circle` is an ordered audience lattice: `Private < Trusted < Group < Network
< Public`. `Trusted` resolves through explicit `TrustGrant` records (grantor
person, grantee person, optional scope, timestamps, revocation) that are
owner-managed and audit-logged.

The permission-relevant relations - Membership (person in group with role),
ownership, TrustGrant - are CORE primitives with fixed shapes, deliberately
not template-definable. Everything the community sees as "the graph"
(collaboration, stewardship, attendance, ...) is template-defined edges.
Templates cannot redefine or shadow the primitives permission evaluation
depends on.

### D6. Edges

```
Edge {
  id, group_id, kind: KindId,
  from: EntityId, to: EntityId, directed: bool,
  weight: Option<f64>,
  attributes: Map<AttrId, AttributeInstance>,   // same envelope as entities
  visibility: Circle,
  provenance: ProvenanceEnvelope, tier: SensitivityTier,
  schema_version,
}
```

Visibility rule (evaluated ONLY in cn-perm, I2; cn-model just stores fields):
an edge is projected iff both endpoints' presence is visible to the viewer,
the edge's own circle admits the viewer, and its tier admits the projection
context.

### D7. Tier x circle interaction

Tier (T0 open .. T3 sovereign-restricted) is community-governance
classification; circle is individual sharing preference. They are independent
axes and the most restrictive always wins: effective disclosure =
min(tier ceiling for the context, circle for the viewer). Tiers additionally
gate exports and future sync (a T3 value never leaves the local store
regardless of circle). Tier assignment authority belongs to community
governance, not to the individual attribute owner; the owner may only tighten.

### D8. Provenance envelope

```
ProvenanceEnvelope {
  origin: Origin,             // SelfReported | Ingested{source} | Derived{inputs} | Authored
  recorded_by: ActorRef,      // human or software agent
  responsible_human: PersonId,   // REQUIRED when recorded_by is non-human
  recorded_at: Timestamp,
  custody: Vec<CustodyEvent>, // append-only; mutation of past events is a type-system impossibility (private field, push-only API)
  schema_version,
}
```

Stamping failures are typed errors and propagate (I3). No silent defaults.

### D9. Groups

`Group { id, name, template_id, template_version, settings, provenance, tier }`.
A group pins the template VERSION it was created against; template upgrades are
explicit migrations validated by cn-schema (I7).

## Options considered and rejected

1. **Enum-per-kind entity model** - rejected: violates R2; every new community
   type needs a recompile.
2. **Fully open attribute values (arbitrary JSON)** - rejected: nothing to
   validate against, permission projection cannot reason about unknown shapes,
   I12 becomes unenforceable.
3. **Entity-level visibility only** - rejected: R4 requires per-attribute
   grants ("share my phone with trusted, my skills with the network").
4. **Single sensitivity axis (merge tier and circle)** - rejected: conflates
   community sovereignty with personal preference; R4 and R10 demand both,
   with different assignment authority.
5. **RDF/triple store** - rejected for now: maximal flexibility but poor
   ergonomics for the typed traversals and viz projections R6 needs; the typed
   property graph is the demonstrated fit. Revisit only via a superseding ADR.
6. **Membership as a template edge** - rejected: permission evaluation cannot
   depend on community-editable vocabulary (a template edit must never be able
   to break or widen access).

## Consequences

- Positive: new community type = one JSON file; permission logic gets stable
  core primitives; every value can justify itself in any projection, export,
  or audit; ingest and UI share one validation path.
- Negative: per-value envelopes cost memory (envelope + map overhead per
  attribute). At the 2-5k node / 10k edge target this is well within budget;
  measure in Phase 2 and intern/share envelopes if profiling demands.
- Negative: interned KindIds require a per-group registry and careful template
  version migration (I7); accepted as the price of R2.

## Open questions (resolve before Phase 1 exit)

- Geo values: point + named-region reference now; full geometry later?
- MediaRef resolution/storage is deliberately opaque until Phase 4 (ingest).

## Amendments (adversarial round 1, 2026-07-06)

Codex review session 019f36fa-a523-7ee3-8e3a-fdb3d9afced4 (verdict
ACCEPT-WITH-AMENDMENTS; full critique
archived in DECISIONS.md D-010 context). Amendments accepted by the director:

### A-B1. Query closure over projections

ALL graph operations - shortest/constrained paths, degree counts, neighborhood
expansion, need-to-solution routing, search, story rendering - execute over the
viewer's materialized projection, never the raw graph. cn-graph's public API
takes a `Projection` (produced only by cn-perm) as input; there is no cn-graph
entry point accepting the raw store. A path that would traverse an invisible
entity does not exist in the projection, so no result, count, or "route exists"
signal can leak it.

### A-B2. Tier ceilings are circles - one lattice

Each tier maps to a ceiling ON THE CIRCLE LATTICE per viewer context:
T0 -> Public, T1 -> Network, T2 -> Group, T3 -> Private (owner/custodian only;
custodian access defined in Phase 5, never export, never sync). Effective
visibility of any value = min(value's circle, tier ceiling). Both operands are
circles, so min is total and every (tier, circle, context) cell is defined:
e.g. a T3 attribute marked Public resolves to Private.

### A-B3. Attribute-level tier override (tighten-only)

`AttributeInstance` gains `tier: Option<SensitivityTier>`. Effective tier of a
value = max-restrictiveness(entity tier, override); overrides can only tighten,
never loosen (most restrictive wins). This expresses the fisheries case: a T1
fishing site with group-visible display name and T3 site_knowledge.

### A-B4. Story visibility

A story carries its own visibility circle and tier, gating the WHOLE story per
viewer via the same rules as entities. Within a visible story, steps whose
entities/edges are not in the viewer's projection are elided silently - no
markers, counts, or ordering gaps that signal hidden content. Authoring-time
validation (R7) runs against the author's own projection, not the raw graph.

### A-B5. Canonical attribute type ids

Wire ids in templates: `text`, `number`, `enum`, `tags`, `date`, `geo`,
`link`, `media`. `media` corresponds to `AttributeValue::MediaRef`. The
fixture templates are conformant as written.

### A-B6. Template migration with live data

A template version that removes or narrows a kind/attribute still referenced by
live data MUST ship an explicit migration directive (`map_to`, `archive`, or
`delete_with_consent`); absent that, migration fails loudly (I3, I7). Retired
kind ids are tombstoned in the group registry forever and never reused.

### Advisories folded in

- Weights: each edge kind declares `weighted: required | optional | forbidden`
  (template default forbidden); violations are validation errors (I12).
- Handed to ADR-002 as hard requirements: operations are idempotent by
  operation id (UUIDv7), deterministic under duplicate and out-of-order
  delivery; custody events carry stable ids and an ordering rule.
- Memory: Phase 2 exit gate includes a measured heap figure for 5k entities x
  10 attributes + 10k edges; target core data under 64MB in wasm; envelope
  interning is the pre-approved mitigation.
- `vocabulary`, `shape`, `color_role`, `theme` in templates are presentation
  metadata: structurally validated by cn-schema, semantically ignored by
  cn-model.
- Email links are stored as `mailto:` URLs; `format: email` is a validation
  hint on `link`.

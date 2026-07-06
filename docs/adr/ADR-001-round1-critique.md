## Blocking

1. Permission-filtered projection is not closed over queries. Concrete failure: a path query from visible need A to visible person B could traverse invisible fishing_site X, then return a shorter path, a count, or "route exists" result that reveals X or its hidden edges. ADR-001 defines edge projection, but not degree counts, shortest paths, neighborhood expansion, need-routing, or stories as operations over the already-filtered projection.

2. The tier x circle rule is undefined. `effective disclosure = min(tier ceiling for the context, circle for the viewer)` compares different axes without a complete mapping. Undefined cells include public-circle plus T2/T3 for anonymous, network, group member, trusted peer, self, and admin contexts. A T3 attribute marked public has no deterministic answer except "most restrictive wins", but the ADR does not define what context can ever see it.

3. Attribute tiering contradicts the ADR text. `AttributeInstance` has visibility and provenance but no tier, while D7 says "a T3 value never leaves the local store" and "owner may only tighten." Concrete failure: a fisheries `fishing_site` may have a group-visible display name and sovereign-restricted site knowledge. Entity-level tier forces either over-restriction of the display name or leakage of the site knowledge.

4. The ADR does not specify permission-safe story validation or rendering. Concrete failure: a story path references a hidden person or hidden edge. If validation errors, missing-step indicators, path length, or narration order are returned to a viewer, the story leaks existence even when graph projection hides the entity.

5. Fixture media attributes cannot be represented literally. The fixtures use `"type": "media"` for `portrait`, `family_canoe_photo`, and `flyer`; ADR-001 defines `MediaRef(opaque id)` and names R2 as "media ref." Without an explicit schema alias, a conforming implementation rejects these templates.

6. Template migration with removed live kinds is not defined. Concrete failure: fisheries v0.2 removes `fishing_site` while live entities and `stewards` edges still use it. D9 says upgrades are explicit migrations, but does not say whether the migration must fail, tombstone the kind, map it, archive entities, or keep old registry entries. This can orphan live data or make validation dependent on whichever template registry is loaded.

## Advisory

1. Weighted edge kinds need validation rules. The model has `weight: Option<f64>`, and fixtures mark `coauthored` and `need_met_by` as weighted. The ADR should say whether weights are required, optional, forbidden on unweighted kinds, normalized, or constrained by template data.

2. R5 folding needs operation granularity. D4 forces at least entity create, edge create, attribute set/remove, visibility set, edge weight set, and custody append operations. ADR-001 should hand ADR-002 a clear requirement that these ops are idempotent by operation id and deterministic under duplicate or out-of-order delivery.

3. Custody append-only semantics are underspecified for sync. A private `Vec` plus push-only API protects local mutation, but duplicate or out-of-order custody append ops can still produce divergent vectors unless custody events have stable ids and ordering rules.

4. The memory claim is too casual. At 50k entity attributes, per-attribute maps plus provenance envelopes plausibly land in the tens of MB before renderer data, edge maps, strings, indexes, and wasm allocator overhead. This may still be acceptable, but "well within budget" needs a Phase 2 measurement target and an envelope interning plan.

5. Template UI vocabulary is not clearly in or out of the domain model. The fixtures include `vocabulary`, `shape`, `color_role`, and `theme`. If cn-model ignores them, say they are schema/UI metadata. If cn-schema validates them, say they are persisted template fields with schema-versioned compatibility.

6. Email-as-link needs a stated encoding. The fixtures use `type: link` with `format: email`; ADR-001 says `Link(url)`. The model should define whether email links are `mailto:` URLs or a separate constrained link format.

## Verdict

ACCEPT-WITH-AMENDMENTS
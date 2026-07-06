## Blocking

1. `EntityArchive` vs concurrent `AttributeSet` is underspecified. Scenario: op A archives entity E, op B sets E.phone with a later canonical `(hlc, actor_id, op_id)`. LWW per `(object, field)` can make the attribute live while the entity is archived, or preserve a mutated tombstone, depending on whether archive is a field, object lifecycle state, or dominance rule. ADR-002 must define lifecycle dominance for archive, post-archive ops, and whether unarchive exists.

2. `MembershipRemove` vs concurrent `TrustGrantCreate` has permission consequences outside LWW fields. Scenario: person P is removed from group G while P concurrently grants trusted visibility to Q for a group-scoped record. Does the grant survive, become invalid, or become inert until membership returns? Without a cross-object invariant, the fold can converge structurally while producing an authorization hole.

3. `TemplateMigrationApply` vs concurrent `AttributeSet` can quarantine valid intent. Scenario: migration M renames `site_name` to `display_name`; concurrent `AttributeSet(site_name)` sorts after M. The old attr no longer validates and is quarantined, while the same op sorting before M would migrate cleanly. The ADR needs migration-era compatibility rules, not only canonical order.

4. Quarantine recheck is undefined and can break eventual convergence. Scenario: `EdgeCreate(E1 -> E2)` arrives first and is quarantined because E2 is missing. Later `EntityCreate(E2)` arrives. If quarantine is not re-examined after every dependency-changing op or full refold, peers with different arrival orders can produce different live states from the same op set.

5. The export gate is only precise for `AttributeSet`. `EntityCreate`, `EdgeCreate`, `StoryCreate`, `MembershipAdd`, and `TrustGrantCreate` can leak existence, kind, endpoints, owner, roles, or narrative membership before any attribute filter runs. ADR-002 says batches pass through a tier/export gate, but does not specify per-OpKind redaction, suppression, or dependency closure.

6. `VisibilitySet` and `TierSet` authority is silent. ADR-001 says tier authority belongs to community governance and permission logic lives only in `cn-perm`. ADR-002 needs to state that op submission authorization is delegated to `cn-perm` and recorded as a typed rejection, otherwise cn-store becomes a write-path permission bypass in violation of I2.

7. Storage failure modes are not sufficiently typed. `ops.jsonl` with fsync per append on Windows can make bulk import impractical, contradicting the accepted chatty ingest consequence unless batch durability semantics are defined. Also, partial JSONL line writes, torn snapshot writes, and snapshot watermark ahead of durable log need explicit detection and typed recovery reports under I3 and I12.

8. `SyncTransport` shape is too narrow for the stated future protocol seam. A single scalar `Watermark` plus `offer(since)` assumes ordered complete logs. It does not model anti-entropy by op id set, partial sync by tier or projection, dependency negotiation, resumable chunks, or peer capability exchange. That risks foreclosing R5 rather than preserving it.

## Advisory

1. D5 says local order is append order, but future canonical order is lexicographic HLC. The ADR should state whether local single-writer replay also sorts canonically before fold, otherwise local state can differ from later merged state.

2. `recorded_at` is described as wall clock plus monotonic counter and also as `(hlc, actor_id, op_id)`. Split timestamp from canonical sort key to avoid schema ambiguity.

3. `CustodyAppend` orders custody events by op order, but does not say whether custody append to an archived or quarantined target is valid.

4. `responsible_human: PersonId` may itself be sensitive. Export rules need to cover actor and responsible-human metadata, not only payloads.

5. Unknown major rejection is good, but minor-version forward compatibility rules are absent for op payloads and snapshots.

## Verdict

REDESIGN. The append-only log direction is sound, but ADR-002 currently lacks the lifecycle, dependency, authorization, export, storage recovery, and sync capability semantics needed before implementation.
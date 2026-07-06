## Resolved

A-B1: yes - lifecycle as an LWW field makes archive, unarchive, and concurrent updates converge.

A-B2: yes - cross-object validity is read-time in `cn-perm`; invalid grants become inert, not fold-order authorization facts.

A-B3: yes - validating against the op's declared template version, then migrating forward, resolves rename races under canonical order.

A-B4: no - fixpoint order is deterministic, but it does not say admitted ops replay at their canonical position or compare by `sort_key`. A low-sort quarantined `AttributeSet` can be admitted after a higher-sort write to the same field and overwrite LWW.

A-B5: yes - per-OpKind disclosure classification plus dependency closure resolves the export leak objection.

A-B6: yes for delegating write authorization to `cn-perm`, but see new blocking.

A-B7: no - batch fsync and torn final JSONL lines are covered, but a torn or invalid `snapshot.json` with watermark `<=` durable tip still lacks typed recovery/reporting.

A-B8: yes - versioned frames, capabilities, and confining frame internals to `cn-sync` preserve the provisional seam.

## New blocking

1. `TierSet` authorization contradicts the permission spec for owner tightening. Scenario: a non-governance owner tightens an attribute from T1 to T3. ADR-001 and the permission spec allow owners to tighten, but A-B6 says `TierSet` requires the group's governance role, making the owner path either rejected or ambiguous. The predicate must be explicit: governance can assign within policy; owners can only tighten values they own.

2. A-B4 introduces a canonical-order contradiction unless clarified. Scenario: `AttributeSet(E.name = old)` has sort key 1 and quarantines because `E` is missing; `EntityCreate(E)` has sort key 2; `AttributeSet(E.name = new)` has sort key 3. The fixpoint pass may admit key 1 after key 3. If application mutates then, final value is `old`, violating D5 LWW under canonical order.

## Verdict

ACCEPT-WITH-AMENDMENTS.
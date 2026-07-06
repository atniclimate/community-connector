# Permission Model Spec

This document consolidates the existing permission decisions for the repo. It does not introduce new policy.

## 1. Scope

- `cn-perm` is the only place permission logic lives.
- The app never receives an unfiltered API.
- Every graph the app sees is a projection for a specific viewer context.
- `cn-graph` and the rest of the app consume only the projection output, not the raw store.
- Permission behavior is therefore a read-time projection concern, not an app-layer concern.

## 2. The Circle Lattice

The audience lattice is:

`Private < Trusted < Group < Network < Public`

What each admits:

- `Private`: only the owner or custodian path when Phase 5 defines that access.
- `Trusted`: viewers with an explicit `TrustGrant` between grantor and grantee.
- `Group`: viewers with group membership access.
- `Network`: viewers in the broader network audience.
- `Public`: everyone, including anonymous viewers.

`TrustGrant` records are core primitives with this shape from ADR-001 D5:

- grantor
- grantee
- optional scope
- timestamps
- revocation state
- audit logging

## 3. Viewer Contexts

Viewer contexts are:

- anonymous
- group member
- trusted peer
- self
- admin

Per group, each viewer context resolves to a circle bound through the projection rules:

- anonymous resolves to `Public` only.
- group member resolves to at most `Group`.
- trusted peer resolves to at most `Trusted` for values covered by a valid `TrustGrant`.
- self resolves to the least restrictive view available to the record owner, subject to tier ceilings and per-value circles.
- admin resolves according to the same lattice as every other viewer.

Admin is not exempt from tier ceilings.

RESOLVED (director, 2026-07-06): admin has NO operational override outside the
projection path. Administrative and governance actions are ordinary operations
(ADR-002) subject to cn-perm authorization at submit time; no read, export, or
maintenance path bypasses projection. If a future recovery scenario appears to
need one, that is a new ADR plus a human gate, never an exception here.

## 4. Tier Ceilings

The A-B2 tier ceiling table is:

- `T0 -> Public`
- `T1 -> Network`
- `T2 -> Group`
- `T3 -> Private` and custodian-only

Tiered values are never exported or synced if they resolve to `T3`.

Tier assignment authority belongs to community governance.

Owners may only tighten visibility. They may not loosen a tier.

Attribute-level tier overrides exist as `AttributeInstance.tier` and can only tighten the effective tier.

## 5. Effective Visibility Algorithm

Effective disclosure is computed on the circle lattice.

```text
effective_tier = most_restrictive(entity.tier, attribute.tier_override)
ceiling = circle_ceiling(effective_tier)
visible = viewer_circle >= min(value.visibility, ceiling)
```

Rules by object type:

- Entity presence is visible only if the entity presence circle admits the viewer.
- Attribute values are visible only if the attribute circle and the tier ceiling both admit the viewer.
- Edges are visible only when both endpoints are visible, the edge circle admits the viewer, and the edge tier admits the projection context.
- Stories are gated at the story level first. If the story is visible, any invisible step is silently elided.

For stories, silent step elision means no markers, no counts, and no ordering gaps that reveal hidden content.

## 6. Projection Construction

Inputs:

- raw store
- viewer context

Output:

- a `Projection` consumed by `cn-graph`

Construction rules:

- `cn-perm` materializes the projection.
- `cn-graph` never touches the raw store.
- projection construction must be deterministic for the same input state and viewer context.
- projections must not leak hidden data through counts, paths, or ordering.

This includes indirect leaks from absent nodes, route existence, degree counts, neighborhood expansion, and story ordering.

## 7. Audit

Grant changes and tier changes are op-log events.

They are tracked as part of the append-only audit trail and point to ADR-002, which is still in draft.

OPEN QUESTION (Phase 1): the final op-log event names and field shapes are not defined in the current source set.

## 8. Test Obligations for Phase 2

Required tests:

- property test: no projection ever contains an attribute above the viewer's access.
- unit cases for circle admission across private, trusted, group, network, and public values.
- unit cases for entity presence, attribute values, edges, and stories.
- unit cases for admin respecting tier ceilings.
- unit cases for tighten-only tier overrides.
- unit cases for silent story step elision.
- unit cases for determinism and non-leakage through counts, paths, and ordering.
- one fisheries fixture scenario: a `T1` `fishing_site` with `T3` `site_knowledge` override and a trusted-only location.

The fisheries scenario must prove that the site can remain visible while the trusted-only location and private knowledge stay constrained by the projection rules.

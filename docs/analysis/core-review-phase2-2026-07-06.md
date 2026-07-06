## Blocking

1. `core/crates/cn-api/src/lib.rs:524-593` - export disclosure rules live in `cn-api`, not `cn-perm`. The T3 export gate directly reads raw tiers and filters entities, attributes, and edges outside the permission crate, violating I2.

2. `core/crates/cn-store/src/authz.rs:38-44` and `core/crates/cn-store/src/fold.rs:152-154` - `submit` misclassifies `MissingTarget` ops. `GroupState::apply` stores them in private quarantine without adding a report entry, so `submit` returns `Applied` for a quarantined op and the viewer sees no quarantine report. This violates I3/I12 and the `SubmitOutcome::Quarantined` contract.

3. `core/crates/cn-api/src/lib.rs:454-456` - submit outcomes are returned unredacted. A hidden existing entity can produce `not_owner_or_governance` while an absent id produces `target_missing`, making hidden distinguishable from absent at the API boundary, contrary to ADR-003 A-B4.

4. `core/crates/cn-perm/src/reports.rs:69-107` - report redaction is substring-based over visible ids, kinds, and attr names, then clones matching entries. A hidden finding containing a visible kind or attribute string is disclosed even when its subject object is not projected, violating ADR-003 round-2 report redaction.

5. `core/crates/cn-store/src/log.rs:213-235` and `core/crates/cn-store/src/fold.rs:120-141` - snapshot load reconstructs `GroupState` with empty `field_clocks` and `seen`. Subsequent lower-sort writes can overwrite snapshotted fields, and duplicate op ids from before the snapshot are no longer deduped, violating ADR-002 per-field LWW and D4 idempotency.

6. `core/crates/cn-sync/src/lib.rs:1-4` - `cn-sync` is only a placeholder. ADR-002 D8/A-B8 require a versioned `SyncTransport` seam and local loopback adapter with frame/capability boundaries; none exist.

## Advisory

1. `core/crates/cn-store/src/log.rs:273-279` - `OpLog::open` deserializes operations without rejecting unsupported `schema_version` or `template_version`. The API rejects unsupported ops later, but the persisted log reader itself does not loudly reject unknown majors as required by I7/ADR-002 D7.

2. `core/crates/cn-store/src/log.rs:91-123` - snapshot parse, checksum, and watermark discard paths return `Ok(None)` with no `StoreReport` warning channel. This matches the known debt called out in the task, and remains unresolved.

3. `core/crates/cn-store/src/op.rs:29-37` - `HlcClock::tick` uses `saturating_add` for the counter. At `u32::MAX`, repeated stalled/regressing ticks stop being strictly increasing, violating the HLC contract.

4. `core/crates/cn-api/src/lib.rs:1-628` - module exceeds 500 lines without a documented I5 exception. `cn-store/src/fold.rs` has such a note; `cn-api` does not.

## Verdict

FIXES-REQUIRED
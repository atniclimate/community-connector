/**
 * P3.5 wizard pure logic: dashboard summarization (the I12 visibility
 * surface) and review-request assembly for the read-only core calls.
 * No I/O, no DOM - testable in the node environment.
 */

import type { IntakeRecordSummaryDto, JsonObject, JsonValue } from "../../state/state";
import { formModel } from "../forms/model";

export type QueueDashboard = {
  readonly countsByState: Readonly<Record<string, number>>;
  /** Milliseconds since the oldest still-pending record was staged. */
  readonly oldestPendingAgeMs: number | null;
  /** Records with unreadable sidecars (investigation items). */
  readonly unreadable: number;
  readonly pendingDecisionFiles: number;
};

export function dashboard(
  records: readonly IntakeRecordSummaryDto[],
  pendingDecisionFiles: number,
  nowMs: number,
): QueueDashboard {
  const countsByState: Record<string, number> = {};
  let oldestPendingStagedAt: number | null = null;
  let unreadable = 0;
  for (const record of records) {
    if (record.reviewState === null) {
      unreadable += 1;
      continue;
    }
    countsByState[record.reviewState] = (countsByState[record.reviewState] ?? 0) + 1;
    if (
      record.reviewState === "pending" &&
      (oldestPendingStagedAt === null || record.stagedAtMs < oldestPendingStagedAt)
    ) {
      oldestPendingStagedAt = record.stagedAtMs;
    }
  }
  return {
    countsByState,
    oldestPendingAgeMs:
      oldestPendingStagedAt === null ? null : Math.max(0, nowMs - oldestPendingStagedAt),
    unreadable,
    pendingDecisionFiles,
  };
}

/**
 * Template-driven attribute-id sets for near-duplicate scoring: name
 * attrs are the payload kind's REQUIRED text attributes (the pilot
 * form's display_name); affiliation attrs are its tags attributes. A
 * deterministic heuristic - templates carry no "name role" concept, and
 * the scorer itself stays conservative regardless.
 */
export function nearDupAttrSets(
  template: JsonObject,
  kindId: string,
): { readonly nameAttrs: readonly string[]; readonly affiliationAttrs: readonly string[] } {
  const kind = formModel(template).kinds.find((candidate) => candidate.id === kindId);
  if (kind === undefined) {
    return { nameAttrs: [], affiliationAttrs: [] };
  }
  return {
    nameAttrs: kind.attributes
      .filter((attr) => attr.attrType === "text" && attr.required)
      .map((attr) => attr.id),
    affiliationAttrs: kind.attributes
      .filter((attr) => attr.attrType === "tags")
      .map((attr) => attr.id),
  };
}

function payloadKind(record: IntakeRecordSummaryDto): string {
  const kind = record.payload["kind"];
  return typeof kind === "string" ? kind : "person";
}

/** The intake_near_duplicates request for one record under review:
 * OTHER queue payloads only (never itself), template-driven attr sets. */
export function nearDupRequest(
  template: JsonObject,
  record: IntakeRecordSummaryDto,
  allRecords: readonly IntakeRecordSummaryDto[],
): JsonObject {
  const sets = nearDupAttrSets(template, payloadKind(record));
  const queuePayloads: JsonValue[] = allRecords
    .filter((other) => other.recordId !== record.recordId)
    .map((other) => [other.recordId, other.payload]);
  return {
    payload: record.payload,
    queue_payloads: queuePayloads,
    name_attrs: [...sets.nameAttrs],
    affiliation_attrs: [...sets.affiliationAttrs],
  };
}

/** The intake_dedup_check existing-keys list: every OTHER record's
 * semantic key (transport keys exist only on remote records, which the
 * puller stages - the in-app wizard compares semantically). */
export function dedupKeys(
  record: IntakeRecordSummaryDto,
  allRecords: readonly IntakeRecordSummaryDto[],
): readonly JsonObject[] {
  return allRecords
    .filter((other) => other.recordId !== record.recordId)
    .map((other) => ({
      submission_id:
        typeof other.payload["submission_id"] === "string"
          ? other.payload["submission_id"]
          : "",
      payload_hash: other.payloadHash,
    }));
}

/** Records the wizard can stage decisions for (blueprint section 5: it
 * refuses approved_intent and unreadable sidecars - native recovery owns
 * those). */
export function reviewable(record: IntakeRecordSummaryDto): boolean {
  return record.reviewState !== null && record.reviewState !== "approved_intent";
}

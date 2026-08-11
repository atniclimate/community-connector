/**
 * Full-state round-trip integration test (D-080 debt 2): grant -> stage
 * submission -> scan -> decide -> verify the complete state at each step.
 * This is the app-side counterpart of the Rust approve_decide_apply_reload
 * round trip; it exercises the production state flow end-to-end through
 * the effects layer, reducer, and store contract.
 */

import { describe, expect, it } from "vitest";

import type { Action } from "./actions";
import type {
  IntakeBuilderClient,
  IntakeDirHandle,
  IntakeFileHandle,
} from "./intake";
import {
  clearQueueDirectory,
  grantQueueDirectory,
  scanQueue,
  stageDecision,
  stageSubmission,
} from "./intake";
import { createInitialState } from "./state";
import type { AppState, JsonObject } from "./state";
import { reduce } from "./reducer";

class FakeDir implements IntakeDirHandle {
  public readonly files = new Map<string, string>();
  public readonly dirs = new Map<string, FakeDir>();
  constructor(public readonly name: string) {}
  getFileHandle(
    name: string,
    options?: { readonly create?: boolean },
  ): Promise<IntakeFileHandle> {
    if (!this.files.has(name)) {
      if (options?.create !== true) {
        return Promise.reject(new Error(`NotFound: ${name}`));
      }
      this.files.set(name, "");
    }
    const files = this.files;
    return Promise.resolve({
      getFile: () => Promise.resolve({ text: () => Promise.resolve(files.get(name) ?? "") }),
      createWritable: () => {
        let buffer = "";
        return Promise.resolve({
          write: (data: string) => { buffer += data; return Promise.resolve(); },
          close: () => { files.set(name, buffer); return Promise.resolve(); },
        });
      },
    });
  }
  getDirectoryHandle(
    name: string,
    options?: { readonly create?: boolean },
  ): Promise<IntakeDirHandle> {
    const existing = this.dirs.get(name);
    if (existing) return Promise.resolve(existing);
    if (options?.create !== true) return Promise.reject(new Error(`NotFound: ${name}`));
    const created = new FakeDir(name);
    this.dirs.set(name, created);
    return Promise.resolve(created);
  }
  async *keys(): AsyncIterable<string> {
    for (const name of [...this.files.keys(), ...this.dirs.keys()].sort()) {
      yield name;
    }
  }
}

let recordCounter = 0;

function fakeClient(): IntakeBuilderClient {
  return {
    intakeStageRecord: (_payloadJson: string, stagedAtMs: number) => {
      const id = `rec-${++recordCounter}`;
      return Promise.resolve({
        record: {
          queue_record_version: "0.1.0",
          record_id: id,
          staged_at: stagedAtMs,
          source: { kind: "in_app" },
          payload: JSON.parse(_payloadJson),
          payload_hash: `ph-${id}`,
          record_checksum: `rc-${id}`,
        },
        sidecar: {
          queue_record_version: "0.1.0",
          record_id: id,
          payload_digest: `rc-${id}`,
          review_state: "pending",
          sidecar_revision: 0,
          decision_generation: 0,
          history: [],
          plan: null,
          validation_report: null,
          sidecar_checksum: `sc-${id}`,
        },
      });
    },
    intakeBuildDecision: (requestJson: string) => {
      const request = JSON.parse(requestJson) as JsonObject;
      return Promise.resolve({
        queue_record_version: "0.1.0",
        decision_id: `dec-${request["record_id"]}`,
        ...request,
      });
    },
  };
}

function reduceAll(actions: Action[]): AppState {
  let state = createInitialState();
  for (const action of actions) state = reduce(state, action);
  return state;
}

describe("intake full-state round trip (D-080 debt 2)", () => {
  it("grant -> stage two submissions -> scan -> decide approve + reject -> verify", async () => {
    recordCounter = 0;
    const dir = new FakeDir("intake-queue");
    const allActions: Action[] = [];
    const dispatch = (a: Action) => allActions.push(a);
    const client = fakeClient();

    // --- Step 1: Grant the queue directory ---
    const granted = await grantQueueDirectory(dir, dispatch);
    expect(granted).toBe(true);
    let state = reduceAll(allActions);
    expect(state.intake.dirName).toBe("intake-queue");
    expect(state.intake.status).toBe("ready");
    expect(state.intake.records).toHaveLength(0);

    // --- Step 2: Stage first submission ---
    const summary1 = await stageSubmission(
      { client, dir, dispatch, now: () => 1000 },
      { submission_id: "sub-1", fields: { display_name: "Ada Lovelace" } },
    );
    state = reduceAll(allActions);
    expect(state.intake.records).toHaveLength(1);
    expect(state.intake.records[0]!.recordId).toBe("rec-1");
    expect(state.intake.records[0]!.reviewState).toBe("pending");
    expect(dir.files.has("rec-1.record.json")).toBe(true);
    expect(dir.files.has("rec-1.sidecar.json")).toBe(true);

    // --- Step 3: Stage second submission ---
    const summary2 = await stageSubmission(
      { client, dir, dispatch, now: () => 2000 },
      { submission_id: "sub-2", fields: { display_name: "Charles Babbage" } },
    );
    state = reduceAll(allActions);
    expect(state.intake.records).toHaveLength(2);
    expect(state.intake.records[1]!.recordId).toBe("rec-2");

    // --- Step 4: Re-scan and verify consistency ---
    await scanQueue(dir, dispatch);
    state = reduceAll(allActions);
    expect(state.intake.status).toBe("ready");
    expect(state.intake.records).toHaveLength(2);
    expect(state.intake.scanIssues).toHaveLength(0);

    // --- Step 5: Approve first submission ---
    await stageDecision(
      { client, dir, dispatch, now: () => 3000 },
      { record: summary1, decision: "approve", reviewer: "facilitator-1" },
    );
    state = reduceAll(allActions);
    expect(state.intake.pendingDecisionFiles).toBe(1);
    const decisions = dir.dirs.get("decisions");
    expect(decisions).toBeDefined();
    const decisionFile = decisions?.files.get("rec-1.dec-rec-1.json");
    expect(decisionFile).toBeDefined();
    const decisionObj = JSON.parse(decisionFile!) as JsonObject;
    expect(decisionObj["decision"]).toBe("approve");
    expect(decisionObj["expected_review_state"]).toBe("pending");
    expect(decisionObj["expected_decision_generation"]).toBe(0);

    // --- Step 6: Reject second submission with reason ---
    await stageDecision(
      { client, dir, dispatch, now: () => 4000 },
      {
        record: summary2,
        decision: { reject: { reason: "Duplicate entry - already registered" } },
        reviewer: "facilitator-1",
      },
    );
    state = reduceAll(allActions);
    expect(state.intake.pendingDecisionFiles).toBe(2);

    // --- Step 7: Final scan and full-state verification ---
    await scanQueue(dir, dispatch);
    state = reduceAll(allActions);
    expect(state.intake.status).toBe("ready");
    expect(state.intake.records).toHaveLength(2);
    expect(state.intake.pendingDecisionFiles).toBe(2);
    expect(state.intake.scanIssues).toHaveLength(0);
    expect(state.intake.lastError).toBeNull();

    // --- Step 8: Cleanup ---
    clearQueueDirectory(dispatch);
    state = reduceAll(allActions);
    expect(state.intake.dirName).toBeNull();
    expect(state.intake.records).toHaveLength(0);
    expect(state.intake.status).toBe("idle");
  });

  it("refuses staging a decision on an approved_intent record", async () => {
    recordCounter = 100;
    const dir = new FakeDir("intake-queue");
    const { dispatch } = { dispatch: () => {} };
    const client = fakeClient();
    const summary = await stageSubmission(
      { client, dir, dispatch, now: () => 1000 },
      { submission_id: "sub-mid" },
    );
    const midApproval = { ...summary, reviewState: "approved_intent" as const };
    await expect(
      stageDecision(
        { client, dir, dispatch, now: () => 2000 },
        { record: midApproval, decision: "approve", reviewer: "fac-1" },
      ),
    ).rejects.toMatchObject({ code: "IntakeUnreviewable" });
  });

  it("refuses staging a decision on a record with no readable sidecar", async () => {
    recordCounter = 200;
    const dir = new FakeDir("intake-queue");
    const { dispatch } = { dispatch: () => {} };
    const client = fakeClient();
    const summary = await stageSubmission(
      { client, dir, dispatch, now: () => 1000 },
      { submission_id: "sub-unreadable" },
    );
    const unreadable = { ...summary, reviewState: null };
    await expect(
      stageDecision(
        { client, dir, dispatch, now: () => 2000 },
        { record: unreadable, decision: "approve", reviewer: "fac-1" },
      ),
    ).rejects.toMatchObject({ code: "IntakeUnreviewable" });
  });
});

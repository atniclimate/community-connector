import { describe, expect, it } from "vitest";

import type { Action } from "./actions";
import type {
  IntakeBuilderClient,
  IntakeDirHandle,
  IntakeFileHandle,
} from "./intake";
import {
  clearQueueDirectory,
  createOnly,
  getQueueDirectory,
  grantQueueDirectory,
  guardQueueDirectory,
  scanQueue,
  stageDecision,
  stageSubmission,
} from "./intake";
import type { IntakeRecordSummaryDto, JsonObject } from "./state";

/** In-memory fake of the structural FSA surface. */
class FakeDir implements IntakeDirHandle {
  public readonly files = new Map<string, string>();
  public readonly dirs = new Map<string, FakeDir>();

  public constructor(public readonly name: string) {}

  public getFileHandle(
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
          write: (data: string) => {
            buffer += data;
            return Promise.resolve();
          },
          close: () => {
            files.set(name, buffer);
            return Promise.resolve();
          },
        });
      },
    });
  }

  public getDirectoryHandle(
    name: string,
    options?: { readonly create?: boolean },
  ): Promise<IntakeDirHandle> {
    const existing = this.dirs.get(name);
    if (existing !== undefined) {
      return Promise.resolve(existing);
    }
    if (options?.create !== true) {
      return Promise.reject(new Error(`NotFound: ${name}`));
    }
    const created = new FakeDir(name);
    this.dirs.set(name, created);
    return Promise.resolve(created);
  }

  public async *keys(): AsyncIterable<string> {
    for (const name of [...this.files.keys(), ...this.dirs.keys()].sort()) {
      yield name;
    }
  }
}

const stagedPair: JsonObject = {
  record: {
    queue_record_version: "0.1.0",
    record_id: "rec-1",
    staged_at: 10,
    source: { kind: "in_app" },
    payload: { submission_id: "sub-1", fields: { display_name: "Ada" } },
    payload_hash: "ph",
    record_checksum: "rc",
  },
  sidecar: {
    queue_record_version: "0.1.0",
    record_id: "rec-1",
    payload_digest: "rc",
    review_state: "pending",
    sidecar_revision: 0,
    decision_generation: 0,
    history: [],
    plan: null,
    validation_report: null,
    sidecar_checksum: "sc",
  },
};

function fakeClient(): IntakeBuilderClient {
  return {
    intakeStageRecord: () => Promise.resolve(structuredClone(stagedPair)),
    intakeBuildDecision: (requestJson: string) => {
      const request = JSON.parse(requestJson) as JsonObject;
      return Promise.resolve({
        queue_record_version: "0.1.0",
        decision_id: "dec-1",
        ...request,
      });
    },
  };
}

function collect(): { actions: Action[]; dispatch: (action: Action) => void } {
  const actions: Action[] = [];
  return { actions, dispatch: (action) => actions.push(action) };
}

describe("guardQueueDirectory (best-effort wizard mirror of the native guard)", () => {
  it("refuses cloud-sync names and .git-bearing directories", async () => {
    expect(await guardQueueDirectory(new FakeDir("OneDrive - org"))).toContain("cloud-synced");
    const worktree = new FakeDir("queue");
    worktree.dirs.set(".git", new FakeDir(".git"));
    expect(await guardQueueDirectory(worktree)).toContain("worktree");
    expect(await guardQueueDirectory(new FakeDir("intake-queue"))).toBeNull();
  });
});

describe("createOnly (the app's whole write authority)", () => {
  it("writes with read-back verification and refuses existing files", async () => {
    const dir = new FakeDir("queue");
    await createOnly(dir, "a.json", "{\"x\":1}");
    expect(dir.files.get("a.json")).toBe("{\"x\":1}");
    await expect(createOnly(dir, "a.json", "other")).rejects.toMatchObject({
      code: "IntakeCreateOnly",
    });
  });
});

describe("stageSubmission (record then sidecar, both create-only)", () => {
  it("writes the core-built pair verbatim and dispatches the staged summary", async () => {
    const dir = new FakeDir("queue");
    const { actions, dispatch } = collect();
    const summary = await stageSubmission(
      { client: fakeClient(), dir, dispatch, now: () => 10 },
      { submission_id: "sub-1" },
    );
    expect(dir.files.has("rec-1.record.json")).toBe(true);
    expect(dir.files.has("rec-1.sidecar.json")).toBe(true);
    // The written record is the core's bytes re-serialized, content-equal.
    const written = JSON.parse(dir.files.get("rec-1.record.json") ?? "") as JsonObject;
    expect(written).toEqual(stagedPair["record"]);
    expect(summary.reviewState).toBe("pending");
    expect(summary.payloadDigest).toBe("rc");
    expect(actions).toEqual([{ kind: "intakeRecordStaged", record: summary }]);
  });
});

describe("stageDecision (create-only decision files with the CAS premise)", () => {
  const record: IntakeRecordSummaryDto = {
    recordId: "rec-1",
    reviewState: "pending",
    decisionGeneration: 3,
    payloadDigest: "rc",
    payloadHash: "ph",
    stagedAtMs: 10,
    payload: {},
    record: {},
  };

  it("writes into decisions/ carrying the observed state and generation", async () => {
    const dir = new FakeDir("queue");
    const { actions, dispatch } = collect();
    await stageDecision(
      { client: fakeClient(), dir, dispatch, now: () => 99 },
      { record, decision: "approve", reviewer: "fac-1" },
    );
    const decisions = dir.dirs.get("decisions");
    const written = JSON.parse(
      decisions?.files.get("rec-1.dec-1.json") ?? "",
    ) as JsonObject;
    expect(written["expected_review_state"]).toBe("pending");
    expect(written["expected_decision_generation"]).toBe(3);
    expect(written["payload_digest"]).toBe("rc");
    expect(actions).toEqual([{ kind: "intakeReviewDecided", recordId: "rec-1" }]);
  });

  it("refuses unreadable and mid-approval records (native recovery owns them)", async () => {
    const dir = new FakeDir("queue");
    const { dispatch } = collect();
    const deps = { client: fakeClient(), dir, dispatch };
    await expect(
      stageDecision(deps, {
        record: { ...record, reviewState: null },
        decision: "approve",
        reviewer: "fac-1",
      }),
    ).rejects.toMatchObject({ code: "IntakeUnreviewable" });
    await expect(
      stageDecision(deps, {
        record: { ...record, reviewState: "approved_intent" },
        decision: "approve",
        reviewer: "fac-1",
      }),
    ).rejects.toMatchObject({ code: "IntakeUnreviewable" });
    expect(dir.dirs.has("decisions")).toBe(false);
  });
});

describe("scanQueue (read-only dashboard view, I12)", () => {
  it("summarizes records, flags unreadable sidecars, counts staged decisions", async () => {
    const dir = new FakeDir("queue");
    const { actions, dispatch } = collect();
    await stageSubmission({ client: fakeClient(), dir, dispatch, now: () => 10 }, {});
    // A second record with a corrupt sidecar.
    dir.files.set("rec-2.record.json", JSON.stringify({
      record_id: "rec-2",
      staged_at: 11,
      payload: {},
      record_checksum: "rc2",
    }));
    dir.files.set("rec-2.sidecar.json", "{corrupt");
    const decisions = new FakeDir("decisions");
    decisions.files.set("rec-1.dec-1.json", "{}");
    decisions.dirs.set("consumed", new FakeDir("consumed"));
    dir.dirs.set("decisions", decisions);

    await scanQueue(dir, dispatch);
    const loaded = actions.at(-1);
    expect(loaded?.kind).toBe("intakeQueueLoaded");
    if (loaded?.kind !== "intakeQueueLoaded") {
      return;
    }
    expect(loaded.records.map((r) => r.recordId)).toEqual(["rec-1", "rec-2"]);
    expect(loaded.records[0]?.reviewState).toBe("pending");
    expect(loaded.records[1]?.reviewState).toBeNull();
    expect(loaded.pendingDecisionFiles).toBe(1);
  });
});


describe("grantQueueDirectory (round-1 F4: the effects layer owns the handle)", () => {
  it("refuses unsafe directories with a dispatched error and no registration", async () => {
    const { actions, dispatch } = collect();
    const unsafe = new FakeDir("OneDrive - org");
    expect(await grantQueueDirectory(unsafe, dispatch)).toBe(false);
    expect(getQueueDirectory()).toBeNull();
    expect(actions[0]?.kind).toBe("intakeFailed");
  });

  it("registers, announces, and scans a safe directory", async () => {
    const { actions, dispatch } = collect();
    const dir = new FakeDir("intake-queue");
    expect(await grantQueueDirectory(dir, dispatch)).toBe(true);
    expect(getQueueDirectory()).toBe(dir);
    // In this node environment IndexedDB is absent, so the honest
    // persist-failure notice fires between grant and scan (round-2 F11).
    expect(actions.map((a) => a.kind)).toEqual([
      "intakeDirGranted",
      "intakeFailed",
      "intakeScanStarted",
      "intakeQueueLoaded",
    ]);
    clearQueueDirectory(dispatch);
    expect(getQueueDirectory()).toBeNull();
  });
});

describe("scanQueue issue surfacing (round-1 F3)", () => {
  it("reports unreadable records and sidecars instead of silently dropping them", async () => {
    const dir = new FakeDir("queue");
    dir.files.set("rec-x.record.json", "{corrupt");
    dir.files.set("rec-y.record.json", JSON.stringify({ record_id: "rec-y", payload: {} }));
    dir.files.set("rec-y.sidecar.json", "{corrupt");
    const { actions, dispatch } = collect();
    await scanQueue(dir, dispatch);
    const loaded = actions.at(-1);
    if (loaded?.kind !== "intakeQueueLoaded") {
      throw new Error("expected intakeQueueLoaded");
    }
    expect(loaded.scanIssues.length).toBe(2);
    expect(loaded.scanIssues[0]).toContain("rec-x.record.json");
    expect(loaded.scanIssues[1]).toContain("rec-y.sidecar.json");
    expect(loaded.records.map((r) => r.recordId)).toEqual(["rec-y"]);
    expect(loaded.records[0]?.reviewState).toBeNull();
  });
});

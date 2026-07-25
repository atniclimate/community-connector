/**
 * Intake queue effects: the app's CREATE-ONLY File System Access adapter
 * plus the wasm builder calls (ADR-005 D4; blueprint sections 1 and 5;
 * D-070.3 fixes the on-disk layout; D-073 keeps checksum authority in
 * the core - the app never computes a digest).
 *
 * The app creates new uniquely named files and NEVER rewrites an existing
 * one: payload records, their initial pending sidecars, and decision
 * files. Every create is followed by read-back verification before the
 * UI reports success. All mutation of authoritative state (sidecar
 * rewrites, the op log) belongs to native `cn intake apply`; the wizard
 * merely stages and surfaces.
 *
 * All state flows through the store (I4): this module owns the I/O and
 * dispatches; the reducer stays pure.
 */

import type { Action } from "./actions";
import type {
  ErrorEnvelopeDto,
  IntakeRecordSummaryDto,
  JsonObject,
  JsonValue,
} from "./state";

/** Structural FSA surface the adapter needs (real handles and test fakes
 * both satisfy it; the picker itself is wired by the wizard). */
export type IntakeWritable = {
  readonly write: (data: string) => Promise<void>;
  readonly close: () => Promise<void>;
};

export type IntakeFileHandle = {
  readonly getFile: () => Promise<{ text: () => Promise<string> }>;
  readonly createWritable: () => Promise<IntakeWritable>;
};

export type IntakeDirHandle = {
  readonly name: string;
  readonly getFileHandle: (
    name: string,
    options?: { readonly create?: boolean },
  ) => Promise<IntakeFileHandle>;
  readonly getDirectoryHandle: (
    name: string,
    options?: { readonly create?: boolean },
  ) => Promise<IntakeDirHandle>;
  readonly keys: () => AsyncIterable<string>;
};

/** The wasm-client subset the effects consume (structural for tests). */
export type IntakeBuilderClient = {
  readonly intakeStageRecord: (payloadJson: string, stagedAtMs: number) => Promise<JsonObject>;
  readonly intakeBuildDecision: (requestJson: string) => Promise<JsonObject>;
};

export type Dispatch = (action: Action) => void;

/**
 * The live FSA handle is I/O infrastructure owned by THIS effects module
 * (round-1 F4): the store describes it (dirName), components never hold
 * it. One queue directory per app session.
 */
let queueDirectory: IntakeDirHandle | null = null;

export function getQueueDirectory(): IntakeDirHandle | null {
  return queueDirectory;
}

export function clearQueueDirectory(dispatch: Dispatch): void {
  queueDirectory = null;
  dispatch({ kind: "intakeDirCleared" });
}

/**
 * Guards, registers, persists, and scans a picked queue directory - the
 * single entry point for granting (wizard button or IndexedDB restore).
 */
export async function grantQueueDirectory(
  dir: IntakeDirHandle,
  dispatch: Dispatch,
): Promise<boolean> {
  const refusal = await guardQueueDirectory(dir);
  if (refusal !== null) {
    dispatch({
      kind: "intakeFailed",
      error: { code: "IntakeUnsafeDirectory", message: refusal },
    });
    return false;
  }
  queueDirectory = dir;
  dispatch({ kind: "intakeDirGranted", dirName: dir.name });
  const persisted = await persistHandle(dir);
  if (!persisted) {
    // Visible, non-fatal (round-2 F11): the grant works this session,
    // but the "use previously granted folder" promise would be false.
    dispatch({
      kind: "intakeFailed",
      error: {
        code: "IntakePersistFailed",
        message:
          "Queue folder granted, but it could not be remembered for the next " +
          "session; you will need to grant it again after a reload",
      },
    });
  }
  await scanQueue(dir, dispatch);
  return true;
}

// --- IndexedDB handle persistence (blueprint section 5; round-1 F11) ---

const IDB_NAME = "cn-intake";
const IDB_STORE = "handles";
const IDB_KEY = "queue-directory";

function openHandleDb(): Promise<IDBDatabase | null> {
  if (typeof indexedDB === "undefined") {
    return Promise.resolve(null);
  }
  return new Promise((resolve) => {
    const request = indexedDB.open(IDB_NAME, 1);
    request.onupgradeneeded = () => {
      request.result.createObjectStore(IDB_STORE);
    };
    request.onsuccess = () => resolve(request.result);
    request.onerror = () => resolve(null);
  });
}

async function persistHandle(dir: IntakeDirHandle): Promise<boolean> {
  const db = await openHandleDb();
  if (db === null) {
    return false;
  }
  const stored = await new Promise<boolean>((resolve) => {
    const tx = db.transaction(IDB_STORE, "readwrite");
    tx.objectStore(IDB_STORE).put(dir, IDB_KEY);
    tx.oncomplete = () => resolve(true);
    tx.onerror = () => resolve(false);
  });
  db.close();
  return stored;
}

type PermissionedHandle = IntakeDirHandle & {
  readonly queryPermission?: (options: { mode: string }) => Promise<string>;
  readonly requestPermission?: (options: { mode: string }) => Promise<string>;
};

export type RestoreOutcome = "granted" | "none" | "failed";

/**
 * Restores a previously granted handle from IndexedDB and re-requests
 * permission (must run from a user gesture in most browsers). The
 * outcome distinguishes "nothing saved" from "restore failed" (round-2
 * F3: failures are never disguised as absence).
 */
export async function restoreQueueDirectory(dispatch: Dispatch): Promise<RestoreOutcome> {
  const db = await openHandleDb();
  if (db === null) {
    return "failed";
  }
  const stored = await new Promise<PermissionedHandle | null | "error">((resolve) => {
    const tx = db.transaction(IDB_STORE, "readonly");
    const request = tx.objectStore(IDB_STORE).get(IDB_KEY);
    request.onsuccess = () => resolve((request.result as PermissionedHandle) ?? null);
    request.onerror = () => resolve("error");
  });
  db.close();
  if (stored === "error") {
    return "failed";
  }
  if (stored === null) {
    return "none";
  }
  try {
    const query = (await stored.queryPermission?.({ mode: "readwrite" })) ?? "granted";
    const status =
      query === "granted"
        ? "granted"
        : ((await stored.requestPermission?.({ mode: "readwrite" })) ?? "denied");
    if (status !== "granted") {
      return "failed";
    }
  } catch {
    return "failed";
  }
  return (await grantQueueDirectory(stored, dispatch)) ? "granted" : "failed";
}

function intakeError(code: string, message: string): ErrorEnvelopeDto {
  return { code, message };
}

/**
 * Best-effort in-browser mirror of the native queue-root guard (the
 * blueprint's wizard rule): refuse a directory that looks like a git
 * worktree or a cloud-synced folder. FSA grants no ancestor access, so
 * only the granted directory itself is probed - the native CLI's guard
 * remains the authoritative one. Returns a refusal reason or null.
 */
export async function guardQueueDirectory(dir: IntakeDirHandle): Promise<string | null> {
  const lowered = dir.name.toLowerCase();
  for (const marker of ["dropbox", "onedrive", "google drive", "googledrive"]) {
    if (lowered.includes(marker)) {
      return `directory name '${dir.name}' looks cloud-synced ('${marker}'); no sync agent may cover the queue (ADR-005 D4)`;
    }
  }
  for (const probe of [
    () => dir.getDirectoryHandle(".git"),
    () => dir.getFileHandle(".git"),
  ]) {
    try {
      await probe();
      return `directory '${dir.name}' contains .git; the queue root must live outside any git worktree (ADR-005 D4)`;
    } catch {
      // Absent: the good case.
    }
  }
  return null;
}

function isNotFound(error: unknown): boolean {
  if (typeof error === "object" && error !== null) {
    const named = error as { readonly name?: unknown; readonly message?: unknown };
    if (named.name === "NotFoundError") {
      return true;
    }
    if (typeof named.message === "string" && named.message.includes("NotFound")) {
      return true;
    }
  }
  return false;
}

/** Existence check that FAILS CLOSED (round-2 F3): only a NotFound error
 * means absent - a permission or IO failure must never let createOnly
 * proceed as though the file were missing. */
async function exists(dir: IntakeDirHandle, name: string): Promise<boolean> {
  try {
    await dir.getFileHandle(name);
    return true;
  } catch (error) {
    if (isNotFound(error)) {
      return false;
    }
    throw intakeError(
      "IntakeIoError",
      `cannot determine whether '${name}' exists (${String(error)}); refusing to write`,
    );
  }
}

/**
 * The create-only write contract: refuse when the name exists, create,
 * write, close (FSA atomic replace-on-close), then read back and compare
 * bytes before reporting success. A mismatch is a loud typed error.
 */
export async function createOnly(
  dir: IntakeDirHandle,
  name: string,
  text: string,
): Promise<void> {
  if (await exists(dir, name)) {
    throw intakeError(
      "IntakeCreateOnly",
      `'${name}' already exists; the app never rewrites an existing queue file (ADR-005 D4)`,
    );
  }
  const handle = await dir.getFileHandle(name, { create: true });
  const writable = await handle.createWritable();
  await writable.write(text);
  await writable.close();
  const readBack = await (await handle.getFile()).text();
  if (readBack !== text) {
    throw intakeError(
      "IntakeReadBackMismatch",
      `read-back verification failed for '${name}'; treat the file as suspect and re-create it`,
    );
  }
}

function summarize(record: JsonObject, sidecar: JsonObject | null): IntakeRecordSummaryDto {
  const payload = record["payload"];
  return {
    recordId: typeof record["record_id"] === "string" ? record["record_id"] : "",
    reviewState:
      sidecar !== null && typeof sidecar["review_state"] === "string"
        ? sidecar["review_state"]
        : null,
    decisionGeneration:
      sidecar !== null && typeof sidecar["decision_generation"] === "number"
        ? sidecar["decision_generation"]
        : 0,
    payloadDigest: typeof record["record_checksum"] === "string" ? record["record_checksum"] : "",
    payloadHash: typeof record["payload_hash"] === "string" ? record["payload_hash"] : "",
    stagedAtMs: typeof record["staged_at"] === "number" ? record["staged_at"] : 0,
    payload: typeof payload === "object" && payload !== null && !Array.isArray(payload)
      ? (payload as JsonObject)
      : {},
    record,
  };
}

export type StageSubmissionDeps = {
  readonly client: IntakeBuilderClient;
  readonly dir: IntakeDirHandle;
  readonly dispatch: Dispatch;
  readonly now?: () => number;
};

/**
 * Stages one form payload: the core builds the checksummed record and
 * initial pending sidecar (D-073); the app writes record THEN sidecar
 * create-only (the crash between the two writes is the recovery table's
 * reconstruct-pending row).
 */
export async function stageSubmission(
  deps: StageSubmissionDeps,
  payload: JsonObject,
): Promise<IntakeRecordSummaryDto> {
  const now = deps.now ?? (() => Date.now());
  const staged = await deps.client.intakeStageRecord(JSON.stringify(payload), now());
  const record = staged["record"];
  const sidecar = staged["sidecar"];
  if (
    typeof record !== "object" || record === null || Array.isArray(record) ||
    typeof sidecar !== "object" || sidecar === null || Array.isArray(sidecar)
  ) {
    throw intakeError("IntakeProtocol", "stage builder returned an unexpected shape");
  }
  const recordObject = record as JsonObject;
  const sidecarObject = sidecar as JsonObject;
  const recordId = recordObject["record_id"];
  if (typeof recordId !== "string" || recordId.length === 0) {
    throw intakeError("IntakeProtocol", "stage builder returned no record id");
  }
  await createOnly(deps.dir, `${recordId}.record.json`, JSON.stringify(recordObject, null, 2));
  await createOnly(deps.dir, `${recordId}.sidecar.json`, JSON.stringify(sidecarObject, null, 2));
  const summary = summarize(recordObject, sidecarObject);
  deps.dispatch({ kind: "intakeRecordStaged", record: summary });
  return summary;
}

export type StageDecisionDeps = StageSubmissionDeps;

export type DecisionInput = {
  readonly record: IntakeRecordSummaryDto;
  /** "approve" | "reject" | { set_aside_note: { note } } | "clear_failed" */
  readonly decision: JsonValue;
  readonly reviewer: string;
};

/**
 * Stages one review decision as a create-only decision file carrying the
 * CAS premise the wizard observed. The wizard refuses records whose
 * sidecar is approved_intent or unreadable (native recovery owns those).
 */
export async function stageDecision(
  deps: StageDecisionDeps,
  input: DecisionInput,
): Promise<void> {
  const state = input.record.reviewState;
  if (state === null) {
    throw intakeError(
      "IntakeUnreviewable",
      `record ${input.record.recordId} has no readable sidecar; run cn intake apply (native recovery) first`,
    );
  }
  if (state === "approved_intent") {
    throw intakeError(
      "IntakeUnreviewable",
      `record ${input.record.recordId} is mid-approval (approved_intent); run cn intake apply first`,
    );
  }
  const now = deps.now ?? (() => Date.now());
  const message = await deps.client.intakeBuildDecision(
    JSON.stringify({
      record_id: input.record.recordId,
      payload_digest: input.record.payloadDigest,
      expected_review_state: state,
      expected_decision_generation: input.record.decisionGeneration,
      decision: input.decision,
      reviewer: input.reviewer,
      decided_at_ms: now(),
    }),
  );
  const decisionId = message["decision_id"];
  if (typeof decisionId !== "string" || decisionId.length === 0) {
    throw intakeError("IntakeProtocol", "decision builder returned no decision id");
  }
  const decisions = await deps.dir.getDirectoryHandle("decisions", { create: true });
  await createOnly(
    decisions,
    `${input.record.recordId}.${decisionId}.json`,
    JSON.stringify(message, null, 2),
  );
  deps.dispatch({ kind: "intakeReviewDecided", recordId: input.record.recordId });
}

type ReadResult = { readonly value: JsonObject } | { readonly issue: string };

/** Tagged read: an unreadable file is a REPORTED issue, never silently
 * identical to an absent one (round-1 F3, I12). */
async function readJson(dir: IntakeDirHandle, name: string): Promise<ReadResult> {
  let text: string;
  try {
    text = await (await (await dir.getFileHandle(name)).getFile()).text();
  } catch (error) {
    return { issue: `${name}: unreadable (${String(error)})` };
  }
  try {
    const parsed: unknown = JSON.parse(text);
    if (typeof parsed === "object" && parsed !== null && !Array.isArray(parsed)) {
      return { value: parsed as JsonObject };
    }
    return { issue: `${name}: not a JSON object` };
  } catch (error) {
    return { issue: `${name}: does not parse (${String(error)})` };
  }
}

/**
 * Read-only queue scan for the dashboard (I12 visibility): summaries per
 * record plus the count of staged-but-unapplied decision files. Display
 * only - the native CLI re-verifies checksums and bindings itself.
 */
export async function scanQueue(dir: IntakeDirHandle, dispatch: Dispatch): Promise<void> {
  dispatch({ kind: "intakeScanStarted" });
  const recordNames: string[] = [];
  for await (const name of dir.keys()) {
    if (name.endsWith(".record.json")) {
      recordNames.push(name);
    }
  }
  recordNames.sort();
  const records: IntakeRecordSummaryDto[] = [];
  const scanIssues: string[] = [];
  for (const name of recordNames) {
    const recordId = name.slice(0, -".record.json".length);
    const record = await readJson(dir, name);
    if ("issue" in record) {
      scanIssues.push(record.issue);
      continue;
    }
    const sidecar = await readJson(dir, `${recordId}.sidecar.json`);
    if ("issue" in sidecar) {
      scanIssues.push(sidecar.issue);
    }
    records.push(summarize(record.value, "issue" in sidecar ? null : sidecar.value));
  }
  let pendingDecisionFiles = 0;
  try {
    const decisions = await dir.getDirectoryHandle("decisions");
    for await (const name of decisions.keys()) {
      if (name.endsWith(".json") && !name.includes(".tmp-")) {
        pendingDecisionFiles += 1;
      }
    }
  } catch (error) {
    if (!isNotFound(error)) {
      // An enumeration failure is NOT "zero staged decisions" (round-2
      // F3): report it so the dashboard never understates staged work.
      scanIssues.push(`decisions/: enumeration failed (${String(error)})`);
    }
  }
  dispatch({ kind: "intakeQueueLoaded", records, pendingDecisionFiles, scanIssues });
}

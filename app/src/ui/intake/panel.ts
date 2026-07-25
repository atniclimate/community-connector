/**
 * P3.5 facilitator wizard (docs/blueprints/intake-pipeline.md section 5):
 * ops-directory step, queue dashboard, new-entry flow, review view, and
 * the staged-awaiting-apply surface. Mounted only when the active viewer
 * resolves to facilitator-or-governance - an AFFORDANCE; the core
 * enforces authority regardless.
 *
 * The wizard renders and stages: every mutation of authoritative state
 * belongs to native `cn intake apply` (the dashboard shows the apply
 * instruction and, after an apply, the facilitator reloads the group via
 * the existing load path). Entry and approval remain TWO distinct
 * recorded acts even when one person performs both (D-030).
 */

import type { Action } from "../../state/actions";
import type { IntakeDirHandle } from "../../state/intake";
import {
  getQueueDirectory,
  grantQueueDirectory,
  restoreQueueDirectory,
  scanQueue,
  stageDecision,
  stageSubmission,
} from "../../state/intake";
import type { Store } from "../../state/store";
import type {
  IntakeRecordSummaryDto,
  JsonObject,
  ViewerContextDto,
} from "../../state/state";
import type { WasmClient } from "../../wasm/client";
import { el } from "../dom";
import { mountEntryForm } from "../forms/renderer";
import { dashboard, dedupKeys, nearDupRequest, reviewable } from "./model";

export type IntakeWizardDeps = {
  readonly store: Store;
  readonly client: WasmClient;
  readonly groupId: () => string | null;
  readonly viewer: () => ViewerContextDto;
  readonly template: () => JsonObject | null;
  /** Injectable directory picker (defaults to window.showDirectoryPicker). */
  readonly pickDirectory?: () => Promise<IntakeDirHandle>;
  readonly now?: () => number;
};

function defaultPicker(): Promise<IntakeDirHandle> {
  const picker = (
    window as unknown as { showDirectoryPicker?: (options: JsonObject) => Promise<unknown> }
  ).showDirectoryPicker;
  if (picker === undefined) {
    return Promise.reject(
      new Error("This browser has no File System Access support; use Chromium on the pilot PC"),
    );
  }
  return picker({ mode: "readwrite" }) as Promise<IntakeDirHandle>;
}

export function mountIntakeWizard(container: HTMLElement, deps: IntakeWizardDeps): () => void {
  const now = deps.now ?? (() => Date.now());
  // The FSA handle lives in the state module's effects layer and review
  // selection lives in the store (round-1 F4); the only component-local
  // value is DOM housekeeping.
  let unmountForm: (() => void) | null = null;

  const dispatch = (action: Action): void => {
    deps.store.dispatch(action);
  };
  const fail = (error: unknown): void => {
    dispatch({ kind: "intakeFailed", error: deps.client.toErrorEnvelope(error) });
  };

  const root = el("section", {
    className: "cn-intake",
    attrs: { "aria-label": "Intake queue (facilitator)" },
  });
  const heading = el("h2", { text: "Intake queue" });
  const dirRegion = el("div", { className: "cn-intake-dir" });
  const dashRegion = el("div", { className: "cn-intake-dashboard", attrs: { role: "status" } });
  const entryRegion = el("details", { className: "cn-intake-entry" }, [
    el("summary", { text: "New entry (staged for review, not published)" }),
  ]);
  const entryHost = el("div", {});
  entryRegion.append(entryHost);
  const listRegion = el("div", { className: "cn-intake-records" });
  const reviewRegion = el("div", { className: "cn-intake-review" });
  root.append(heading, dirRegion, dashRegion, entryRegion, listRegion, reviewRegion);
  container.append(root);

  async function grantDirectory(): Promise<void> {
    const picked = await (deps.pickDirectory ?? defaultPicker)();
    await grantQueueDirectory(picked, dispatch);
  }

  function renderDir(dirName: string | null): void {
    dirRegion.replaceChildren();
    if (dirName === null) {
      const grant = el("button", {
        text: "Grant queue folder (outside any repo or synced folder)",
        attrs: { type: "button" },
      });
      grant.addEventListener("click", () => {
        grantDirectory().catch(fail);
      });
      const restore = el("button", {
        text: "Use previously granted folder",
        attrs: { type: "button" },
      });
      restore.addEventListener("click", () => {
        restoreQueueDirectory(dispatch)
          .then((restored) => {
            if (!restored) {
              fail({
                code: "IntakeNoSavedFolder",
                message: "No previously granted folder could be restored; grant one",
              });
            }
          })
          .catch(fail);
      });
      dirRegion.append(grant, restore);
      return;
    }
    const rescan = el("button", { text: "Rescan queue", attrs: { type: "button" } });
    rescan.addEventListener("click", () => {
      const dir = getQueueDirectory();
      if (dir !== null) {
        scanQueue(dir, dispatch).catch(fail);
      }
    });
    dirRegion.append(el("span", { text: `Queue folder: ${dirName} ` }), rescan);
  }

  function renderDashboard(): void {
    const intake = deps.store.getState().intake;
    dashRegion.replaceChildren();
    if (intake.dirName === null) {
      return;
    }
    const summary = dashboard(intake.records, intake.pendingDecisionFiles, now());
    const parts: string[] = Object.entries(summary.countsByState).map(
      ([state, count]) => `${state}: ${count}`,
    );
    if (summary.unreadable > 0) {
      parts.push(`unreadable: ${summary.unreadable}`);
    }
    if (summary.oldestPendingAgeMs !== null) {
      parts.push(`oldest pending: ${Math.round(summary.oldestPendingAgeMs / 60_000)} min`);
    }
    dashRegion.append(el("p", { text: parts.length > 0 ? parts.join(" | ") : "Queue is empty." }));
    if (summary.pendingDecisionFiles > 0) {
      dashRegion.append(
        el("p", {
          className: "cn-intake-apply-note",
          text:
            `${summary.pendingDecisionFiles} staged decision(s) await the native owner: ` +
            "run `cn intake apply` on the pilot PC, then reload the group to see " +
            "approved entries.",
        }),
      );
    }
    for (const issue of intake.scanIssues) {
      dashRegion.append(
        el("p", { className: "cn-intake-error", text: `Scan issue: ${issue}` }),
      );
    }
    if (intake.lastError !== null) {
      dashRegion.append(
        el("p", { className: "cn-intake-error", text: `Error: ${intake.lastError.message}` }),
      );
    }
  }

  function stageFromForm(payload: JsonObject): void {
    const dir = getQueueDirectory();
    if (dir === null) {
      fail({ code: "IntakeNoDirectory", message: "Grant the queue folder first" });
      return;
    }
    stageSubmission({ client: deps.client, dir, dispatch, now }, payload)
      .then(() => scanQueue(dir, dispatch))
      .catch(fail);
  }

  function renderEntryForm(): void {
    const template = deps.template();
    if (template === null || unmountForm !== null) {
      return;
    }
    unmountForm = mountEntryForm(entryHost, { template, onStage: stageFromForm, now });
  }

  function decide(record: IntakeRecordSummaryDto, decision: unknown): void {
    const dir = getQueueDirectory();
    if (dir === null) {
      return;
    }
    const reviewer = viewerName(deps.viewer());
    stageDecision(
      { client: deps.client, dir, dispatch, now },
      { record, decision: decision as never, reviewer },
    )
      .then(() => scanQueue(dir, dispatch))
      .catch(fail);
  }

  function decisionButtons(record: IntakeRecordSummaryDto): HTMLElement {
    const row = el("div", { className: "cn-intake-decisions" });
    if (record.reviewState === "pending") {
      const approve = el("button", { text: "Approve", attrs: { type: "button" } });
      approve.addEventListener("click", () => decide(record, "approve"));
      const reject = el("button", { text: "Reject...", attrs: { type: "button" } });
      reject.addEventListener("click", () => {
        const reason = window.prompt("Rejection reason (required; the record persists):");
        if (reason !== null && reason.trim().length > 0) {
          decide(record, { reject: { reason: reason.trim() } });
        }
      });
      const note = el("button", { text: "Set aside note...", attrs: { type: "button" } });
      note.addEventListener("click", () => {
        const text = window.prompt("Note (stays pending; invalidates older views):");
        if (text !== null && text.trim().length > 0) {
          decide(record, { set_aside_note: { note: text.trim() } });
        }
      });
      row.append(approve, reject, note);
    } else if (record.reviewState === "failed") {
      const clear = el("button", { text: "Clear failed -> pending", attrs: { type: "button" } });
      clear.addEventListener("click", () => decide(record, "clear_failed"));
      row.append(
        el("p", {
          className: "cn-intake-error",
          text: "Failed: investigate before clearing (terminal until explicit disposition).",
        }),
        clear,
      );
    }
    return row;
  }

  function renderReview(): void {
    reviewRegion.replaceChildren();
    const state = deps.store.getState();
    const record = state.intake.records.find(
      (entry) => entry.recordId === state.intake.reviewRecordId,
    );
    const groupId = deps.groupId();
    if (record === undefined) {
      return;
    }
    const fields = record.payload["fields"];
    const fieldList = el("dl", {});
    if (typeof fields === "object" && fields !== null && !Array.isArray(fields)) {
      for (const [key, value] of Object.entries(fields)) {
        fieldList.append(
          el("dt", { text: key }),
          el("dd", { text: JSON.stringify(value) }),
        );
      }
    }
    const asyncRegion = el("div", { className: "cn-intake-review-checks", attrs: { role: "status" } });
    reviewRegion.append(
      el("h3", { text: `Review ${record.recordId}` }),
      el("p", {
        text:
          `state: ${record.reviewState ?? "UNREADABLE (native recovery owns this record)"}` +
          ` | generation: ${String(record.decisionGeneration)}`,
      }),
      fieldList,
      asyncRegion,
      reviewable(record) ? decisionButtons(record) : el("div", {}),
    );

    // Read-only core checks (I2): validation, dedup, near-duplicates.
    if (groupId !== null) {
      const template = deps.template();
      const checks: Promise<string>[] = [
        deps.client
          .intakeValidateRecord(groupId, JSON.stringify(record.record))
          .then((result) => `validation: ${JSON.stringify(result["report"])}`),
        deps.client
          .intakeDedupCheck(
            JSON.stringify(record.record),
            JSON.stringify(dedupKeys(record, state.intake.records)),
          )
          .then((result) => `dedup: ${String(result["verdict"])}`),
      ];
      if (template !== null) {
        checks.push(
          deps.client
            .intakeNearDuplicates(
              groupId,
              deps.viewer(),
              JSON.stringify(nearDupRequest(template, record, state.intake.records)),
            )
            .then((candidates) =>
              candidates.length === 0
                ? "near-duplicates: none"
                : `near-duplicates: ${JSON.stringify(candidates)}`,
            ),
        );
      }
      for (const check of checks) {
        const line = el("p", { text: "checking..." });
        asyncRegion.append(line);
        check
          .then((text) => {
            line.textContent = text;
          })
          .catch((error: unknown) => {
            line.textContent = `check failed: ${deps.client.toErrorEnvelope(error).message}`;
          });
      }
    }
  }

  function renderRecords(): void {
    const intake = deps.store.getState().intake;
    listRegion.replaceChildren();
    if (intake.records.length === 0) {
      return;
    }
    const list = el("ul", {});
    for (const record of intake.records) {
      const label = `${record.recordId.slice(0, 8)}... [${record.reviewState ?? "unreadable"}]`;
      const open = el("button", { text: `Review ${label}`, attrs: { type: "button" } });
      open.addEventListener("click", () => {
        dispatch({ kind: "intakeReviewSelected", recordId: record.recordId });
      });
      list.append(el("li", {}, [open]));
    }
    listRegion.append(list);
  }

  function render(): void {
    const intake = deps.store.getState().intake;
    renderDir(intake.dirName);
    renderDashboard();
    renderEntryForm();
    renderRecords();
    renderReview();
  }

  const unsubscribe = deps.store.subscribe(render);
  render();

  return () => {
    unsubscribe();
    if (unmountForm !== null) {
      unmountForm();
    }
    root.remove();
  };
}

function viewerName(viewer: ViewerContextDto): string {
  const person = viewer["person"];
  return typeof person === "string" ? person : "anonymous";
}

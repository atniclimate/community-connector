import { describe, expect, it } from "vitest";

import type { IntakeState } from "../../state/state";
import { initialIntakeState } from "../../state/state";
import { dashboardErrorLines } from "./panel";

function intakeState(overrides: Partial<IntakeState>): IntakeState {
  return { ...initialIntakeState(), ...overrides };
}

describe("dashboardErrorLines (pre-grant error surfacing defect)", () => {
  it("surfaces lastError even when no queue directory is granted", () => {
    // Reproduces the rehearsal defect: stageFromForm() dispatches
    // intakeFailed({ code: "IntakeNoDirectory", ... }) before any
    // directory is granted, so dirName stays null while lastError is
    // set. The dashboard must still show it.
    const intake = intakeState({
      dirName: null,
      lastError: { code: "IntakeNoDirectory", message: "Grant the queue folder first" },
    });
    expect(dashboardErrorLines(intake)).toEqual([
      { className: "cn-intake-error", text: "Error: Grant the queue folder first" },
    ]);
  });

  it("surfaces IntakeNoSavedFolder and IntakeRestoreFailed the same way", () => {
    const noSaved = intakeState({
      dirName: null,
      lastError: {
        code: "IntakeNoSavedFolder",
        message: "No previously granted folder is saved; grant one",
      },
    });
    expect(dashboardErrorLines(noSaved)).toEqual([
      {
        className: "cn-intake-error",
        text: "Error: No previously granted folder is saved; grant one",
      },
    ]);

    const restoreFailed = intakeState({
      dirName: null,
      lastError: {
        code: "IntakeRestoreFailed",
        message: "A saved folder exists but could not be restored; grant it again",
      },
    });
    expect(dashboardErrorLines(restoreFailed)).toEqual([
      {
        className: "cn-intake-error",
        text: "Error: A saved folder exists but could not be restored; grant it again",
      },
    ]);
  });

  it("returns no lines when there is nothing to report, directory or not", () => {
    expect(dashboardErrorLines(intakeState({ dirName: null }))).toEqual([]);
    expect(dashboardErrorLines(intakeState({ dirName: "queue" }))).toEqual([]);
  });

  it("still surfaces notices and scan issues ahead of lastError, with a directory granted", () => {
    const intake = intakeState({
      dirName: "queue",
      notices: ["persistence unavailable"],
      scanIssues: ["rec-1: unreadable sidecar"],
      lastError: { code: "IntakeScanFailed", message: "scan failed" },
    });
    expect(dashboardErrorLines(intake)).toEqual([
      { className: "cn-intake-error", text: "Notice: persistence unavailable" },
      { className: "cn-intake-error", text: "Scan issue: rec-1: unreadable sidecar" },
      { className: "cn-intake-error", text: "Error: scan failed" },
    ]);
  });
});

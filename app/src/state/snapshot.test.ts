import { describe, expect, it } from "vitest";
import { createInitialState } from "./state";
import { createStore } from "./store";
import {
  bootSnapshot,
  parseSnapshotEnvelope,
  SNAPSHOT_ENVELOPE_VERSION,
  SnapshotReadError,
} from "./snapshot";

function envelope(): Record<string, unknown> {
  return {
    schema_version: SNAPSHOT_ENVELOPE_VERSION,
    viewer_scope: "anonymous",
    viewer_context: { kind: "anonymous" },
    export: {
      boundary_version: "0.1.0",
      schema_version: "0.1.0",
      template: {
        schema_version: "0.1.0",
        kinds: [{ id: "organization", label: "Institution", color_role: "kind-1" }],
      },
      projection: {
        group_id: "00000000-0000-0000-0000-000000000010",
        revision: 7,
        entities: [{ id: "entity-1", kind: "organization" }],
        edges: [],
        stories: [],
      },
    },
    theme: {
      schema_version: "0.1.0",
      tokens: { "bg.center": { hex: "#0d1017", source: "default" } },
    },
  };
}

describe("snapshot envelope reader", () => {
  it("boots a validated anonymous projection through the state machine", () => {
    const parsed = parseSnapshotEnvelope(JSON.stringify(envelope()));
    const store = createStore(createInitialState(), { devFreeze: false });
    bootSnapshot(store, parsed);

    expect(store.getState().session.loadState).toBe("ready");
    expect(store.getState().session.viewer).toEqual({ kind: "anonymous" });
    expect(store.getState().session.revision).toBe(7);
    expect(store.getState().data.projection?.entities).toHaveLength(1);
  });

  it.each([
    ["envelope", (value: Record<string, unknown>) => { value.schema_version = "1.0.0"; }],
    ["export", (value: Record<string, unknown>) => {
      (value.export as Record<string, unknown>).schema_version = "2.0.0";
    }],
    ["theme", (value: Record<string, unknown>) => {
      (value.theme as Record<string, unknown>).schema_version = "0.2.0";
    }],
  ])("rejects an unsupported %s version loudly", (_label, mutate) => {
    const value = envelope();
    mutate(value);
    expect(() => parseSnapshotEnvelope(JSON.stringify(value))).toThrow(SnapshotReadError);
  });

  it("rejects a viewer scope/context mismatch", () => {
    const value = envelope();
    value.viewer_context = {
      kind: "person",
      person: "00000000-0000-0000-0000-000000000001",
    };
    expect(() => parseSnapshotEnvelope(JSON.stringify(value))).toThrow(
      "viewer_scope and viewer_context disagree",
    );
  });
});

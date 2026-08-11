/**
 * IndexedDB persistence and restore tests (D-080 debt 1).
 *
 * Uses a lightweight IDB mock that stores references without structured
 * cloning (real FSA handles are natively cloneable in browsers, but
 * fake-indexeddb cannot replicate that). The existing intake.test.ts
 * covers the "indexedDB absent" path; this file covers the "indexedDB
 * present" paths: persist-success, restore-none, restore-granted,
 * restore-denied, restore-error.
 */

import { afterEach, beforeEach, describe, expect, it } from "vitest";

import type { Action } from "./actions";
import type { IntakeDirHandle, IntakeFileHandle } from "./intake";
import {
  clearQueueDirectory,
  getQueueDirectory,
  grantQueueDirectory,
  restoreQueueDirectory,
} from "./intake";
import { createInitialState } from "./state";
import { reduce } from "./reducer";

function collect(): { actions: Action[]; dispatch: (a: Action) => void } {
  const actions: Action[] = [];
  return { actions, dispatch: (a) => actions.push(a) };
}

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

type PermissionedDir = IntakeDirHandle & {
  queryPermission?: (opts: { mode: string }) => Promise<string>;
  requestPermission?: (opts: { mode: string }) => Promise<string>;
};

function permissionedDir(opts: {
  queryResult?: string;
  requestResult?: string;
  queryThrows?: boolean;
}): PermissionedDir {
  const base = new FakeDir("intake-queue") as PermissionedDir;
  base.queryPermission = opts.queryThrows
    ? () => Promise.reject(new Error("NotAllowedError"))
    : () => Promise.resolve(opts.queryResult ?? "granted");
  base.requestPermission = () => Promise.resolve(opts.requestResult ?? "denied");
  return base;
}

// --- Lightweight IDB mock (stores references, no structured cloning) ---

type MockStore = Map<IDBValidKey, unknown>;

function installMockIdb(): { data: MockStore; cleanup: () => void } {
  const data: MockStore = new Map();
  const origIdb = (globalThis as Record<string, unknown>)["indexedDB"];

  const fakeDb = {
    createObjectStore: () => {},
    transaction: () => {
      const tx = {
        oncomplete: null as (() => void) | null,
        onerror: null as (() => void) | null,
        objectStore: () => ({
          put: (value: unknown, key: IDBValidKey) => {
            data.set(key, value);
          },
          get: (key: IDBValidKey) => {
            const req = {
              result: data.get(key) ?? null,
              onsuccess: null as (() => void) | null,
              onerror: null as (() => void) | null,
            };
            queueMicrotask(() => req.onsuccess?.());
            return req;
          },
        }),
      };
      queueMicrotask(() => tx.oncomplete?.());
      return tx;
    },
    close: () => {},
  };

  (globalThis as Record<string, unknown>)["indexedDB"] = {
    open: () => {
      const req = {
        result: fakeDb,
        onupgradeneeded: null as (() => void) | null,
        onsuccess: null as (() => void) | null,
        onerror: null as (() => void) | null,
      };
      queueMicrotask(() => {
        req.onupgradeneeded?.();
        req.onsuccess?.();
      });
      return req;
    },
  };

  return {
    data,
    cleanup: () => {
      if (origIdb === undefined) {
        delete (globalThis as Record<string, unknown>)["indexedDB"];
      } else {
        (globalThis as Record<string, unknown>)["indexedDB"] = origIdb;
      }
    },
  };
}

let mockIdb: ReturnType<typeof installMockIdb>;

beforeEach(() => {
  mockIdb = installMockIdb();
});

afterEach(() => {
  const { dispatch } = collect();
  clearQueueDirectory(dispatch);
  mockIdb.cleanup();
});

describe("grantQueueDirectory with IndexedDB present (D-080 debt 1)", () => {
  it("persists the handle and does NOT fire the remember-failure notice", async () => {
    const dir = new FakeDir("intake-queue");
    const { actions, dispatch } = collect();
    expect(await grantQueueDirectory(dir, dispatch)).toBe(true);
    const kinds = actions.map((a) => a.kind);
    expect(kinds).toContain("intakeDirGranted");
    expect(kinds).not.toContain("intakeNoticeAdded");
    expect(mockIdb.data.has("queue-directory")).toBe(true);
    let state = createInitialState();
    for (const action of actions) state = reduce(state, action);
    expect(state.intake.notices).toHaveLength(0);
    expect(state.intake.status).toBe("ready");
  });
});

describe("restoreQueueDirectory (D-080 debt 1)", () => {
  it("returns 'none' when nothing has been persisted", async () => {
    const { dispatch } = collect();
    const outcome = await restoreQueueDirectory(dispatch);
    expect(outcome).toBe("none");
    expect(getQueueDirectory()).toBeNull();
  });

  it("returns 'granted' when a persisted handle has permission", async () => {
    const dir = permissionedDir({ queryResult: "granted" });
    const { dispatch } = collect();
    await grantQueueDirectory(dir, dispatch);
    clearQueueDirectory(dispatch);
    expect(getQueueDirectory()).toBeNull();

    const { actions, dispatch: dispatch2 } = collect();
    const outcome = await restoreQueueDirectory(dispatch2);
    expect(outcome).toBe("granted");
    expect(getQueueDirectory()).not.toBeNull();
    expect(actions.some((a) => a.kind === "intakeDirGranted")).toBe(true);
  });

  it("returns 'failed' when permission is denied on re-request", async () => {
    const dir = permissionedDir({
      queryResult: "prompt",
      requestResult: "denied",
    });
    const { dispatch } = collect();
    await grantQueueDirectory(dir, dispatch);
    clearQueueDirectory(dispatch);

    const { dispatch: dispatch2 } = collect();
    const outcome = await restoreQueueDirectory(dispatch2);
    expect(outcome).toBe("failed");
    expect(getQueueDirectory()).toBeNull();
  });

  it("returns 'failed' when queryPermission throws", async () => {
    const dir = permissionedDir({ queryThrows: true });
    const { dispatch } = collect();
    await grantQueueDirectory(dir, dispatch);
    clearQueueDirectory(dispatch);

    const { dispatch: dispatch2 } = collect();
    const outcome = await restoreQueueDirectory(dispatch2);
    expect(outcome).toBe("failed");
    expect(getQueueDirectory()).toBeNull();
  });
});

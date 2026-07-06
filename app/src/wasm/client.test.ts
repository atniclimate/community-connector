import { describe, expect, it } from "vitest";
import { WasmClient, type WasmTransport, type WorkerMessageEvent } from "./client";
import type { ErrorEnvelopeDto } from "../state/state";
import type { WorkerRequest, WorkerResponse } from "./protocol";

class MockTransport implements WasmTransport {
  public readonly sent: WorkerRequest[] = [];
  private readonly listeners = new Set<(event: WorkerMessageEvent) => void>();

  public postMessage(request: WorkerRequest): void {
    this.sent.push(request);
  }

  public addEventListener(
    type: "message",
    listener: (event: WorkerMessageEvent) => void,
  ): void {
    if (type === "message") {
      this.listeners.add(listener);
    }
  }

  public removeEventListener(
    type: "message",
    listener: (event: WorkerMessageEvent) => void,
  ): void {
    if (type === "message") {
      this.listeners.delete(listener);
    }
  }

  public reply(response: WorkerResponse): void {
    for (const listener of this.listeners) {
      listener({ data: response });
    }
  }
}

describe("WasmClient protocol routing", () => {
  it("routes out-of-order responses to the right promise", async () => {
    const transport = new MockTransport();
    const client = new WasmClient(transport);
    const first = client.search("group-alpha", { kind: "anonymous" }, { text: "alpha" });
    const second = client.queryPaths("group-alpha", { kind: "anonymous" }, { from: "a", to: "b" });

    transport.reply({ correlationId: 2, ok: { id: "second" } });
    transport.reply({ correlationId: 1, ok: { id: "first" } });

    await expect(first).resolves.toEqual({ id: "first" });
    await expect(second).resolves.toEqual({ id: "second" });
  });

  it("rejects error envelopes as typed values", async () => {
    const transport = new MockTransport();
    const client = new WasmClient(transport);
    const error: ErrorEnvelopeDto = {
      code: "NotFound",
      message: "Not found",
    };
    const pending = client.entityDetail("group-alpha", { kind: "anonymous" }, "entity-missing");

    transport.reply({ correlationId: 1, err: error });

    await expect(pending).rejects.toEqual(error);
  });
});

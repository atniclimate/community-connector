import { describe, expect, it } from "vitest";
import { BeatSheetError, validateBeats } from "./beats";

function sheet(): unknown[] {
  return [
    { id: "network-overview", label: "Overview", labelKinds: ["committee", "organization"] },
    {
      id: "one-node",
      label: "One node",
      focusEntityId: "00000000-0000-0000-0000-000000000001",
      camera: "hold",
      labelKinds: ["committee"],
    },
    {
      id: "connectors",
      label: "Connectors",
      measure: "betweenness_top_n",
      topN: 5,
      filter: { kinds: ["person"], edgeKinds: ["connected_to"] },
      labelKinds: [],
    },
  ];
}

describe("beat sheet reader (validateBeats)", () => {
  it("accepts a well-formed sheet and returns typed beats with only known keys", () => {
    const beats = validateBeats([...sheet(), { id: "extra", label: "Extra", unknownKey: 1 }]);

    expect(beats).toHaveLength(4);
    expect(beats[0]).toEqual({ id: "network-overview", label: "Overview", labelKinds: ["committee", "organization"] });
    expect(beats[1]?.camera).toBe("hold");
    expect(beats[2]?.filter).toEqual({ kinds: ["person"], edgeKinds: ["connected_to"] });
    expect(beats[2]?.labelKinds).toEqual([]);
    expect(beats[3]).toEqual({ id: "extra", label: "Extra" });
    expect(validateBeats([])).toEqual([]);
  });

  it.each<[string, unknown, string]>([
    ["a non-array", { id: "x", label: "X" }, "must be a JSON array"],
    ["a non-object beat", ["not a beat"], "Beat 0 is not an object"],
    ["a beat without an id", [{ label: "X" }], "Beat 0 has no string id"],
    ["a beat with a non-string id", [{ id: 3, label: "X" }], "Beat 0 has no string id"],
    ["a beat with an empty id", [{ id: "", label: "X" }], "Beat 0 has no string id"],
    ["a beat without a label", [{ id: "x" }], "Beat x: label must be a string"],
    ["a duplicate id", [{ id: "x", label: "X" }, { id: "x", label: "Y" }], "Beat x: duplicate id"],
    ["an unknown camera", [{ id: "x", label: "X", camera: "zoom" }], 'Beat x: unknown camera "zoom"'],
    ["a non-string camera", [{ id: "x", label: "X", camera: 1 }], "Beat x: unknown camera 1"],
    ["an unknown measure", [{ id: "x", label: "X", measure: "pagerank" }], 'Beat x: unknown measure "pagerank"'],
    ["a non-number topN", [{ id: "x", label: "X", measure: "single_tie", topN: "5" }], "Beat x: topN must be a number"],
    ["a non-string focusEntityId", [{ id: "x", label: "X", focusEntityId: 7 }], "focusEntityId must be a string"],
    ["a non-object filter", [{ id: "x", label: "X", filter: ["person"] }], "Beat x: filter must be an object"],
    ["filter.kinds that is not a string array", [{ id: "x", label: "X", filter: { kinds: "person" } }],
      "filter.kinds must be an array of strings"],
    ["filter.kinds with a non-string item", [{ id: "x", label: "X", filter: { kinds: ["person", 2] } }],
      "filter.kinds must be an array of strings"],
    ["filter.edgeKinds that is not a string array", [{ id: "x", label: "X", filter: { edgeKinds: { a: 1 } } }],
      "filter.edgeKinds must be an array of strings"],
    ["labelKinds that is not a string array", [{ id: "x", label: "X", labelKinds: "committee" }],
      "labelKinds must be an array of strings"],
    ["labelKinds with a non-string item", [{ id: "x", label: "X", labelKinds: [null] }],
      "labelKinds must be an array of strings"],
    ["labelKinds containing person (D-099)", [{ id: "x", label: "X", labelKinds: ["committee", "person"] }],
      "labelKinds may not include person"],
  ])("rejects %s loudly", (_label, input, message) => {
    expect(() => validateBeats(input)).toThrow(BeatSheetError);
    expect(() => validateBeats(input)).toThrow(message);
  });

  it("carries an error envelope so the loader can surface it through errorSurfaced", () => {
    try {
      validateBeats([{ id: "x", label: "X", labelKinds: ["person"] }]);
      expect.unreachable("validateBeats must throw");
    } catch (error) {
      expect(error).toBeInstanceOf(BeatSheetError);
      const envelope = (error as BeatSheetError).envelope;
      expect(envelope.code).toBe("BeatSheetError");
      expect(envelope.message).toContain("Beat x");
    }
  });
});

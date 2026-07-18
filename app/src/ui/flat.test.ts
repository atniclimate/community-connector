import { createRequire } from "node:module";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { EntityDetailDto, ProjectionDto } from "../state/state";
import { projectedEntities } from "../viz/projection";
import { flatRows } from "./flat";
import { detailAttributeViews } from "./format";

const here = path.dirname(fileURLToPath(import.meta.url));
const repo = path.resolve(here, "../../..");
const pkgEntry = path.join(repo, "core/crates/cn-wasm/pkg-node/cn_wasm.js");

describe("flatRows agreement law (P1.7 / D-035)", () => {
  it("lists exactly the entities the 3D scene adapter keeps", () => {
    const projection: ProjectionDto = {
      revision: 1,
      entities: [
        {
          id: "e1",
          kind: "person",
          attributes: { display_name: { type: "text", value: "Ada Example" } },
        },
        { id: "e2", kind: "site", label: "Site Two" },
        // Kindless entity: the scene adapter drops it, so the flat view must too.
        { id: "e3" },
      ],
      edges: [{ id: "ed1", from: "e1", to: "e2" }],
    };

    const rows = flatRows(projection, {});
    const sceneIds = projectedEntities(projection)
      .map((entity) => entity.id)
      .sort();

    expect(rows.map((row) => row.id).sort()).toEqual(sceneIds);
    expect(rows.map((row) => row.id).sort()).toEqual(["e1", "e2"]);
    const byId = new Map(rows.map((row) => [row.id, row]));
    expect(byId.get("e1")?.label).toBe("Ada Example");
    expect(byId.get("e1")?.degree).toBe(1);
    expect(byId.get("e2")?.degree).toBe(1);
  });

  it("renders an empty projection as zero rows", () => {
    expect(flatRows(null, {})).toEqual([]);
  });
});

type CnApiLike = {
  load_group_begin(groupId: string, viewerJson: string, templateJson: string): string;
  load_ops_chunk(groupId: string, opsJsonl: string): string;
  load_group_commit(groupId: string, atMs: number): string;
  projection(groupId: string, viewerJson: string): string;
  search(groupId: string, viewerJson: string, queryJson: string): string;
  entity_detail(groupId: string, viewerJson: string, entityId: string): string;
};

function unwrap<T>(name: string, json: string): T {
  const envelope = JSON.parse(json) as { ok?: T; err?: { code: string; message: string } };
  if (envelope.err !== undefined || envelope.ok === undefined) {
    throw new Error(`${name} failed: ${JSON.stringify(envelope.err ?? "missing ok")}`);
  }
  return envelope.ok;
}

// Uses the node wasm build the smoke script produces. Skipped (visibly) when
// pkg-node is absent; run `npm run smoke:node` once to build it.
describe.skipIf(!existsSync(pkgEntry))(
  "flat projection vs real fixture projection (requires core/crates/cn-wasm/pkg-node)",
  () => {
    const groupId = "00000000-0000-0000-0000-000000000010";
    const viewer = JSON.stringify({
      kind: "person",
      person: "00000000-0000-0000-0000-0000000003e9",
    });

    function loadFixtureProjection(): { api: CnApiLike; projection: ProjectionDto } {
      const require = createRequire(import.meta.url);
      const { CnApi } = require(pkgEntry) as { CnApi: new () => CnApiLike };
      const api = new CnApi();
      const template = readFileSync(
        path.join(repo, "fixtures/templates/research-network.template.json"),
        "utf8",
      );
      const ops = readFileSync(path.join(repo, "fixtures/groups/research-network.ops.jsonl"), "utf8");
      unwrap("load_group_begin", api.load_group_begin(groupId, viewer, template));
      unwrap("load_ops_chunk", api.load_ops_chunk(groupId, ops));
      unwrap("load_group_commit", api.load_group_commit(groupId, Date.now()));
      const projection = unwrap<ProjectionDto>("projection", api.projection(groupId, viewer));
      return { api, projection };
    }

    it("agrees with the viewer projection on entity count and ids", () => {
      const { projection } = loadFixtureProjection();
      const projectionIds = (projection.entities ?? []).map((entity) => entity.id).sort();
      expect(projectionIds.length).toBeGreaterThan(0);

      const rows = flatRows(projection, {});
      const sceneIds = projectedEntities(projection)
        .map((entity) => entity.id)
        .sort();

      expect(rows).toHaveLength(projectionIds.length);
      expect(rows.map((row) => row.id).sort()).toEqual(projectionIds);
      expect(rows.map((row) => row.id).sort()).toEqual(sceneIds);
    });

    it("entity_detail for a fixture entity renders through the detail formatter (P1.5)", () => {
      const { api, projection } = loadFixtureProjection();
      const withAttributes = (projection.entities ?? []).find(
        (entity) =>
          typeof entity.attributes === "object" &&
          entity.attributes !== null &&
          Object.keys(entity.attributes).length > 0,
      );
      if (withAttributes === undefined) {
        throw new Error("fixture projection has no entity with attributes");
      }

      const detail = unwrap<EntityDetailDto>(
        "entity_detail",
        api.entity_detail(groupId, viewer, withAttributes.id),
      );
      const views = detailAttributeViews(detail);

      expect(detail.id).toBe(withAttributes.id);
      expect(views.length).toBeGreaterThan(0);
      for (const view of views) {
        expect(view.label.length).toBeGreaterThan(0);
        expect(typeof view.valueText).toBe("string");
      }
    });

    it("search returns fixture hits scoped to the same projection (P1.4)", () => {
      const { api, projection } = loadFixtureProjection();
      const projectionIds = new Set((projection.entities ?? []).map((entity) => entity.id));
      const hits = unwrap<readonly { entity: string; snippet: string }[]>(
        "search",
        api.search(groupId, viewer, JSON.stringify({ text: "Alder", limit: 25 })),
      );

      expect(hits.length).toBeGreaterThan(0);
      for (const hit of hits) {
        expect(hit.snippet.toLowerCase()).toContain("alder");
        expect(projectionIds.has(hit.entity)).toBe(true);
      }
    });
  },
);

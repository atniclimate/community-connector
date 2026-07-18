import type { Store } from "./store";
import type {
  ErrorEnvelopeDto,
  ProjectionDto,
  ViewerContextDto,
} from "./state";
import { kindMetaFromTemplate } from "./effects";
import { deriveTheme } from "../theme/derive";
import type { GroupTemplateDto, Theme } from "../theme/tokens";

export const SNAPSHOT_ENVELOPE_VERSION = "0.1.0";
const SNAPSHOT_ELEMENT_ID = "cn-snapshot-envelope";
const JSON_SCRIPT_TYPE = "application/json";

type SnapshotProjectionDto = ProjectionDto & {
  readonly group_id: string;
  readonly revision: number;
};

export type SnapshotEnvelopeDto = {
  readonly schema_version: string;
  readonly viewer_scope: "anonymous" | "group-member";
  readonly viewer_context: ViewerContextDto;
  readonly export: {
    readonly boundary_version: string;
    readonly schema_version: string;
    readonly template: GroupTemplateDto;
    readonly projection: SnapshotProjectionDto;
  };
  readonly theme: Theme;
};

export class SnapshotReadError extends Error {
  public readonly envelope: ErrorEnvelopeDto;

  public constructor(message: string) {
    super(message);
    this.name = "SnapshotReadError";
    this.envelope = { code: "SnapshotFormatError", message };
  }
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function requireSupportedVersion(label: string, value: unknown): string {
  if (typeof value !== "string") {
    throw new SnapshotReadError(`${label} schema version is missing`);
  }
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(value);
  if (match === null) {
    throw new SnapshotReadError(`${label} schema version is invalid: ${value}`);
  }
  const major = Number(match[1]);
  const minor = Number(match[2]);
  if (major !== 0 || minor !== 1) {
    throw new SnapshotReadError(`${label} schema version is unsupported: ${value}`);
  }
  return value;
}

function parseViewer(scope: unknown, raw: unknown): ViewerContextDto {
  if (!isObject(raw) || typeof raw.kind !== "string") {
    throw new SnapshotReadError("Snapshot viewer_context is malformed");
  }
  if (scope === "anonymous" && raw.kind === "anonymous") {
    return { kind: "anonymous" };
  }
  if (scope === "group-member" && raw.kind === "person" && typeof raw.person === "string") {
    return { kind: "person", person: raw.person };
  }
  throw new SnapshotReadError("Snapshot viewer_scope and viewer_context disagree");
}

function parseProjection(value: unknown): SnapshotProjectionDto {
  if (
    !isObject(value) ||
    typeof value.group_id !== "string" ||
    typeof value.revision !== "number" ||
    !Array.isArray(value.entities) ||
    !Array.isArray(value.edges)
  ) {
    throw new SnapshotReadError("Snapshot export projection is malformed");
  }
  return value as SnapshotProjectionDto;
}

export function parseSnapshotEnvelope(rawJson: string): SnapshotEnvelopeDto {
  let parsed: unknown;
  try {
    parsed = JSON.parse(rawJson);
  } catch (error) {
    const reason = error instanceof Error ? error.message : "invalid JSON";
    throw new SnapshotReadError(`Snapshot envelope is not valid JSON: ${reason}`);
  }
  if (!isObject(parsed) || !isObject(parsed.export) || !isObject(parsed.theme)) {
    throw new SnapshotReadError("Snapshot envelope is malformed");
  }
  requireSupportedVersion("Snapshot envelope", parsed.schema_version);
  requireSupportedVersion("Snapshot export boundary", parsed.export.boundary_version);
  requireSupportedVersion("Snapshot export", parsed.export.schema_version);
  requireSupportedVersion("Snapshot theme", parsed.theme.schema_version);
  if (!isObject(parsed.export.template)) {
    throw new SnapshotReadError("Snapshot export template is malformed");
  }
  requireSupportedVersion("Snapshot template", parsed.export.template.schema_version);
  if (parsed.viewer_scope !== "anonymous" && parsed.viewer_scope !== "group-member") {
    throw new SnapshotReadError("Snapshot viewer_scope is unsupported");
  }
  const viewer = parseViewer(parsed.viewer_scope, parsed.viewer_context);
  const projection = parseProjection(parsed.export.projection);
  if (!isObject(parsed.theme.tokens) || Object.keys(parsed.theme.tokens).length === 0) {
    throw new SnapshotReadError("Snapshot theme tokens are malformed");
  }
  return {
    schema_version: parsed.schema_version as string,
    viewer_scope: parsed.viewer_scope,
    viewer_context: viewer,
    export: {
      boundary_version: parsed.export.boundary_version as string,
      schema_version: parsed.export.schema_version as string,
      template: parsed.export.template as unknown as GroupTemplateDto,
      projection,
    },
    theme: parsed.theme as unknown as Theme,
  };
}

export function readSnapshotEnvelope(documentRoot: Document): SnapshotEnvelopeDto {
  const element = documentRoot.getElementById(SNAPSHOT_ELEMENT_ID);
  if (element === null) {
    throw new SnapshotReadError(`Missing #${SNAPSHOT_ELEMENT_ID} snapshot envelope`);
  }
  if (element.getAttribute("type") !== JSON_SCRIPT_TYPE) {
    throw new SnapshotReadError(`#${SNAPSHOT_ELEMENT_ID} must use type=${JSON_SCRIPT_TYPE}`);
  }
  return parseSnapshotEnvelope(element.textContent ?? "");
}

/** Boots a baked projection exclusively through state-machine actions (I4). */
export function bootSnapshot(store: Store, envelope: SnapshotEnvelopeDto): void {
  const groupId = envelope.export.projection.group_id;
  store.dispatch({ kind: "groupLoadRequested", groupId, viewer: envelope.viewer_context });
  const derived = deriveTheme(envelope.export.template);
  store.dispatch({
    kind: "themeDerived",
    theme: envelope.theme,
    report: derived.report,
    kindMeta: kindMetaFromTemplate(envelope.export.template),
  });
  store.dispatch({
    kind: "projectionReceived",
    projection: envelope.export.projection,
    revision: envelope.export.projection.revision,
  });
  store.dispatch({ kind: "groupLoadSucceeded", groupId });
}

export function snapshotError(error: unknown): ErrorEnvelopeDto {
  if (error instanceof SnapshotReadError) {
    return error.envelope;
  }
  return {
    code: "SnapshotFormatError",
    message: error instanceof Error ? error.message : "Unknown snapshot boot error",
  };
}

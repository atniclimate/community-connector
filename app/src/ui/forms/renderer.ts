/**
 * P3.6 template-driven entry form (docs/blueprints/intake-pipeline.md
 * section 4): kind picker -> typed field widgets from the group template,
 * advisory validation, the consent statement panel (DRAFT boilerplate,
 * D-072.1), and the structural consent checkbox gate (D-030: unchecked
 * means nothing stages).
 *
 * The component renders and assembles the inner payload; it stages
 * nothing itself - the caller (the P3.5 wizard, via app/src/state) owns
 * staging, so all mutation stays in the state layer (I4). Accessibility
 * per R9: every widget labeled, keyboard navigable, error text linked via
 * aria-describedby, layout holds at 375px.
 */

import type { JsonObject } from "../../state/state";
import { el, uiId } from "../dom";
import {
  CONSENT_AFFIRMATION,
  CONSENT_DRAFT_BANNER,
  CONSENT_PARAGRAPHS,
  FORM_VERSION,
  consentTextDigest,
} from "./consent";
import type { FormAttr, FormKind, RawFieldValue } from "./model";
import { advisoryIssues, buildPayload, canSubmit, formModel } from "./model";

export type EntryFormDeps = {
  /** Parsed group-template JSON (the R2 schema). */
  readonly template: JsonObject;
  /** Receives the assembled inner payload when the form submits. */
  readonly onStage: (payload: JsonObject) => void;
  /** Injectable clock (defaults to Date.now). */
  readonly now?: () => number;
  /** Injectable submission-id source (defaults to crypto.randomUUID). */
  readonly newSubmissionId?: () => string;
};

type FieldControl = {
  readonly attr: FormAttr;
  readonly read: () => RawFieldValue;
  readonly errorElement: HTMLElement;
  readonly input: HTMLElement;
};

export function mountEntryForm(container: HTMLElement, deps: EntryFormDeps): () => void {
  const model = formModel(deps.template);
  const now = deps.now ?? (() => Date.now());
  const newSubmissionId = deps.newSubmissionId ?? (() => globalThis.crypto.randomUUID());

  const form = el("form", { className: "cn-entry-form", attrs: { novalidate: "" } });
  const fieldsRegion = el("div", { className: "cn-entry-form-fields" });
  let controls: FieldControl[] = [];

  // Kind picker (R2): one entry form per template kind.
  const kindId = uiId("cn-form-kind");
  const kindSelect = el("select", { attrs: { id: kindId } });
  for (const kind of model.kinds) {
    kindSelect.append(el("option", { text: kind.label, attrs: { value: kind.id } }));
  }
  const kindRow = el("div", { className: "cn-entry-form-row" }, [
    el("label", { text: "What are you adding?", attrs: { for: kindId } }),
    kindSelect,
  ]);

  // Consent panel: DRAFT banner + statement + the structural checkbox gate.
  const consentCheckbox = el("input", {
    attrs: { type: "checkbox", id: uiId("cn-form-consent") },
  });
  const consentPanel = el("fieldset", { className: "cn-entry-form-consent" }, [
    el("legend", { text: "Before you send this" }),
    el("p", { className: "cn-entry-form-draft-banner", text: CONSENT_DRAFT_BANNER }),
    ...CONSENT_PARAGRAPHS.map(([lead, body]) =>
      el("p", {}, [el("strong", { text: `${lead} ` }), body]),
    ),
    el("div", { className: "cn-entry-form-affirm" }, [
      consentCheckbox,
      el("label", {
        text: CONSENT_AFFIRMATION,
        attrs: { for: consentCheckbox.getAttribute("id") ?? "" },
      }),
    ]),
  ]);

  const submit = el("button", {
    className: "cn-entry-form-submit",
    text: "Stage for review",
    attrs: { type: "submit" },
  });
  submit.disabled = true;

  function currentKind(): FormKind | undefined {
    return model.kinds.find((kind) => kind.id === kindSelect.value);
  }

  function readFields(): Record<string, RawFieldValue> {
    const raw: Record<string, RawFieldValue> = {};
    for (const control of controls) {
      raw[control.attr.id] = control.read();
    }
    return raw;
  }

  function refreshAdvisories(): void {
    const kind = currentKind();
    for (const control of controls) {
      const issues = advisoryIssues(control.attr, control.read());
      control.errorElement.textContent = issues.join(" ");
      control.input.setAttribute("aria-invalid", issues.length > 0 ? "true" : "false");
    }
    submit.disabled =
      kind === undefined ||
      !canSubmit(kind.attributes, readFields(), consentCheckbox.checked);
  }

  function widgetFor(attr: FormAttr): FieldControl {
    const inputId = uiId("cn-form-field");
    const errorId = `${inputId}-error`;
    const errorElement = el("p", {
      className: "cn-entry-form-error",
      attrs: { id: errorId, role: "status" },
    });
    const baseAttrs: Record<string, string> = {
      id: inputId,
      "aria-describedby": errorId,
    };
    if (attr.required) {
      baseAttrs["aria-required"] = "true";
    }

    let input: HTMLElement;
    let read: () => RawFieldValue;
    switch (attr.attrType) {
      case "enum": {
        const select = el("select", { attrs: baseAttrs });
        select.append(el("option", { text: "(choose)", attrs: { value: "" } }));
        for (const value of attr.values) {
          select.append(el("option", { text: value, attrs: { value } }));
        }
        input = select;
        read = () => select.value;
        break;
      }
      case "tags": {
        const area = el("textarea", {
          attrs: { ...baseAttrs, rows: "3", placeholder: "One per line" },
        });
        input = area;
        read = () => area.value;
        break;
      }
      case "number": {
        const field = el("input", { attrs: { ...baseAttrs, type: "text", inputmode: "decimal" } });
        input = field;
        read = () => field.value;
        break;
      }
      case "date": {
        const field = el("input", { attrs: { ...baseAttrs, type: "date" } });
        input = field;
        read = () => field.value;
        break;
      }
      case "geo": {
        const field = el("input", {
          attrs: { ...baseAttrs, type: "text", placeholder: "latitude, longitude" },
        });
        input = field;
        read = () => field.value;
        break;
      }
      default: {
        // text, link, media ref: plain text input (media is an opaque
        // string for v0.1; link format is advisory only).
        const field = el("input", { attrs: { ...baseAttrs, type: "text" } });
        input = field;
        read = () => field.value;
        break;
      }
    }
    input.addEventListener("input", refreshAdvisories);

    const labelText = attr.required ? `${attr.id} (required)` : attr.id;
    const row = el("div", { className: "cn-entry-form-row" }, [
      el("label", { text: labelText, attrs: { for: inputId } }),
      input,
      errorElement,
    ]);
    if (attr.defaultVisibility !== null) {
      row.append(
        el("p", {
          className: "cn-entry-form-visibility",
          text: `Shared at: ${attr.defaultVisibility} (pilot entries are T1)`,
        }),
      );
    }
    fieldsRegion.append(row);
    return { attr, read, errorElement, input };
  }

  function renderFields(): void {
    fieldsRegion.replaceChildren();
    controls = [];
    const kind = currentKind();
    if (kind === undefined) {
      return;
    }
    controls = kind.attributes.map(widgetFor);
    refreshAdvisories();
  }

  kindSelect.addEventListener("change", renderFields);
  consentCheckbox.addEventListener("change", refreshAdvisories);
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    const kind = currentKind();
    if (kind === undefined || submit.disabled) {
      return;
    }
    void consentTextDigest().then((digest) => {
      // The checkbox gate is structural (D-030): this path is unreachable
      // with the box unchecked because submit stays disabled.
      const payload = buildPayload({
        kind: kind.id,
        fields: readFields(),
        attributes: kind.attributes,
        consent: {
          consent_text_digest: digest,
          consent_affirmed: consentCheckbox.checked,
          consent_affirmed_at: now(),
        },
        submissionId: newSubmissionId(),
        formVersion: FORM_VERSION,
        capturedAtMs: now(),
      });
      deps.onStage(payload);
    });
  });

  form.append(kindRow, fieldsRegion, consentPanel, submit);
  container.append(form);
  renderFields();

  return () => {
    form.remove();
  };
}

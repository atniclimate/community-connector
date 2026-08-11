/**
 * DOM for the remote intake form (blueprint 4.1; R9/I9 accessibility). Renders
 * a consent-gated, template-driven form; on submit it builds an InnerPayload,
 * seals it once to the baked-in facilitator public key, wraps it in an
 * OuterEnvelope, and POSTs to the relay - classifying the response into
 * distinct, honest screens.
 *
 * Accessibility mirrors app/src/ui/forms/renderer.ts: every input has a
 * <label for>, required fields get aria-required, per-field error text is
 * linked via aria-describedby and toggles aria-invalid, and every widget is a
 * native keyboard-operable control. Layout holds at 375px (style.css). All
 * styling is class-based - no inline styles - so the CSP needs no
 * style 'unsafe-inline'.
 *
 * This module touches `document`; it is never imported by the node-env unit
 * tests (which cover the pure modules it composes).
 */
import type { JsonObject } from "./json";
import {
  CONSENT_AFFIRMATION,
  CONSENT_DRAFT_BANNER,
  CONSENT_PARAGRAPHS,
  consentTextDigest,
} from "./consent";
import { buildInnerPayload, buildOuterEnvelope, type OuterEnvelope } from "./envelope";
import {
  advisoryIssues,
  buildFields,
  canSubmit,
  formModel,
  type FormAttr,
  type FormKind,
  type RawFieldValue,
} from "./model";
import { postEnvelope, type SubmitOutcome } from "./submit";

export type FormDeps = {
  /** Parsed group-template JSON (the R2 schema baked in at build time). */
  readonly template: JsonObject;
  /** Recipient X25519 public key bytes (decoded from config hex). */
  readonly publicKey: Uint8Array;
  /** Recipient key fingerprint, rendered in the footer and outer envelope. */
  readonly fingerprint: string;
  /** Optional friendly labels keyed by attribute id; falls back to raw id. */
  readonly friendlyLabels?: Readonly<Record<string, string>>;
  /** Relay origin to POST to. */
  readonly relayOrigin: string;
  /** InnerPayload.form_version. */
  readonly formVersion: string;
  /** Injectable submission-id source (defaults to crypto.randomUUID). */
  readonly newSubmissionId?: () => string;
  /** Injectable transport (defaults to the real postEnvelope). */
  readonly postEnvelopeImpl?: (
    relayOrigin: string,
    outer: OuterEnvelope,
  ) => Promise<SubmitOutcome>;
};

type ElOptions = {
  readonly className?: string;
  readonly text?: string;
  readonly attrs?: Readonly<Record<string, string>>;
};

function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  options: ElOptions = {},
  children: readonly (HTMLElement | string)[] = [],
): HTMLElementTagNameMap[K] {
  const element = document.createElement(tag);
  if (options.className !== undefined) {
    element.className = options.className;
  }
  if (options.text !== undefined) {
    element.textContent = options.text;
  }
  if (options.attrs !== undefined) {
    for (const [name, value] of Object.entries(options.attrs)) {
      element.setAttribute(name, value);
    }
  }
  element.append(...children);
  return element;
}

let nextUiId = 0;
function uiId(prefix: string): string {
  nextUiId += 1;
  return `${prefix}-${nextUiId}`;
}

type FieldControl = {
  readonly attr: FormAttr;
  readonly read: () => RawFieldValue;
  readonly errorElement: HTMLElement;
  readonly input: HTMLElement;
};

/** Mounts the intake form into `container`, replacing its contents. */
export function mountForm(container: HTMLElement, deps: FormDeps): void {
  const model = formModel(deps.template);
  const friendlyLabels = deps.friendlyLabels ?? {};
  const newSubmissionId = deps.newSubmissionId ?? (() => globalThis.crypto.randomUUID());
  const postImpl = deps.postEnvelopeImpl ?? postEnvelope;

  container.replaceChildren();

  const form = el("form", { className: "cn-form", attrs: { novalidate: "" } });
  const fieldsRegion = el("div", { className: "cn-form-fields" });
  let controls: FieldControl[] = [];

  const heading = el("h1", { className: "cn-form-title", text: "Join the network map" });
  const draftBanner = el("p", {
    className: "cn-form-draft-banner",
    text: CONSENT_DRAFT_BANNER,
    attrs: { role: "note" },
  });

  // Kind picker (R2). Hidden when the template has exactly one kind.
  const kindId = uiId("cn-form-kind");
  const kindSelect = el("select", { attrs: { id: kindId } });
  for (const kind of model.kinds) {
    kindSelect.append(el("option", { text: kind.label, attrs: { value: kind.id } }));
  }
  const kindRow = el("div", { className: "cn-form-row" }, [
    el("label", { text: "What are you adding?", attrs: { for: kindId } }),
    kindSelect,
  ]);
  const singleKind = model.kinds.length <= 1;
  if (singleKind) {
    kindRow.classList.add("cn-hidden");
  }

  // Consent panel: DRAFT banner + statement + the structural checkbox gate.
  const consentCheckbox = el("input", {
    attrs: { type: "checkbox", id: uiId("cn-form-consent") },
  });
  const consentPanel = el("fieldset", { className: "cn-form-consent" }, [
    el("legend", { text: "Before you send this" }),
    ...CONSENT_PARAGRAPHS.map(([lead, body]) =>
      el("p", {}, [el("strong", { text: `${lead} ` }), body]),
    ),
    el("div", { className: "cn-form-affirm" }, [
      consentCheckbox,
      el("label", {
        text: CONSENT_AFFIRMATION,
        attrs: { for: consentCheckbox.getAttribute("id") ?? "" },
      }),
    ]),
  ]);

  const submit = el("button", {
    className: "cn-form-submit",
    text: "Send to the facilitator",
    attrs: { type: "submit" },
  });
  submit.disabled = true;

  // Live region for submission progress / errors (assertive - it reports the
  // result of an explicit action).
  const status = el("div", {
    className: "cn-form-status",
    attrs: { role: "alert", "aria-live": "assertive" },
  });

  // Fingerprint footer for out-of-band verification (ceremony design section 7).
  const footer = el("footer", { className: "cn-form-footer" }, [
    el("span", { text: `intake key: ${deps.fingerprint}` }),
  ]);

  function currentKind(): FormKind | undefined {
    if (singleKind) {
      return model.kinds[0];
    }
    return model.kinds.find((kind) => kind.id === kindSelect.value);
  }

  function labelFor(attr: FormAttr): string {
    const friendly = friendlyLabels[attr.id] ?? attr.id;
    return attr.required ? `${friendly} (required)` : friendly;
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
      className: "cn-form-error",
      attrs: { id: errorId, role: "status" },
    });
    const baseAttrs: Record<string, string> = { id: inputId, "aria-describedby": errorId };
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
        // text, link: plain text input (link format is advisory only).
        const field = el("input", { attrs: { ...baseAttrs, type: "text" } });
        input = field;
        read = () => field.value;
        break;
      }
    }
    input.addEventListener("input", refreshAdvisories);

    const row = el("div", { className: "cn-form-row" }, [
      el("label", { text: labelFor(attr), attrs: { for: inputId } }),
      input,
      errorElement,
    ]);
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

  // --- Submission ----------------------------------------------------------

  function setStatus(children: readonly (HTMLElement | string)[]): void {
    status.replaceChildren(...children);
  }

  function retryButton(label: string, onClick: () => void): HTMLElement {
    const button = el("button", {
      className: "cn-form-retry",
      text: label,
      attrs: { type: "button" },
    });
    button.addEventListener("click", onClick);
    return button;
  }

  function showConfirmation(): void {
    // Correction 4 (consent-draft section 7 item 4; ADR-005 D6): a successful
    // POST means the relay ACCEPTED the sealed envelope for delivery - it does
    // NOT mean the facilitator has it yet. Do not overclaim.
    const panel = el("section", { className: "cn-confirmation", attrs: { role: "status" } }, [
      el("h1", { text: "Thank you" }),
      el("p", {
        text: "Your sealed answers were accepted for delivery to the facilitator.",
      }),
      el("p", {
        className: "cn-form-draft-banner",
        text: CONSENT_DRAFT_BANNER,
        attrs: { role: "note" },
      }),
    ]);
    // Start over = a genuinely NEW submission (fresh submission_id + fresh seal).
    panel.append(
      retryButton("Add another response", () => {
        mountForm(container, deps);
      }),
    );
    container.replaceChildren(panel);
  }

  async function attempt(outer: OuterEnvelope): Promise<void> {
    setStatus(["Sending your sealed answers..."]);
    const outcome = await postImpl(deps.relayOrigin, outer);
    switch (outcome.kind) {
      case "accepted":
        showConfirmation();
        return;
      case "form_out_of_date":
        // Distinct message: retrying the same stale envelope can never succeed.
        setStatus([
          el("p", {
            text:
              "This form is out of date and needs to be reloaded before you can " +
              "send. Please reload the page (or rescan the code) and try again.",
          }),
        ]);
        submit.disabled = true;
        return;
      case "rate_limited": {
        const wait =
          outcome.retryAfter !== null
            ? ` Please wait ${outcome.retryAfter} seconds and try again.`
            : " Please wait a moment and try again.";
        setStatus([
          el("p", { text: `Too many submissions right now.${wait}` }),
          retryButton("Try again", () => void attempt(outer)),
        ]);
        return;
      }
      case "unavailable":
        setStatus([
          el("p", { text: "The service is temporarily unavailable. Please try again shortly." }),
          retryButton("Try again", () => void attempt(outer)),
        ]);
        return;
      case "network_error":
        setStatus([
          el("p", {
            text:
              "Could not reach the service - your answers were not sent. Check your " +
              "connection and try again.",
          }),
          retryButton("Try again", () => void attempt(outer)),
        ]);
        return;
      case "error":
        setStatus([
          el("p", { text: `Something went wrong (code ${outcome.status}). Your answers were not sent.` }),
          retryButton("Try again", () => void attempt(outer)),
        ]);
        return;
    }
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    const kind = currentKind();
    if (kind === undefined || submit.disabled) {
      return;
    }
    submit.disabled = true;
    setStatus(["Sealing your answers on this device..."]);
    void consentTextDigest()
      .then((digest) => {
        const inner = buildInnerPayload({
          submissionId: newSubmissionId(),
          formVersion: deps.formVersion,
          kind: kind.id,
          fields: buildFields(kind.attributes, readFields()),
          consentTextDigest: digest,
          // The checkbox gate is structural (D-030); submit stays disabled with
          // it unchecked, so this is always true here. The core re-checks.
          consentAffirmed: consentCheckbox.checked,
        });
        // Seal exactly once; retries re-POST this same object (no reseal).
        const outer = buildOuterEnvelope(inner, deps.publicKey, deps.fingerprint);
        return attempt(outer);
      })
      .catch(() => {
        setStatus([
          el("p", { text: "Could not prepare your answers on this device. Please reload and try again." }),
        ]);
      });
  });

  kindSelect.addEventListener("change", renderFields);
  consentCheckbox.addEventListener("change", refreshAdvisories);

  form.append(heading, draftBanner, kindRow, fieldsRegion, consentPanel, submit, status);
  container.append(form, footer);
  renderFields();
}

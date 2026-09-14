/**
 * DOM for the remote intake form (blueprint 4.1; R9/I9 accessibility). Renders
 * a consent-gated, template-driven form; on submit it builds an InnerPayload,
 * seals it once to the baked-in facilitator public key, wraps it in an
 * OuterEnvelope, and POSTs to the relay - classifying the response into
 * distinct, honest screens.
 *
 * Accessibility mirrors app/src/ui/forms/renderer.ts: every input has a
 * <label for>, required fields get aria-required, per-field help and error
 * text are linked via aria-describedby and errors toggle aria-invalid, and
 * every widget is a native keyboard-operable control. Layout holds at 375px
 * (style.css). All styling is class-based - no inline styles - so the CSP
 * needs no style 'unsafe-inline'.
 *
 * Community-facing strings follow the ATNI house voice (institutional third
 * person, Title Case headings and buttons, sentence case elsewhere, no em
 * dashes, no exclamation points) and remain DRAFT pending D-023.
 *
 * This module touches `document`; it is never imported by the node-env unit
 * tests (which cover the pure modules it composes).
 */
import type { JsonObject } from "./json";
import { restrictKinds } from "./config";
import {
  CONSENT_AFFIRMATION,
  CONSENT_DRAFT_BANNER,
  CONSENT_HEADING,
  CONSENT_PARAGRAPHS,
  consentTextDigest,
} from "./consent";
import { buildInnerPayload, buildOuterEnvelope, type OuterEnvelope } from "./envelope";
import {
  REQUIRED_FIELD_MESSAGE,
  advisoryIssues,
  buildFields,
  canSubmit,
  formModel,
  shouldShowRequiredError,
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
  /** Optional caption help text keyed by attribute id. */
  readonly fieldHelp?: Readonly<Record<string, string>>;
  /**
   * Kind ids the form offers (CN_FORM_KINDS). Empty or absent = every kind in
   * the template. With exactly one kind the selector is not rendered at all.
   */
  readonly allowedKinds?: readonly string[];
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
  /** Has this field been blurred after focus, or edited, at least once? */
  readonly isTouched: () => boolean;
};

function draftTag(): HTMLElement {
  return el("p", {
    className: "cn-form-draft-banner",
    text: CONSENT_DRAFT_BANNER,
    attrs: { role: "note" },
  });
}

/** Mounts the intake form into `container`, replacing its contents. */
export function mountForm(container: HTMLElement, deps: FormDeps): void {
  const model = formModel(deps.template);
  const kinds = restrictKinds(model.kinds, deps.allowedKinds ?? []);
  const friendlyLabels = deps.friendlyLabels ?? {};
  const fieldHelp = deps.fieldHelp ?? {};
  const newSubmissionId = deps.newSubmissionId ?? (() => globalThis.crypto.randomUUID());
  const postImpl = deps.postEnvelopeImpl ?? postEnvelope;

  container.replaceChildren();

  const form = el("form", { className: "cn-form", attrs: { novalidate: "" } });
  const fieldsRegion = el("div", { className: "cn-form-fields" });
  let controls: FieldControl[] = [];
  // A required-field error is never shown on first paint (I9): only once the
  // visitor has touched that field, or attempted to submit the form.
  let submitted = false;

  const heading = el("h1", { className: "cn-form-title", text: "Add Yourself to the Network Map" });
  const intro = el("p", {
    className: "cn-form-intro",
    text:
      "Community Connector maps the people, committees, and organizations at work " +
      "across the Affiliated Tribes of Northwest Indians (ATNI) so that relatives " +
      "pursuing the same priorities can find one another.",
  });
  const draftBanner = draftTag();

  // Kind picker (R2). Not rendered at all when exactly one kind is offered
  // (a single-kind template, or a CN_FORM_KINDS build restriction).
  const kindId = uiId("cn-form-kind");
  const kindSelect = el("select", { attrs: { id: kindId } });
  for (const kind of kinds) {
    kindSelect.append(el("option", { text: kind.label, attrs: { value: kind.id } }));
  }
  const kindRow = el("div", { className: "cn-form-row" }, [
    el("label", { text: "What are you adding?", attrs: { for: kindId } }),
    kindSelect,
  ]);
  const singleKind = kinds.length <= 1;

  // Consent card: heading + statement (square-bulleted, bold lead-in + colon)
  // + the structural checkbox gate.
  const consentCheckbox = el("input", {
    attrs: { type: "checkbox", id: uiId("cn-form-consent") },
  });
  const consentPanel = el("fieldset", { className: "cn-form-consent" }, [
    el("legend", { text: CONSENT_HEADING }),
    el(
      "ul",
      { className: "cn-form-consent-list", attrs: { role: "list" } },
      CONSENT_PARAGRAPHS.map(([lead, body]) =>
        el("li", {}, [el("strong", { text: `${lead} ` }), body]),
      ),
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
    text: "Send to the Facilitator",
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
      return kinds[0];
    }
    return kinds.find((kind) => kind.id === kindSelect.value);
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
      const value = control.read();
      const issues = advisoryIssues(control.attr, value).filter((issue) => {
        if (issue !== REQUIRED_FIELD_MESSAGE) {
          return true;
        }
        return shouldShowRequiredError({
          touched: control.isTouched(),
          submitted,
          value,
          required: control.attr.required,
        });
      });
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
    const helpId = `${inputId}-help`;
    const helpText = fieldHelp[attr.id];
    const errorElement = el("p", {
      className: "cn-form-error",
      attrs: { id: errorId, role: "status" },
    });
    const describedBy = helpText !== undefined ? `${helpId} ${errorId}` : errorId;
    const baseAttrs: Record<string, string> = { id: inputId, "aria-describedby": describedBy };
    if (attr.required) {
      baseAttrs["aria-required"] = "true";
    }

    let input: HTMLElement;
    let read: () => RawFieldValue;
    switch (attr.attrType) {
      case "enum": {
        const select = el("select", { attrs: baseAttrs });
        select.append(el("option", { text: "Choose one", attrs: { value: "" } }));
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
    let touched = false;
    function markTouched(): void {
      touched = true;
      refreshAdvisories();
    }
    // "Touched" per the fix spec: any input, or a blur after focus (covers a
    // visitor who tabs through a required field without typing anything).
    input.addEventListener("input", markTouched);
    input.addEventListener("blur", markTouched);

    const rowChildren: HTMLElement[] = [
      el("label", { text: labelFor(attr), attrs: { for: inputId } }),
    ];
    if (helpText !== undefined) {
      rowChildren.push(el("p", { className: "cn-form-help", text: helpText, attrs: { id: helpId } }));
    }
    rowChildren.push(input, errorElement);
    const row = el("div", { className: "cn-form-row" }, rowChildren);
    fieldsRegion.append(row);
    return { attr, read, errorElement, input, isTouched: () => touched };
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
      el("h1", { className: "cn-form-title", text: "Thank You" }),
      el("p", {
        text:
          "Your sealed answers were accepted for delivery to the facilitator. " +
          "Nothing appears in the network until the facilitator has read and approved them.",
      }),
      draftTag(),
    ]);
    // Start over = a genuinely NEW submission (fresh submission_id + fresh seal).
    panel.append(
      retryButton("Add Another Response", () => {
        mountForm(container, deps);
      }),
    );
    container.replaceChildren(panel);
  }

  async function attempt(outer: OuterEnvelope): Promise<void> {
    setStatus(["Sending your sealed answers."]);
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
              "This form is out of date and cannot send. Reload the page, or scan " +
              "the code again, and then try once more.",
          }),
        ]);
        submit.disabled = true;
        return;
      case "rate_limited": {
        const wait =
          outcome.retryAfter !== null
            ? ` Wait ${outcome.retryAfter} seconds and try again.`
            : " Wait a moment and try again.";
        setStatus([
          el("p", { text: `Too many submissions are arriving right now.${wait}` }),
          retryButton("Try Again", () => void attempt(outer)),
        ]);
        return;
      }
      case "unavailable":
        setStatus([
          el("p", { text: "The service is temporarily unavailable. Try again shortly." }),
          retryButton("Try Again", () => void attempt(outer)),
        ]);
        return;
      case "network_error":
        setStatus([
          el("p", {
            text:
              "The service could not be reached, so your answers were not sent. " +
              "Check your connection and try again.",
          }),
          retryButton("Try Again", () => void attempt(outer)),
        ]);
        return;
      case "error":
        setStatus([
          el("p", { text: `Something went wrong (code ${outcome.status}). Your answers were not sent.` }),
          retryButton("Try Again", () => void attempt(outer)),
        ]);
        return;
    }
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault();
    submitted = true;
    refreshAdvisories();
    const kind = currentKind();
    if (kind === undefined || submit.disabled) {
      return;
    }
    submit.disabled = true;
    setStatus(["Sealing your answers on this device."]);
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
          el("p", {
            text: "Your answers could not be prepared on this device. Reload the page and try again.",
          }),
        ]);
      });
  });

  kindSelect.addEventListener("change", renderFields);
  consentCheckbox.addEventListener("change", refreshAdvisories);

  form.append(heading, intro, draftBanner);
  if (!singleKind) {
    form.append(kindRow);
  }
  form.append(fieldsRegion, consentPanel, submit, status);
  container.append(form, footer);
  renderFields();
}

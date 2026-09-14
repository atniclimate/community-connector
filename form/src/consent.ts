/**
 * Consent statement boilerplate for the REMOTE (attendee phone) intake form.
 *
 * DRAFT TEXT - not yet approved. Draft v2 (2026-09-14): the wording follows
 * the ATNI house voice (institutional third person, bold lead-in + colon, no
 * em dashes, Oxford comma) and is recorded verbatim in
 * docs/design/intake-consent-text-draft-2026-09-14.md, which supersedes the
 * 2026-07-24 draft. It keeps every truthfulness correction that draft's
 * section 7 recorded from the ADR-005 round:
 *  - the collection claim is scoped to what is TYPED (the relay operator
 *    necessarily observes traffic metadata - correction 1);
 *  - the sealed-phone paragraph says the key is USED only on the facilitator's
 *    computer, NOT that it is the only copy (the ceremony creates offline
 *    recovery copies - correction 2);
 *  - the removal promise is worded as "no longer be shown" (append-only log;
 *    the no-longer-shown vs true-erasure decision is still the human's - matches
 *    the in-app precedent for product-wide consistency - correction 3).
 * Correction 4 (confirmation-screen wording) lives in the confirmation screen
 * (render.ts), not here.
 *
 * This is the REMOTE path, so - unlike the in-app version
 * (app/src/ui/forms/consent.ts, not updated here) - it INCLUDES the
 * phone-encryption paragraph (this form runs on an attendee's phone, not the
 * facilitator's PC).
 *
 * D-023 human review remains the bar before ANY community-facing use; the UI
 * shows the DRAFT tag until that sign-off lands. Placeholders stay
 * [BRACKETED] and must never be filled with a real person's contact
 * information in this repository (I1).
 *
 * This module imports nothing so the D8 manifest generator can import
 * `consentTextDigest` directly (single source of truth for the digest, no
 * second implementation to drift). It uses only globalThis.crypto.subtle and
 * TextEncoder, both built into the pinned Node 24 runtime and every browser.
 */

/** Rendered as a status tag above the form until D-023 sign-off. */
export const CONSENT_DRAFT_BANNER =
  "DRAFT wording, pending human review (D-023). Not for community use.";

/** The consent block heading (Title Case per house style). */
export const CONSENT_HEADING = "Before You Send This";

/**
 * The consent statement, in display order: [bold lead-in, body]. The renderer
 * shows each as a square-bulleted item with the lead-in in bold.
 */
export const CONSENT_PARAGRAPHS: readonly (readonly [string, string])[] = [
  [
    "What the form keeps:",
    "only what is typed here. Nothing else is gathered from you or your device.",
  ],
  [
    "A person reviews it first:",
    "every entry goes to the network facilitator, a person working for the " +
      "ATNI Climate program. Nothing appears in the network until the " +
      "facilitator has read and approved it; anything that looks off is set " +
      "aside rather than published.",
  ],
  [
    "How it is classified:",
    "everything entered here is held at Tier 1 (T1) of the Tiered Sovereign " +
      "Data Framework, shared within the network and governed by the " +
      "community. Under the framework, ATNI Climate decides how it is " +
      "classified and used; that decision never belongs to a company or to a " +
      "server.",
  ],
  [
    "Taking part is your choice:",
    "every question except your name is optional. You can stop at any time " +
      "before sending, and nothing is kept. After sending, you can ask to be " +
      "removed at any time, and your information will no longer be shown in " +
      "the network.",
  ],
  [
    "To be removed or to ask a question:",
    "contact [REMOVAL CONTACT, set at deployment; never a real name or " +
      "address in this repository].",
  ],
  [
    "Your answers are sealed on your phone:",
    "your answers are locked (encrypted) on your own phone before they are " +
      "sent. The internet services that carry the message cannot read them; " +
      "the key that opens them is used only on the facilitator's computer.",
  ],
];

/** The affirmation the checkbox asserts (the D-030 consent instrument). */
export const CONSENT_AFFIRMATION =
  "I understand, and I agree to be included in the network at the level " +
  "described above.";

/**
 * The canonical consent text the payload digest covers: banner and heading
 * excluded (they are UI chrome, not consented content), paragraphs and
 * affirmation joined in display order. Same shape as
 * app/src/ui/forms/consent.ts `consentText`.
 */
export function consentText(): string {
  const blocks = CONSENT_PARAGRAPHS.map(([lead, body]) => `${lead} ${body}`);
  blocks.push(CONSENT_AFFIRMATION);
  return blocks.join("\n\n");
}

/**
 * SHA-256 (lowercase hex) of the consent text - InnerPayload.consent's
 * consent_text_digest, proving WHICH wording was affirmed (ADR-005 D5). Uses
 * globalThis.crypto.subtle, exactly as the in-app path does. The puller
 * compares this as a plain string (cn-ingest consent.rs), so self-consistency
 * is what matters, not matching any particular external value.
 */
export async function consentTextDigest(): Promise<string> {
  const bytes = new TextEncoder().encode(consentText());
  const hash = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(hash))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

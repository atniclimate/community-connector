/**
 * Consent statement boilerplate for the in-app (facilitator-entered) form.
 *
 * DRAFT TEXT - authorized as functionality-matching boilerplate by the
 * human (DECISIONS.md D-072.1) and derived from
 * docs/design/intake-consent-text-draft-2026-07-24.md with that file's
 * section-7 truthfulness corrections applied:
 *  - the collection claim is scoped to "only what you typed";
 *  - the removal promise is worded as "no longer shown" (append-only log;
 *    the no-longer-shown vs true-erasure decision is the human's, still
 *    open);
 *  - the phone-encryption paragraph is omitted here because this is the
 *    in-app path on the pilot PC (the draft's builder note).
 *
 * The D-023 human review remains the bar before ANY community-facing use;
 * the UI renders the DRAFT banner until that sign-off lands and this
 * marker is removed. Placeholders stay [BRACKETED] and must never be
 * filled with a real person's contact information in this repository (I1).
 */

/** Rendered prominently above the statement until D-023 sign-off. */
export const CONSENT_DRAFT_BANNER =
  "DRAFT wording - pending human review (D-023). Not for community use.";

/** Version tag carried in the payload's form_version. */
export const FORM_VERSION = "in-app-draft-2026-07-24";

/** The consent statement paragraphs, in display order. */
export const CONSENT_PARAGRAPHS: readonly (readonly [string, string])[] = [
  [
    "What we collect.",
    "We collect only what you typed on this form: your name, your Tribe or " +
      "Nation if you gave it, the organizations you listed, and your roles. " +
      "Nothing else is gathered from you or your device.",
  ],
  [
    "A person reviews it first.",
    "Your entry goes to the network facilitator, a person working for the " +
      "ATNI Climate community. Nothing you share appears anywhere until the " +
      "facilitator has read and approved it. If something looks off, the " +
      "facilitator will set it aside rather than publish it.",
  ],
  [
    "How it is classified.",
    "Everything entered through this form is held at Tier 1 (T1) under the " +
      "community's data framework: shared within the network, governed by " +
      "the community. ATNI Climate decides how information is classified " +
      "and used - not a company, and not a server.",
  ],
  [
    "Taking part is your choice.",
    "Every question except your name is optional. You can stop at any time " +
      "before sending, and nothing is kept. After sending, you can ask to " +
      "be removed at any time, and your information will no longer be shown " +
      "in the network.",
  ],
  [
    "To be removed or to ask a question,",
    "contact [REMOVAL CONTACT - set at deployment; never a real name or " +
      "address in this repository].",
  ],
];

/** The affirmation the checkbox asserts (the D-030 consent instrument). */
export const CONSENT_AFFIRMATION =
  "I understand, and I agree to be included in the network at the level " +
  "described above.";

/**
 * The canonical consent text the payload digest covers: banner excluded
 * (it is UI chrome, not consented content), paragraphs and affirmation
 * joined in display order.
 */
export function consentText(): string {
  const blocks = CONSENT_PARAGRAPHS.map(([lead, body]) => `${lead} ${body}`);
  blocks.push(CONSENT_AFFIRMATION);
  return blocks.join("\n\n");
}

/**
 * SHA-256 (lowercase hex) of the consent text - the payload's
 * consent_text_digest, proving WHICH wording was affirmed (ADR-005 D5).
 */
export async function consentTextDigest(): Promise<string> {
  const bytes = new TextEncoder().encode(consentText());
  const hash = await globalThis.crypto.subtle.digest("SHA-256", bytes);
  return Array.from(new Uint8Array(hash))
    .map((byte) => byte.toString(16).padStart(2, "0"))
    .join("");
}

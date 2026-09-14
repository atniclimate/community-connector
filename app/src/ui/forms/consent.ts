/**
 * Consent statement boilerplate for the in-app (facilitator-entered) form.
 *
 * DRAFT TEXT v2 (2026-09-14): the wording is ported verbatim from
 * docs/design/intake-consent-text-draft-2026-09-14.md section 3 (the ATNI
 * house voice - institutional third person, bold lead-in + colon, no em
 * dashes, Oxford comma), which supersedes the 2026-07-24 draft this file
 * previously carried (DECISIONS.md D-072.1 authorized that draft as
 * functionality-matching boilerplate; the wording itself was never the
 * subject of that authorization and moves to v2 here). Every truthfulness
 * correction the 07-24 draft's section 7 recorded is still in force:
 *  - the collection claim is scoped to "only what is typed here";
 *  - the removal promise is worded as "no longer be shown" (append-only log;
 *    the no-longer-shown vs true-erasure decision is the human's, still
 *    open);
 *  - the classification paragraph names the Tiered Sovereign Data Framework
 *    in full on first use, per the v2 draft.
 *
 * Divergence from the remote form (form/src/consent.ts, same draft): the v2
 * draft's sixth item ("Your answers are sealed on your phone") describes the
 * remote path, where an attendee's own phone encrypts the answers before they
 * cross the network. That is not true of this path - the in-app form runs on
 * the facilitator's own computer, entered by the facilitator, with no phone
 * and no network hop before staging. That item is replaced below with one
 * that truthfully states the in-app case ("Where this entry is kept"); the
 * other five items are verbatim from the v2 draft. This is a content
 * divergence the draft anticipates in its own builder notes, not a wording
 * error.
 *
 * The D-023 human review remains the bar before ANY community-facing use;
 * the UI renders the DRAFT banner until that sign-off lands and this
 * marker is removed. Placeholders stay [BRACKETED] and must never be
 * filled with a real person's contact information in this repository (I1).
 */

/** Rendered prominently above the statement until D-023 sign-off. */
export const CONSENT_DRAFT_BANNER =
  "DRAFT wording, pending human review (D-023). Not for community use.";

/** The consent block heading (Title Case per house style; v2 draft section 3). */
export const CONSENT_HEADING = "Before You Send This";

/** Version tag carried in the payload's form_version. */
export const FORM_VERSION = "in-app-draft-2026-07-24";

/** The consent statement paragraphs, in display order. */
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
    // Divergence from the v2 draft's sixth item ("Your answers are sealed on
    // your phone"): that item describes the remote (attendee-phone) path.
    // This is the in-app path - the facilitator types the entry on the
    // facilitator's own computer, so there is no phone and no network hop
    // to describe. Replaced with a truthful in-app statement; see the file
    // doc comment.
    "Where this entry is kept:",
    "this entry is typed by the facilitator on the facilitator's own " +
      "computer and stays there until the facilitator applies it to the " +
      "network.",
  ],
];

/** The affirmation the checkbox asserts (the D-030 consent instrument). */
export const CONSENT_AFFIRMATION =
  "I understand, and I agree to be included in the network at the level " +
  "described above.";

/**
 * The canonical consent text the payload digest covers: banner and heading
 * excluded (they are UI chrome, not consented content), paragraphs and
 * affirmation joined in display order.
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

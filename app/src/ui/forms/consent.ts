/**
 * Consent statement for the in-app (facilitator-entered) form.
 *
 * The wording is the human's own, verbatim (source of truth:
 * docs/design/intake-consent-text-2026-09-14-v3.md), superseding the
 * session-drafted v2 wording this file previously carried (DECISIONS.md
 * D-072.1 authorized that draft as functionality-matching boilerplate; the
 * wording itself was never the subject of that authorization and moves to
 * v3 here). v3 is reproduced exactly - every sentence, punctuation mark, and
 * capitalization, in the same order - except the one paragraph the source
 * file's own notes anticipate cannot travel unchanged to this path (below).
 * The session notes recorded alongside the v3 wording are NOT applied to
 * the wording generally; they are flagged questions for the human, not
 * authorized edits.
 *
 * Divergence from the remote form (form/src/consent.ts, same source): the v3
 * statement's fifth paragraph ("Sealed on your phone. Your answers are
 * encrypted on your own device before they are sent...") describes the
 * remote path, where an attendee's own phone encrypts the answers before
 * they cross the network. That is not true of this path - the in-app form
 * runs on the facilitator's own computer, entered by the facilitator, with
 * no phone and no network hop before staging. That paragraph is replaced
 * below with one that truthfully states the in-app case ("Where this entry
 * is kept"); the other seven paragraphs are verbatim from v3. This is a
 * content divergence the source file's own session notes anticipate (note
 * 3), not a wording error.
 *
 * The D-023 human review remains the bar before ANY community-facing use;
 * the UI renders the DRAFT banner until that sign-off lands and this
 * marker is removed.
 */

/** Rendered prominently above the statement until D-023 sign-off. */
export const CONSENT_DRAFT_BANNER =
  "DRAFT wording, pending human review (D-023). Not for community use.";

/** The consent block heading (accessible group label for the fieldset). */
export const CONSENT_HEADING = "Before You Send This";

/** Version tag carried in the payload's form_version. */
export const FORM_VERSION = "in-app-draft-2026-07-24";

/**
 * The consent statement paragraphs, in display order. Each entry is
 * `[lead, body]`: `lead` is the bold lead-in text where the source uses one
 * (verbatim, including its own trailing punctuation), or `null` where the
 * source paragraph has no lead-in. The renderer joins `lead` and `body` with
 * a single space, so together they reproduce the source paragraph exactly.
 */
export const CONSENT_PARAGRAPHS: readonly (readonly [string | null, string])[] = [
  [
    null,
    "This is a demonstration of the connections between all of us here; where effort " +
      "overlaps, and brings to light what is already here between us.",
  ],
  [null, "A name badge tells you who is here. This asks what holds us together."],
  [
    null,
    "Everything you share here stays here. This runs entirely on ATNI software; no " +
      "outside platforms, no third-party services. The only data collected is what you " +
      "enter into this form. Nothing more.",
  ],
  [
    "How your information is held:",
    "Everything entered here is designated Tier 1 of the Tiered Sovereign Data " +
      "Framework; shared within the network and governed by the community it belongs to.",
  ],
  [
    // Divergence from the v3 source's fifth paragraph ("Sealed on your
    // phone. Your answers are encrypted on your own device before they are
    // sent..."): that paragraph describes the remote (attendee-phone) path.
    // This is the in-app path - the facilitator types the entry on the
    // facilitator's own computer, so there is no phone and no network hop
    // to describe. Replaced with a truthful in-app statement; see the file
    // doc comment.
    "Where this entry is kept:",
    "this entry is typed by the facilitator on the facilitator's own " +
      "computer and stays there until the facilitator applies it to the " +
      "network.",
  ],
  [
    null,
    "Nothing appears in the network without care and human review. The content and " +
      "narrative remains yours, and what is presented are the common connection points.",
  ],
  [
    null,
    "After the conference ends on Wednesday, all information entered here is deleted " +
      "from the system entirely.",
  ],
  [
    null,
    "The connections you make, the conversations that follow, the shared experiences; " +
      "those are yours to keep.",
  ],
];

/** The affirmation the checkbox asserts (the D-030 consent instrument), the human's exact wording. */
export const CONSENT_AFFIRMATION =
  "I understand and consent to my information being used for this demonstration.";

/**
 * The canonical consent text the payload digest covers: banner and heading
 * excluded (they are UI chrome, not consented content), paragraphs and
 * affirmation joined in display order.
 */
export function consentText(): string {
  const blocks = CONSENT_PARAGRAPHS.map(([lead, body]) => (lead !== null ? `${lead} ${body}` : body));
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

/**
 * Consent statement for the REMOTE (attendee phone) intake form.
 *
 * The wording is the human's own, verbatim (source of truth:
 * docs/design/intake-consent-text-2026-09-14-v3.md), superseding the
 * session-drafted v2 wording this file previously carried (which followed
 * the ATNI house voice - institutional third person, bold lead-in + colon on
 * every paragraph, bulleted). v3 is prose written in the human's own voice,
 * addressed directly to the reader; it is reproduced exactly - every
 * sentence, punctuation mark, and capitalization, in the same order - and
 * the session notes recorded alongside it in that file are NOT applied to
 * the wording (they are flagged questions for the human, not authorized
 * edits).
 *
 * Two of the eight paragraphs carry a bold lead-in in the source text
 * ("How your information is held:" and "Sealed on your phone."); the rest
 * are plain paragraphs with no lead-in. `CONSENT_PARAGRAPHS` models this as
 * `[lead, body]` pairs with `lead` null where the source has none; the
 * renderer bolds only a non-null lead.
 *
 * This is the REMOTE path, so - unlike the in-app version
 * (app/src/ui/forms/consent.ts) - it keeps the "Sealed on your phone."
 * paragraph verbatim (this form runs on an attendee's own phone, not the
 * facilitator's computer); the in-app file substitutes a truthful in-app
 * paragraph in its place, per that file's own doc comment.
 *
 * D-023 human review remains the bar before ANY community-facing use; the UI
 * shows the DRAFT tag until that sign-off lands.
 *
 * This module imports nothing so the D8 manifest generator can import
 * `consentTextDigest` directly (single source of truth for the digest, no
 * second implementation to drift). It uses only globalThis.crypto.subtle and
 * TextEncoder, both built into the pinned Node 24 runtime and every browser.
 */

/** Rendered as a status tag above the form until D-023 sign-off. */
export const CONSENT_DRAFT_BANNER =
  "DRAFT wording, pending human review (D-023). Not for community use.";

/** The consent block heading (accessible group label for the fieldset). */
export const CONSENT_HEADING = "Before You Send This";

/**
 * The consent statement, in display order. Each entry is `[lead, body]`:
 * `lead` is the bold lead-in text where the source uses one (verbatim,
 * including its own trailing punctuation - a colon for one, a period for
 * the other), or `null` where the source paragraph has no lead-in. The
 * renderer joins `lead` and `body` with a single space, so together they
 * reproduce the source paragraph exactly.
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
    "Sealed on your phone.",
    "Your answers are encrypted on your own device before they are sent. The internet " +
      "services that carry the message cannot read them; the key that opens them is " +
      "used only on the facilitator's computer.",
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
 * affirmation joined in display order. Same shape as
 * app/src/ui/forms/consent.ts `consentText`.
 */
export function consentText(): string {
  const blocks = CONSENT_PARAGRAPHS.map(([lead, body]) => (lead !== null ? `${lead} ${body}` : body));
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

import { describe, expect, it } from "vitest";

import {
  CONSENT_AFFIRMATION,
  CONSENT_DRAFT_BANNER,
  CONSENT_HEADING,
  CONSENT_PARAGRAPHS,
  consentText,
  consentTextDigest,
} from "./consent";

describe("consent text (v3, the human's wording verbatim - D-106)", () => {
  it("is deterministic", async () => {
    expect(consentText()).toBe(consentText());
    expect(await consentTextDigest()).toBe(await consentTextDigest());
  });

  it("carries a DRAFT banner (pending D-023)", () => {
    expect(CONSENT_DRAFT_BANNER).toMatch(/DRAFT/);
    expect(CONSENT_DRAFT_BANNER).toMatch(/D-023/);
  });

  it("keeps the accessible group heading", () => {
    expect(CONSENT_HEADING).toBe("Before You Send This");
  });

  it("includes the phone-encryption paragraph (remote path) verbatim", () => {
    const text = consentText();
    expect(text).toContain(
      "Sealed on your phone. Your answers are encrypted on your own device before they are sent. " +
        "The internet services that carry the message cannot read them; the key that opens them is " +
        "used only on the facilitator's computer.",
    );
  });

  it("reproduces every v3 paragraph verbatim, in order, joined into the exact statement", () => {
    expect(consentText()).toBe(
      [
        "This is a demonstration of the connections between all of us here; where effort " +
          "overlaps, and brings to light what is already here between us.",
        "A name badge tells you who is here. This asks what holds us together.",
        "Everything you share here stays here. This runs entirely on ATNI software; no " +
          "outside platforms, no third-party services. The only data collected is what you " +
          "enter into this form. Nothing more.",
        "How your information is held: Everything entered here is designated Tier 1 of the " +
          "Tiered Sovereign Data Framework; shared within the network and governed by the " +
          "community it belongs to.",
        "Sealed on your phone. Your answers are encrypted on your own device before they are " +
          "sent. The internet services that carry the message cannot read them; the key that " +
          "opens them is used only on the facilitator's computer.",
        "Nothing appears in the network without care and human review. The content and " +
          "narrative remains yours, and what is presented are the common connection points.",
        "After the conference ends on Wednesday, all information entered here is deleted " +
          "from the system entirely.",
        "The connections you make, the conversations that follow, the shared experiences; " +
          "those are yours to keep.",
        CONSENT_AFFIRMATION,
      ].join("\n\n"),
    );
  });

  it("has exactly two lead-ins, verbatim including their own trailing punctuation", () => {
    const leads = CONSENT_PARAGRAPHS.map(([lead]) => lead).filter((lead) => lead !== null);
    expect(leads).toEqual(["How your information is held:", "Sealed on your phone."]);
  });

  it("carries the exact affirmation wording (the checkbox label)", () => {
    expect(CONSENT_AFFIRMATION).toBe(
      "I understand and consent to my information being used for this demonstration.",
    );
  });

  it("has a stable golden SHA-256 digest (catches accidental text edits)", async () => {
    // Regenerate deliberately if the consent wording is intentionally changed:
    //   node --input-type=module -e "import('./src/consent.ts').then(m=>m.consentTextDigest()).then(console.log)"
    // v3 (2026-09-14; docs/design/intake-consent-text-2026-09-14-v3.md; D-106).
    expect(await consentTextDigest()).toBe(
      "460112188735f0d45cabbb08a762aaf3e636fd5d420c16f386cfdf927bebf235",
    );
  });
});

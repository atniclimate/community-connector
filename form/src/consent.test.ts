import { describe, expect, it } from "vitest";

import {
  CONSENT_AFFIRMATION,
  CONSENT_DRAFT_BANNER,
  CONSENT_HEADING,
  CONSENT_PARAGRAPHS,
  consentText,
  consentTextDigest,
} from "./consent";

describe("consent text", () => {
  it("is deterministic", async () => {
    expect(consentText()).toBe(consentText());
    expect(await consentTextDigest()).toBe(await consentTextDigest());
  });

  it("carries a DRAFT banner (pending D-023)", () => {
    expect(CONSENT_DRAFT_BANNER).toMatch(/DRAFT/);
    expect(CONSENT_DRAFT_BANNER).toMatch(/D-023/);
  });

  it("includes the phone-encryption paragraph (remote path) with corrected key-custody wording", () => {
    const text = consentText();
    expect(text).toMatch(/sealed on your phone/i);
    // Correction 2: the key is USED only on the facilitator's computer, not
    // "the only copy".
    expect(text).toMatch(/used only on the facilitator's computer/);
  });

  it("scopes the collection claim to what the person typed (correction 1)", () => {
    expect(consentText()).toMatch(/only what is typed here/);
  });

  it("words removal as no longer shown, not erased (correction 3)", () => {
    expect(consentText()).toMatch(/no longer be shown/);
    expect(consentText()).not.toMatch(/taken out of the network/);
  });

  it("follows the house voice: institutional third person, no em dashes, lead-in + colon", () => {
    const statement = CONSENT_PARAGRAPHS.map(([lead, body]) => `${lead} ${body}`).join("\n");
    expect(statement).not.toMatch(/\b(we|our|ours|us)\b/i);
    expect(statement).not.toMatch(/\bI\b/);
    expect(consentText()).not.toMatch(/—|--/);
    expect(consentText()).not.toMatch(/!/);
    for (const [lead] of CONSENT_PARAGRAPHS) {
      expect(lead.endsWith(":"), lead).toBe(true);
    }
    expect(CONSENT_HEADING).toBe("Before You Send This");
    // The affirmation is the participant's own statement (D-030 instrument),
    // so "I" is theirs, not the institution's.
    expect(CONSENT_AFFIRMATION).toMatch(/^I understand/);
  });

  it("expands the framework name on first use and names the tier authority", () => {
    expect(consentText()).toMatch(/Tier 1 \(T1\) of the Tiered Sovereign Data Framework/);
    expect(consentText()).toMatch(/ATNI Climate decides/);
  });

  it("has a stable golden SHA-256 digest (catches accidental text edits)", async () => {
    // Regenerate deliberately if the consent wording is intentionally changed:
    //   node --input-type=module -e "import('./src/consent.ts').then(m=>m.consentTextDigest()).then(console.log)"
    // Draft v2 (2026-09-14; docs/design/intake-consent-text-draft-2026-09-14.md).
    expect(await consentTextDigest()).toBe("501bc30da042601f683d24bf50670d4d69ab960a913a7739b93e2c5b546a478b");
  });
});

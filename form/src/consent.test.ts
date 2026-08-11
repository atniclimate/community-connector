import { describe, expect, it } from "vitest";

import { CONSENT_DRAFT_BANNER, consentText, consentTextDigest } from "./consent";

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
    expect(consentText()).toMatch(/only what you typed/);
  });

  it("has a stable golden SHA-256 digest (catches accidental text edits)", async () => {
    // Regenerate deliberately if the consent wording is intentionally changed:
    //   node --input-type=module -e "import('./src/consent.ts').then(m=>m.consentTextDigest()).then(console.log)"
    expect(await consentTextDigest()).toBe(
      "ae5f7cc0d740726e81785919d1dbe6ad715895c94da278610da354d730bcd36c",
    );
  });
});

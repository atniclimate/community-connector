/**
 * Bootstrap: initialize libsodium, verify the embedded key against its embedded
 * fingerprint (fail loudly on mismatch - the same self-check crypto.rs's
 * validate_envelope does on the Rust side), then mount the form.
 *
 * Fonts are self-hosted through the pinned @fontsource packages (D-101 item 1:
 * never a runtime request to a font CDN, which would expose each participant's
 * IP address to it and would violate the form's `default-src 'self'` CSP).
 * Subsets: Latin plus Latin Extended (D-101 item 3, names with diacritics).
 * Display: League Spartan 700 (Heading 1) and 800 (wordmark). Body: Arimo,
 * the metric-compatible fallback behind Arial in the D-102 stack, at 400
 * (body), 600 (labels), and 700 (bold lead-ins).
 */
import "@fontsource/league-spartan/latin-500.css";
import "@fontsource/league-spartan/latin-ext-500.css";
import "@fontsource/league-spartan/latin-600.css";
import "@fontsource/league-spartan/latin-ext-600.css";
import "@fontsource/arimo/latin-400.css";
import "@fontsource/arimo/latin-ext-400.css";
import "@fontsource/arimo/latin-600.css";
import "@fontsource/arimo/latin-ext-600.css";
import "@fontsource/arimo/latin-700.css";
import "@fontsource/arimo/latin-ext-700.css";
import "./style.css";
import {
  ALLOWED_KINDS,
  FIELD_HELP,
  FORM_VERSION,
  FRIENDLY_LABELS,
  KEY_FINGERPRINT,
  PUBLIC_KEY_HEX,
  RELAY_ORIGIN,
  TEMPLATE,
} from "./config";
import { computeFingerprint, hexToBytes, ready } from "./crypto";
import { mountForm } from "./render";

function fatal(container: HTMLElement, message: string): void {
  const panel = document.createElement("section");
  panel.className = "cn-fatal";
  panel.setAttribute("role", "alert");
  const heading = document.createElement("h1");
  heading.className = "cn-form-title";
  heading.textContent = "This Form Is Not Available";
  const body = document.createElement("p");
  body.textContent = message;
  panel.append(heading, body);
  container.replaceChildren(panel);
}

async function main(): Promise<void> {
  const container = document.getElementById("app");
  if (container === null) {
    return;
  }
  try {
    await ready();
    const publicKey = hexToBytes(PUBLIC_KEY_HEX);
    // Config self-consistency: a build/ceremony paste error (key not matching
    // its declared fingerprint) must never reach a submitter. Refuse to render.
    const derived = computeFingerprint(publicKey);
    if (derived !== KEY_FINGERPRINT) {
      fatal(
        container,
        "The intake key failed its own integrity check. Tell the facilitator, and do not enter anything.",
      );
      return;
    }
    mountForm(container, {
      template: TEMPLATE,
      publicKey,
      fingerprint: KEY_FINGERPRINT,
      friendlyLabels: FRIENDLY_LABELS,
      fieldHelp: FIELD_HELP,
      allowedKinds: ALLOWED_KINDS,
      relayOrigin: RELAY_ORIGIN,
      formVersion: FORM_VERSION,
    });
  } catch {
    fatal(container, "Something went wrong while preparing this form. Reload the page and try again.");
  }
}

void main();

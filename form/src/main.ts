/**
 * Bootstrap: initialize libsodium, verify the embedded key against its embedded
 * fingerprint (fail loudly on mismatch - the same self-check crypto.rs's
 * validate_envelope does on the Rust side), then mount the form.
 */
import "./style.css";
import { FORM_VERSION, FRIENDLY_LABELS, KEY_FINGERPRINT, PUBLIC_KEY_HEX, RELAY_ORIGIN, TEMPLATE } from "./config";
import { computeFingerprint, hexToBytes, ready } from "./crypto";
import { mountForm } from "./render";

function fatal(container: HTMLElement, message: string): void {
  const panel = document.createElement("section");
  panel.className = "cn-fatal";
  panel.setAttribute("role", "alert");
  const heading = document.createElement("h1");
  heading.textContent = "This form is not available";
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
        "The intake key failed its own integrity check. Please tell the facilitator; do not enter anything.",
      );
      return;
    }
    mountForm(container, {
      template: TEMPLATE,
      publicKey,
      fingerprint: KEY_FINGERPRINT,
      friendlyLabels: FRIENDLY_LABELS,
      relayOrigin: RELAY_ORIGIN,
      formVersion: FORM_VERSION,
    });
  } catch {
    fatal(container, "Something went wrong preparing this form. Please reload the page.");
  }
}

void main();

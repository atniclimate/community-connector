/** Lowercase hex encoding of raw bytes (receipt ids, digest comparison). */
export function toHex(bytes: Uint8Array): string {
  let out = "";
  for (const b of bytes) {
    out += b.toString(16).padStart(2, "0");
  }
  return out;
}

/**
 * Decodes a lowercase-or-mixed-case hex string to bytes, or null if the input
 * is not well-formed even-length hex. Returning null (rather than throwing) lets
 * the auth path treat a malformed configured hash as a plain non-match without a
 * distinct error surface.
 */
export function fromHex(hex: string): Uint8Array | null {
  if (hex.length === 0 || hex.length % 2 !== 0) {
    return null;
  }
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) {
    const byte = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16);
    if (Number.isNaN(byte)) {
      return null;
    }
    out[i] = byte;
  }
  return out;
}

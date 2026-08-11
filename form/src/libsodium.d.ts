/**
 * Minimal ambient declaration for the exact libsodium-wrappers surface this
 * form uses. A local declaration (rather than @types/libsodium-wrappers) keeps
 * the dependency set to exactly one runtime package (libsodium-wrappers@0.7.15)
 * with no separate, version-skewable types package - matching the blueprint's
 * "the form's ONLY runtime dependency" rule and this repo's minimal-dependency
 * posture. Every function here is proven by scripts/generate-crypto-vectors.js,
 * which drives the same API from Node.
 */
declare module "libsodium-wrappers" {
  interface Sodium {
    /** Resolves once the WASM module is initialized. Await before any call. */
    readonly ready: Promise<void>;
    /** Base64 variant selectors. ORIGINAL = RFC 4648 s4, standard padded. */
    readonly base64_variants: {
      readonly ORIGINAL: number;
      readonly ORIGINAL_NO_PADDING: number;
      readonly URLSAFE: number;
      readonly URLSAFE_NO_PADDING: number;
    };
    /** Anonymous sealed box: X25519 + XSalsa20-Poly1305, BLAKE2b nonce. */
    crypto_box_seal(message: Uint8Array, publicKey: Uint8Array): Uint8Array;
    /** BLAKE2b generic hash; hashLength in bytes, optional key. */
    crypto_generichash(
      hashLength: number,
      message: Uint8Array,
      key?: Uint8Array | null,
    ): Uint8Array;
    to_base64(input: Uint8Array, variant?: number): string;
    from_base64(input: string, variant?: number): Uint8Array;
    readonly SODIUM_VERSION_STRING: string;
  }
  const sodium: Sodium;
  export default sodium;
}

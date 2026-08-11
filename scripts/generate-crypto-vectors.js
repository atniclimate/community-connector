// Cross-implementation crypto test-vector generator for the relay sealed-box
// binding (docs/blueprints/intake-relay.md section 1; ADR-005 D3).
//
// The attendee browser seals submissions with libsodium.js; the Rust puller
// (core/crates/cn-ingest/src/crypto.rs) opens them. This script produces the
// authoritative fixture that pins that interop: a TEST-ONLY keypair, its
// fingerprint, and libsodium `crypto_box_seal` ciphertexts for known
// plaintexts. The Rust test tests/crypto_vectors.rs consumes the fixture and
// asserts every JS-sealed box opens in Rust and the fingerprints match.
//
// This is NOT part of check-all (no Node dependency in CI) and NOT a runtime
// dependency of the app or form. Run it by hand to (re)generate the fixture:
//
//   cd scripts
//   npm install
//   npm run generate:crypto-vectors
//
// Sealed boxes use a fresh ephemeral key per call, so the ciphertext bytes
// differ every run; they still open to the same plaintext. The keypair and
// fingerprint are deterministic (fixed seed).

const fs = require('node:fs');
const path = require('node:path');
const sodium = require('libsodium-wrappers');

// TEST-ONLY seed - 32 bytes of readable ASCII. This keypair NEVER touches an
// operational relay; it exists only so the fixture is reproducible.
const SEED_TEXT = 'cn-intake-relay-test-seed-000001';

// Plaintexts to seal. `utf8` vectors carry text; `hex` vectors carry raw bytes
// (including an empty box and a full-byte-range binary blob).
const PLAINTEXTS = [
  { description: 'empty plaintext', encoding: 'hex', value: '' },
  { description: 'short ascii', encoding: 'utf8', value: 'hello relay' },
  {
    description: 'inner-payload-shaped json',
    encoding: 'utf8',
    value: JSON.stringify({
      submission_version: '0.1.0',
      submission_id: 'test-0001',
      fields: { display_name: 'Synthetic Person' },
    }),
  },
  {
    description: 'utf8 with multibyte + separator characters',
    encoding: 'utf8',
    value: 'riparian — fisheries   éè \u{1f30a}',
  },
  {
    description: 'binary, full byte range',
    encoding: 'hex',
    value: Array.from({ length: 256 }, (_, i) => i.toString(16).padStart(2, '0')).join(''),
  },
];

function toBytes(entry) {
  return entry.encoding === 'hex'
    ? Buffer.from(entry.value, 'hex')
    : Buffer.from(entry.value, 'utf8');
}

function fingerprint(publicKey) {
  // BLAKE2b-256 over the raw public key, truncated to 16 bytes, rendered as 8
  // lowercase hex groups of 4 (ceremony design section 2). Must match
  // cn-ingest::crypto::fingerprint exactly.
  const digest = sodium.crypto_generichash(32, publicKey);
  const hex = Buffer.from(digest.slice(0, 16)).toString('hex');
  return hex.match(/.{4}/g).join('-');
}

async function main() {
  await sodium.ready;

  const seed = Buffer.from(SEED_TEXT, 'utf8');
  if (seed.length !== sodium.crypto_box_SEEDBYTES) {
    throw new Error(
      `seed must be ${sodium.crypto_box_SEEDBYTES} bytes, got ${seed.length}`,
    );
  }
  const keypair = sodium.crypto_box_seed_keypair(seed);

  const vectors = PLAINTEXTS.map((entry) => {
    const ciphertext = sodium.crypto_box_seal(toBytes(entry), keypair.publicKey);
    return {
      description: entry.description,
      plaintext_encoding: entry.encoding,
      plaintext: entry.value,
      ciphertext_base64: Buffer.from(ciphertext).toString('base64'),
    };
  });

  const fixture = {
    note:
      'TEST-ONLY sealed-box interop vectors for the relay crypto binding. ' +
      'The keypair here is synthetic and NEVER used operationally. Generated ' +
      'by scripts/generate-crypto-vectors.js. Consumed by ' +
      'core/crates/cn-ingest/tests/crypto_vectors.rs.',
    algorithm: 'crypto_box_seal (X25519 + XSalsa20-Poly1305, BLAKE2b nonce)',
    fingerprint_algorithm: 'blake2b-256(public_key)[..16], 8 lowercase hex groups of 4',
    libsodium_wrappers_version: sodium.SODIUM_VERSION_STRING,
    keypair: {
      seed_utf8: SEED_TEXT,
      public_key_hex: Buffer.from(keypair.publicKey).toString('hex'),
      secret_key_hex: Buffer.from(keypair.privateKey).toString('hex'),
      fingerprint: fingerprint(keypair.publicKey),
    },
    seal_vectors: vectors,
  };

  const outPath = path.resolve(__dirname, '..', 'fixtures', 'crypto', 'sealed-box-vectors.json');
  fs.mkdirSync(path.dirname(outPath), { recursive: true });
  fs.writeFileSync(outPath, JSON.stringify(fixture, null, 2) + '\n');
  console.log(`wrote ${vectors.length} sealed-box vectors to ${outPath}`);
  console.log(`test keypair fingerprint: ${fixture.keypair.fingerprint}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});

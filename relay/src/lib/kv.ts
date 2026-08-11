/**
 * KV key scheme (blueprint 5.2): one namespace, two prefixes. The blob holds
 * the opaque ciphertext envelope; the ledger holds non-content metadata. A
 * receipt id names both halves of the same receipt.
 */
export const BLOB_PREFIX = "blob:";
export const LEDGER_PREFIX = "ledger:";
export const RATELIMIT_PREFIX = "ratelimit:";

export function blobKey(receiptId: string): string {
  return BLOB_PREFIX + receiptId;
}

export function ledgerKey(receiptId: string): string {
  return LEDGER_PREFIX + receiptId;
}

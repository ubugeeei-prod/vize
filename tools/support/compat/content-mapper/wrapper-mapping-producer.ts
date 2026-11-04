/** The #11675 producer adds only diagnostic-owned delimiter atoms. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function assertWrapperMappingProducer(producer: Uint8Array): void {
  assert.equal(
    createHash("sha256").update(producer).digest("hex"),
    "25abafce233364f0b11f71202d32f87290ef7d231f939063938f8649e3448b67",
    "unqualified #11675 wrapper mapping producer",
  );
}

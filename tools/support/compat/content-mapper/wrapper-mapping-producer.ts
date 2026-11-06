/** The #11675 producer adds only diagnostic-owned delimiter atoms. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function assertWrapperMappingProducer(producer: Uint8Array): void {
  const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(
    digest(producer),
    "d51c0acbed93e899ec5b634825bd3d7d2ffb2d47908f5bbc575e68feb909a937",
    "unqualified #11675 wrapper mapping producer",
  );
  let historical = Buffer.from(producer).toString("utf8");
  for (const addition of [
    "mod component_export;\n",
    "    component_export::insert_spans(&mut spans, mappings, metadata);\n",
  ]) {
    assert.equal(historical.split(addition).length, 2);
    historical = historical.replace(addition, "");
  }
  assert.equal(
    digest(historical),
    "71e871d85b79ce81fbf09e684c031d80bea14ec04d1ae34c4f3a0fcaa4580061",
    "certified component-export wrapper successor",
  );
}

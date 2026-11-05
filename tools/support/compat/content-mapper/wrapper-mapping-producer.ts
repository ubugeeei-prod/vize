/** The #11675 producer adds only diagnostic-owned delimiter atoms. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export function assertWrapperMappingProducer(producer: Uint8Array): void {
  const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(
    digest(producer),
    "f75096741294e3f99e5fedf6accad2daf49ee402c72af2bc684b20b0f4310e20",
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
    "25abafce233364f0b11f71202d32f87290ef7d231f939063938f8649e3448b67",
    "certified component-export wrapper successor",
  );
}

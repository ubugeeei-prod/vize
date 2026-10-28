import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  closedSetupReference,
  type ExposurePacket,
} from "../tooling/support/script-setup-default-expose.ts";

export const exposureReferenceCustody = {
  fixSource: "9157b9fa83dbdc98100be084be288f21d0d1bc18",
  referenceBlob: "9de66fef47542ca09981eea7f7fda685d5932f85",
  rawBasePacketsRetained: true,
  candidatePacketsUnmodified: true,
} as const;

const referenceBytes = readFileSync(
  new URL("../tooling/support/script-setup-default-expose.ts", import.meta.url),
);
const referenceHash = createHash("sha1")
  .update(Buffer.from(`blob ${referenceBytes.length}\0`))
  .update(referenceBytes)
  .digest("hex");
assert.equal(referenceHash, exposureReferenceCustody.referenceBlob);

export function attributesPacketReference(
  packet: Record<string, unknown>,
  source: string,
  mode: string,
): Record<string, unknown> {
  if (mode !== "compile" || packet.kind !== "return") return structuredClone(packet);
  assert.equal(typeof packet.json, "string");
  const value: unknown = JSON.parse(String(packet.json));
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  assert("code" in value && typeof value.code === "string");
  const reference = closedSetupReference(value as ExposurePacket, source);
  return { ...packet, json: JSON.stringify(reference) };
}

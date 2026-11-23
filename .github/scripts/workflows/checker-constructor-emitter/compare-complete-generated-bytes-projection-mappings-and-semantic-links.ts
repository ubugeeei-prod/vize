import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
const directory = `${process.env.RUNNER_TEMP}/constructor-emitter`;
const baseBytes = readFileSync(`${directory}/base.json`);
const headBytes = readFileSync(`${directory}/head.json`);
const base = JSON.parse(baseBytes.toString("utf8")),
  head = JSON.parse(headBytes.toString("utf8"));
assert.equal(base.length, 228);
assert.equal(head.length, 228);
assert.deepEqual(head, base, "complete checker/mapper projection packets must be unchanged");
const hash = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const observer = readFileSync("crates/vize_canon/tests/constructor_emission_projection.rs");
const receipt = {
  base: process.env.BASE_SHA,
  head: process.env.HEAD_SHA,
  observerSha256: hash(observer),
  basePacketSha256: hash(baseBytes),
  headPacketSha256: hash(headBytes),
  completePackets: head.length,
  scope: "Exact source projection parity; not native type-engine or publication acceptance",
};
writeFileSync(`${directory}/receipt.json`, JSON.stringify(receipt, null, 2) + "\n");
console.log(JSON.stringify(receipt));

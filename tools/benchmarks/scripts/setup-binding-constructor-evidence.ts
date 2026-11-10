import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import {
  comparePackets,
  readAssets,
  type Row,
} from "../../support/compat/content-mapper/constructor-projection-parity.ts";

const [directory, baseRoot, headRoot] = process.argv.slice(2);
assert.ok(directory && baseRoot && headRoot, "evidence directory and source roots are required");
const read = (name: string) => readFileSync(join(directory, name));
const digest = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
const baseBytes = read("base-constructor.json");
const headBytes = read("head-constructor.json");
const proof = comparePackets(
  JSON.parse(baseBytes.toString()) as Row[],
  JSON.parse(headBytes.toString()) as Row[],
  readAssets(baseRoot, headRoot),
);
const receipt = {
  baseSha256: digest(baseBytes),
  headSha256: digest(headBytes),
  ...proof,
  scope: "complete constructor packets under exact authenticated projection transitions",
};
writeFileSync(
  join(directory, "constructor-comparison.json"),
  JSON.stringify(receipt, null, 2) + "\n",
);
console.log(JSON.stringify(receipt));

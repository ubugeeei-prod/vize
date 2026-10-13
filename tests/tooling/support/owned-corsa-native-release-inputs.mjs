import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const pin = new URL("../fixtures/typechecker-native-release-inputs.json", import.meta.url);
const stat = fs.lstatSync(pin);
assert(stat.isFile() && (stat.mode & 0o777) === 0o644);
const raw = fs.readFileSync(pin);
assert.equal(
  createHash("sha256").update(raw).digest("hex"),
  "bb6a287b283f253a887fcc5e027fc916f691367afa2ee9971d77bbf61a4bfdc0",
);
const original = JSON.parse(raw).positivePaths;
const additions = [
  "vendor/corsa/**",
  "vendor/corsa_core/**",
  "vendor/corsa_jsonrpc/**",
  "vendor/corsa_client/**",
  "vendor/corsa_lsp/**",
  "vendor/corsa_orchestrator/**",
  "tools/support/compat/github/owned-corsa-package-*",
  "tools/support/compat/github/owned-canon-external-consumer.mjs",
  "tests/_fixtures/ci/owned-corsa-package-consumer/**",
  "tests/_fixtures/ci/owned-canon-external-consumer/**",
];

/** Ten required package inputs; all original ordered triggers and exclusions remain literal. */
export function ownedCorsaNativeReleaseInputs(retained) {
  assert.deepEqual(retained, original, "complete original release input authority");
  assert.equal(original.length, 34);
  assert.deepEqual(original.slice(-3), ["Cargo.toml", "Cargo.lock", "pnpm-lock.yaml"]);
  return [...original.slice(0, -3), ...additions, ...original.slice(-3)];
}

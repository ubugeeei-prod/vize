import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

const root = new URL("../../../../", import.meta.url);
const support = "tests/performance/support/";
const names = [
  "warm-type-backed-native-lexical-composition-manifest.json",
  "warm-type-backed-native-owned-image-manifest.json",
  "warm-type-backed-native-terminal-manifest.json",
  "warm-type-backed-native-terminal-members-1.json",
  "warm-type-backed-native-terminal-members-2.json",
];
const hash = (value) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
const read = (name) => JSON.parse(fs.readFileSync(new URL(support + name, root), "utf8"));
const [composition, sdk, original, first, second] = names.map(read);
original.files = [...first, ...second];
assert.equal(hash(composition), "605226da9a7016515902692e0a21a4f3fd983bb8381aff8de31795b5ace746b8");
assert.equal(hash(sdk), composition.sdkManifestSHA256);
assert.equal(hash(original), sdk.originalManifestSHA256);
const expected = new Map(original.files.map(([file, , after]) => [file, after]));
for (const [file, , after] of sdk.successorBodies) expected.set(file, after);
assert.equal(expected.size, 121);
for (const [file, sdkAfter, , after] of composition.sourceSuccessors) {
  assert.equal(expected.get(file), sdkAfter);
  expected.set(file, after);
}
for (const [file, body] of original.preservedBodies) expected.set(file, body);
for (const [file, , after] of original.qualifiedHarnessBodies) expected.set(file, after);
for (const [file, , after] of sdk.qualifiedHarnessBodies) expected.set(file, after);
for (const [file, previous, after] of composition.qualifiedHarnessSuccessors) {
  assert.equal(expected.get(file), previous);
  expected.set(file, after);
}
for (const [file, , after] of [...sdk.corpusBodies, ...composition.historicalReplayBodies])
  expected.set(file, after);
for (const [file, , after] of composition.incomingPreservedSuccessors) expected.set(file, after);

export const lexicalCompositionSourcePaths = [
  ...names.map((name) => support + name),
  ...expected.keys(),
];

export const lexicalSourcePaths = (manifest, helpers) => [
  ...new Set([
    ...helpers,
    ...manifest.files.map(([file]) => file),
    ...manifest.corpusBodies.map(([file]) => file),
    "tests/performance/support/warm-type-backed-lexical-dollar-manifest.json",
    "Cargo.toml",
    "Cargo.lock",
    "rust-toolchain.toml",
    "crates/vize_canon/Cargo.toml",
    "crates/vize_canon/src/lib.rs",
    ...lexicalCompositionSourcePaths,
    ...manifest.preservedBodies.map(([file]) => file),
  ]),
];

/** Original lexical authority is unchanged; only the whole finite SDK composition is additive. */
export function validateLexicalSourceBodies(manifest, copied) {
  assert.equal(hash(manifest), composition.lexicalManifestSHA256);
  const bodies = new Map(copied.map((entry) => [entry.file, entry.sha256]));
  const originalBodies = [...manifest.files, ...manifest.corpusBodies];
  if (originalBodies.every(([file, , after]) => bodies.get(file) === after)) return null;
  for (const [file, , after] of originalBodies) {
    const replacement = composition.sourceSuccessors.find(([name]) => name === file);
    if (replacement) assert.equal(replacement[2], after);
    assert.equal(bodies.get(file), replacement?.[3] ?? after, file);
  }
  for (const [file, body] of manifest.preservedBodies) {
    const replacement = composition.incomingPreservedSuccessors.find(([name]) => name === file);
    if (replacement) assert.equal(replacement[1], body);
    assert.equal(bodies.get(file), replacement?.[2] ?? body, file);
  }
  for (const [file, body] of expected) assert.equal(bodies.get(file), body, file);
  for (const name of names)
    assert.equal(
      bodies.get(support + name),
      createHash("sha256")
        .update(fs.readFileSync(new URL(support + name, root)))
        .digest("hex"),
      name,
    );
  return {
    authority: "complete-reviewed-native-owned-image-incoming-lexical-composition",
    manifestSHA256: hash(composition),
    reviewedSource: composition.reviewedSource,
    reviewedSourceTree: composition.reviewedSourceTree,
    completeSdkProductionBodies: 121,
    incomingLexicalSourceBodies: 9,
    incomingLexicalCorpusBodies: 21,
    originalRuntimeBodies: 23,
    finalGuardBodies: 2,
    runtimeCredit: false,
    performanceCredit: false,
  };
}

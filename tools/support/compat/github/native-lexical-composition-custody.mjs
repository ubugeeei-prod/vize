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

const rawHash = (body) => createHash("sha256").update(body).digest("hex");
const sourceBytes = (file) => fs.readFileSync(new URL(file, root));
const finalCaller = support + "warm-type-backed-native-terminal-final.ts";
const finalHelper = support + "warm-type-backed-native-proxy-final-body.ts";
const finalMetadata = support + "warm-type-backed-native-proxy-final-body-manifest.json";
const finalCapsule = "tests/tooling/support/native-proxy-original-final.json";
const fixtureMetadata = support + "warm-type-backed-native-proxy-manifest.json";
const fixtureCorrection = support + "warm-type-backed-native-proxy-correction.json";
const finalDataBodies = new Map([
  [finalHelper, "1d8803f466a468508b794e32ca664543bea519f181b658cc542b38aa501abefc"],
  [finalMetadata, "6c2f6d6a1431e562e608762bb017ef77a5dc667a24456d24a74fe5966c870f24"],
  [finalCapsule, "516019d239dc419f90bc3b14eef99f46b2c7abde550a00d49e3aecf548f18c1e"],
  [fixtureMetadata, "00faed1d04b5d3d70ff54b32bc9dc32bdad6c36229df3fee8fc3d0c486381933"],
  [fixtureCorrection, "19e6cbc523685ed997d248d3ae9875c0f8428d40a3047b5a900136f53cdd8190"],
]);
const currentFixture = JSON.parse(sourceBytes(fixtureMetadata));
assert.equal(
  hash(currentFixture),
  "d90b51370f0eaaa6e7459477d05dd0d225cd88e91fed9bbeb11e04083f0e2680",
);
assert.equal(currentFixture.files.length, 7);
assert.equal(new Set(currentFixture.files.map(([file]) => file)).size, 7);
export const lexicalFinalSourcePaths = [
  ...finalDataBodies.keys(),
  ...currentFixture.files.map(([file]) => file),
];
const runtimeHelpers = [
  "native-lexical-dollar.sh",
  "native-lexical-dollar-metadata.mjs",
  "native-lexical-dollar-receipt.mjs",
  "native-lexical-composition-custody.mjs",
].map((name) => "tools/support/compat/github/" + name);

function once(body, before, after = "") {
  assert.equal(body.split(before).length, 2, "whole final caller inverse clause must occur once");
  return body.replace(before, after);
}

/** This finite source adapter grants no native or performance qualification. */
export function validateLexicalFinalSource(copied, reader = sourceBytes) {
  assert.equal(new Set(copied.map(({ file }) => file)).size, copied.length);
  const bodies = new Map(copied.map(({ file, sha256 }) => [file, sha256]));
  for (const [file, expectedBody] of finalDataBodies) {
    assert.equal(bodies.get(file), expectedBody, file);
    assert.equal(rawHash(reader(file)), expectedBody, file);
  }
  const metadata = JSON.parse(reader(finalMetadata));
  assert.deepEqual(metadata, {
    version: 1,
    beforeSource: "dd188515f8ec3c51662b4f4d69adcd39c86a0907",
    previousSource: "e17c27d04521ca1409ffb65b7c700d417fc66733",
    file: finalCaller,
    beforeSHA256: null,
    previousAfterSHA256: expected.get(finalCaller),
    afterSHA256: "f5a60202ec1ff1b149cbb320ac3c8f7382ff85a2c9a1c8cb5ebafed863bdccbc",
  });
  const { rawBase64, ...identity } = JSON.parse(reader(finalCapsule));
  assert.deepEqual(identity, {
    version: 1,
    source: metadata.previousSource,
    file: finalCaller,
    mode: "100644",
    blob: "20ad1caaaaf3cdea63ed741a81ad0dfdb8e85345",
    bytes: 1144,
    sha256: metadata.previousAfterSHA256,
  });
  assert.equal(typeof rawBase64, "string");
  const originalBody = Buffer.from(rawBase64, "base64");
  assert.equal(originalBody.toString("base64"), rawBase64);
  assert.equal(originalBody.length, identity.bytes);
  assert.equal(rawHash(originalBody), identity.sha256);
  assert.equal(
    createHash("sha1").update(`blob ${originalBody.length}\0`).update(originalBody).digest("hex"),
    identity.blob,
  );
  const current = reader(finalCaller);
  assert.equal(rawHash(current), metadata.afterSHA256);
  assert.equal(bodies.get(finalCaller), metadata.afterSHA256);
  let restored = current.toString("utf8");
  restored = once(
    restored,
    'import { qualifyNativeProxyHost } from "./warm-type-backed-native-proxy-host.ts";\n',
  );
  restored = once(
    restored,
    'import { assertNativeProxyFinalFixtureReceipt } from "./warm-type-backed-native-proxy-final-body.ts";\n',
  );
  restored = once(restored, "  nativeProxyHost?: ReturnType<typeof qualifyNativeProxyHost>;\n");
  restored = once(
    restored,
    `  const allowed = new Set(reviewedHostPaths);
  const fixture = qualifyNativeProxyHost(binding.production, allowed, (side, file) =>
    gitBodyDigest(driverRoot, side === "before" ? binding.baseline : binding.head, file),
  );
  assertNativeProxyFinalFixtureReceipt(fixture, binding.nativeProxyHost);
  for (const file of fixture.files) allowed.add(file);
`,
  );
  restored = once(
    restored,
    `  const qualified = qualifyNativeTerminalHost(binding.production, allowed, (side, file) =>
    gitBodyDigest(driverRoot, side === "before" ? binding.baseline : binding.head, file),
  );`,
    `  const qualified = qualifyNativeTerminalHost(
    binding.production,
    new Set(reviewedHostPaths),
    (side, file) =>
      gitBodyDigest(driverRoot, side === "before" ? binding.baseline : binding.head, file),
  );`,
  );
  assert.deepEqual(Buffer.from(restored), originalBody);
  const fixture = JSON.parse(reader(fixtureMetadata));
  const correction = JSON.parse(reader(fixtureCorrection));
  assert.equal(hash(fixture), "d90b51370f0eaaa6e7459477d05dd0d225cd88e91fed9bbeb11e04083f0e2680");
  assert.equal(
    hash(correction),
    "a5ba4433860ea0c3c2cd53c4f5eb00cc5d4c079ba62b221c33a5581620d0b42b",
  );
  assert.equal(fixture.files.find(([file]) => file === correction.file)?.[2], correction.before);
  const variants = [
    {
      reviewedSource: fixture.reviewedSource,
      reviewedSourceTree: fixture.reviewedSourceTree,
      files: fixture.files,
    },
    {
      reviewedSource: correction.reviewedSource,
      reviewedSourceTree: correction.reviewedSourceTree,
      files: fixture.files.map(([file, before, after]) => [
        file,
        before,
        file === correction.file ? correction.after : after,
      ]),
    },
  ];
  const selected = variants.find(({ files }) =>
    files.every(([file, , after]) => bodies.get(file) === after),
  );
  assert(selected, "one complete reviewed seven-body fixture variant is required");
  for (const [file, , body] of selected.files) assert.equal(rawHash(reader(file)), body, file);
  return {
    authority: "complete-reviewed-native-lexical-final-caller-source",
    file: finalCaller,
    previousSource: metadata.previousSource,
    beforeSHA256: metadata.beforeSHA256,
    previousAfterSHA256: metadata.previousAfterSHA256,
    afterSHA256: metadata.afterSHA256,
    completeCallerBodies: 1,
    completeFixtureBodies: 7,
    fixtureReviewedSource: selected.reviewedSource,
    fixtureReviewedSourceTree: selected.reviewedSourceTree,
    runtimeCredit: false,
    performanceCredit: false,
  };
}

export const lexicalCompositionSourcePaths = [
  ...names.map((name) => support + name),
  ...expected.keys(),
  ...lexicalFinalSourcePaths,
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
export function validateLexicalSourceBodies(manifest, copied, reader = sourceBytes) {
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
  const finalSource =
    bodies.get(finalCaller) === expected.get(finalCaller)
      ? null
      : validateLexicalFinalSource(copied, reader);
  if (finalSource) {
    assert.deepEqual(
      copied.map(({ file }) => file).toSorted((a, b) => a.localeCompare(b)),
      lexicalSourcePaths(manifest, runtimeHelpers).toSorted((a, b) => a.localeCompare(b)),
    );
    for (const entry of copied) assert.equal(entry.sha256, rawHash(reader(entry.file)), entry.file);
  }
  for (const [file, body] of expected)
    assert.equal(
      bodies.get(file),
      finalSource?.file === file ? finalSource.afterSHA256 : body,
      file,
    );
  for (const name of names)
    assert.equal(
      bodies.get(support + name),
      createHash("sha256")
        .update(reader(support + name))
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
    ...(finalSource ? { finalSource } : {}),
  };
}

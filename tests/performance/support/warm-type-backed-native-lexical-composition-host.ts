import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import {
  qualifyNativeProxyFinalBody,
  nativeProxyFinalManifestBodySHA256,
} from "./warm-type-backed-native-proxy-final-body.ts";
import { nativeTerminalManifest as original } from "./warm-type-backed-native-terminal-host.ts";
import {
  nativeOwnedImageBodyPairs,
  nativeOwnedImageManifest as sdk,
  qualifyNativeTerminalHost as qualifyPreviousHost,
} from "./warm-type-backed-native-owned-image-host.ts";

type Digest = (side: "before" | "after", file: string) => string | null;
type Body = [string, string | null, string];
type Successor = [string, string, string, string];
const hash = (value: unknown) => createHash("sha256").update(JSON.stringify(value)).digest("hex");
const read = (name: string) => JSON.parse(fs.readFileSync(new URL(name, import.meta.url), "utf8"));
export const nativeLexicalCompositionManifest = read(
  "./warm-type-backed-native-lexical-composition-manifest.json",
) as {
  version: number;
  sdkManifestSHA256: string;
  lexicalManifestSHA256: string;
  reviewedSource: string;
  reviewedSourceTree: string;
  sourceSuccessors: Successor[];
  qualifiedHarnessSuccessors: Array<[string, string, string]>;
  incomingPreservedSuccessors: Array<[string, string, string]>;
  historicalReplayBodies: Body[];
};
const composition = nativeLexicalCompositionManifest;
const lexical = read("./warm-type-backed-lexical-dollar-manifest.json") as {
  files: Body[];
  corpusBodies: Body[];
  preservedBodies: Array<[string, string]>;
};
const manifestSHA256 = hash(composition);
assert.equal(manifestSHA256, "605226da9a7016515902692e0a21a4f3fd983bb8381aff8de31795b5ace746b8");
assert.equal(composition.version, 1);
assert.equal(composition.sdkManifestSHA256, hash(sdk));
assert.equal(composition.lexicalManifestSHA256, hash(lexical));
assert.equal(
  composition.sdkManifestSHA256,
  "cba2a1701ebdf3af5ae2b02465507a70a9cc43d0d56b2d94271efbe24ba28a4f",
);
assert.equal(
  composition.lexicalManifestSHA256,
  "c6cee7b641ef65f599a9077cfce5936317f8bc14a5670ca4e1ad5291eb780229",
);
assert.match(composition.reviewedSourceTree, /^[a-f0-9]{40}$/u);
assert.equal(composition.reviewedSource, "46bebe0045e5d5ac44446d02dc4352c526c86e6c");
assert.equal(composition.historicalReplayBodies.length, 7);
assert.equal(new Set(composition.historicalReplayBodies.map(([file]) => file)).size, 7);
for (const [file, before, after] of composition.historicalReplayBodies) {
  assert.match(
    file,
    /^tests\/_fixtures\/differential\/lsp\/native-lexical-composition-before\/[a-z.-]+$/u,
  );
  assert.equal(before, null);
  assert.match(after, /^[a-f0-9]{64}$/u);
}
assert.deepEqual(
  composition.sourceSuccessors.map(([file]) => file),
  [
    "crates/vize_canon/src/batch/executor/cli.rs",
    "crates/vize_canon/src/batch/executor/session.rs",
  ],
);
assert.equal(lexical.files.length, 9);
assert.equal(lexical.corpusBodies.length, 21);
assert.equal(lexical.preservedBodies.length, 7);
assert.deepEqual(composition.incomingPreservedSuccessors, [
  [
    "tests/_fixtures/differential/typechecker/manifest.json",
    "29744aee2e9521353464cdd2c1d445ed6426525c1e087ac21ed56b2d55f5fdd6",
    "25a76e75a3c3e95e9cc0f474f89d0951b7d87426a0e9f7c1cb1c6fa4c1b7e8f5",
  ],
]);
const successors = new Map(composition.sourceSuccessors.map((row) => [row[0], row]));
for (const [file, sdkAfter, incomingAfter, composedAfter] of composition.sourceSuccessors) {
  const previous = nativeOwnedImageBodyPairs.find(([name]) => name === file);
  const incoming = lexical.files.find(([name]) => name === file);
  assert.ok(previous && incoming);
  assert.equal(previous[1], incoming[1]);
  assert.equal(previous[2], sdkAfter);
  assert.equal(incoming[2], incomingAfter);
  assert.match(composedAfter, /^[a-f0-9]{64}$/u);
  assert.notEqual(composedAfter, sdkAfter);
  assert.notEqual(composedAfter, incomingAfter);
}
export const nativeLexicalCompositionBodyPairs: Body[] = nativeOwnedImageBodyPairs.map(
  ([file, before, after]) => {
    const successor = successors.get(file);
    return [file, successor?.[2] ?? before, successor?.[3] ?? after];
  },
);
const harness: Body[] = original.qualifiedHarnessBodies.map(([file, before, after]) => {
  const previous = sdk.qualifiedHarnessBodies.find(([name]) => name === file)?.[2] ?? after;
  const successor = composition.qualifiedHarnessSuccessors.find(([name]) => name === file);
  if (successor) {
    assert.equal(successor[1], previous);
    assert.match(successor[2], /^[a-f0-9]{64}$/u);
    assert.notEqual(successor[2], previous);
  }
  return [file, before, successor?.[2] ?? previous];
});
assert.equal(composition.qualifiedHarnessSuccessors.length, 1);
assert.equal(new Set(harness.map(([file]) => file)).size, 2);
assert.equal(
  harness.filter(([file]) => composition.qualifiedHarnessSuccessors.some(([name]) => name === file))
    .length,
  1,
);

/** Only this complete incoming lexical custody plus the reviewed SDK composition may record. */
export function qualifyNativeTerminalHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: Digest,
) {
  const previous = qualifyPreviousHost(production, alreadyQualified, digest);
  if (previous) return previous;
  if (new Set(production).size !== production.length) return null;
  const expected = nativeLexicalCompositionBodyPairs.map(([file]) => file).toSorted();
  const files = new Set(expected);
  if (production.every((file) => alreadyQualified.has(file))) return null;
  if (production.some((file) => !alreadyQualified.has(file) && !files.has(file))) return null;
  if (
    JSON.stringify(production.filter((file) => files.has(file)).toSorted()) !==
    JSON.stringify(expected)
  )
    return null;
  for (const [file, before, after] of [
    ...nativeLexicalCompositionBodyPairs,
    ...sdk.corpusBodies,
    ...composition.historicalReplayBodies,
  ])
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  for (const [file, , acceptedAfter] of [...lexical.files, ...lexical.corpusBodies]) {
    const after = successors.get(file)?.[3] ?? acceptedAfter;
    if (digest("before", file) !== acceptedAfter || digest("after", file) !== after) return null;
  }
  for (const [file, body] of lexical.preservedBodies) {
    const successor = composition.incomingPreservedSuccessors.find(([name]) => name === file);
    if (successor && successor[1] !== body) return null;
    if (digest("before", file) !== body || digest("after", file) !== (successor?.[2] ?? body))
      return null;
  }
  for (const [file, originalBody] of original.preservedBodies) {
    const qualified = harness.find(([name]) => name === file);
    if (qualified && qualified[1] !== originalBody) return null;
    if (
      digest("before", file) !== originalBody ||
      digest("after", file) !== (qualified?.[2] ?? originalBody)
    )
      return null;
  }
  const helper = "tests/performance/support/warm-type-backed-native-proxy-final-body.ts";
  const metadata =
    "tests/performance/support/warm-type-backed-native-proxy-final-body-manifest.json";
  const finalBody =
    digest("before", helper) === null &&
    digest("after", helper) ===
      "1d8803f466a468508b794e32ca664543bea519f181b658cc542b38aa501abefc" &&
    digest("before", metadata) === null &&
    digest("after", metadata) === nativeProxyFinalManifestBodySHA256
      ? qualifyNativeProxyFinalBody(production, digest)
      : null;
  for (const [file, before, after] of harness) {
    const expectedAfter = finalBody?.file === file ? finalBody.afterSHA256 : after;
    if (finalBody?.file === file && finalBody.previousAfterSHA256 !== after) return null;
    if (digest("before", file) !== before || digest("after", file) !== expectedAfter) return null;
  }
  return {
    authority: "complete-reviewed-native-owned-image-incoming-lexical-composition",
    manifestSHA256,
    sdkManifestSHA256: composition.sdkManifestSHA256,
    originalManifestSHA256: sdk.originalManifestSHA256,
    originalSource: original.originalSource,
    previousSource: sdk.previousSource,
    reviewedSource: composition.reviewedSource,
    reviewedSourceTree: composition.reviewedSourceTree,
    completeBodyPairs: nativeLexicalCompositionBodyPairs.length,
    additiveCorpusBodyPairs: sdk.corpusBodies.length,
    wholeSourceBodyPairs: nativeLexicalCompositionBodyPairs.length + sdk.corpusBodies.length,
    preservedBodyPairs: original.preservedBodies.length,
    qualifiedHarnessBodyPairs: harness.length,
    historicalReplayBodyPairs: composition.historicalReplayBodies.length,
    incomingLexical: {
      manifestSHA256: composition.lexicalManifestSHA256,
      completeBodyPairs: 9,
      completeCorpusBodyPairs: 21,
      preservedBodyPairs: 7,
    },
    runtimeCredit: false,
    performanceCredit: false,
    ...(finalBody ? { nativeProxyFinalBody: finalBody } : {}),
    files: expected,
  };
}

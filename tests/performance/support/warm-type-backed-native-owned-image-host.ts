import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import {
  nativeTerminalManifest,
  qualifyNativeTerminalHost as qualifyOriginalHost,
} from "./warm-type-backed-native-terminal-host.ts";

type Digest = (side: "before" | "after", file: string) => string | null;
type Body = [string, string | null, string];
const original = nativeTerminalManifest;
export const nativeOwnedImageManifest = JSON.parse(
  fs.readFileSync(
    new URL("./warm-type-backed-native-owned-image-manifest.json", import.meta.url),
    "utf8",
  ),
) as {
  version: number;
  originalManifestSHA256: string;
  previousSource: string;
  reviewedSource: string;
  successorBodies: Body[];
  corpusBodies: Body[];
  qualifiedHarnessBodies: Body[];
};
const successor = nativeOwnedImageManifest;
const manifestSHA256 = createHash("sha256").update(JSON.stringify(successor)).digest("hex");
assert.equal(manifestSHA256, "cba2a1701ebdf3af5ae2b02465507a70a9cc43d0d56b2d94271efbe24ba28a4f");
assert.equal(successor.version, 1);
assert.equal(
  successor.originalManifestSHA256,
  createHash("sha256").update(JSON.stringify(original)).digest("hex"),
);
assert.equal(successor.successorBodies.length, 3);
assert.equal(successor.corpusBodies.length, 1);
assert.equal(successor.qualifiedHarnessBodies.length, 1);
const replacement = new Map(successor.successorBodies.map((row) => [row[0], row]));
assert.equal(replacement.size, 3);
for (const [file, previous, after] of successor.successorBodies) {
  assert.equal(previous, original.files.find(([name]) => name === file)?.[2] ?? null);
  assert.match(after, /^[a-f0-9]{64}$/u);
  assert.notEqual(after, previous);
}
const newProduction = successor.successorBodies.filter(
  ([file]) => !original.files.some(([name]) => name === file),
);
assert.equal(newProduction.length, 1);
export const nativeOwnedImageBodyPairs: Body[] = [
  ...original.files.map(([file, before, after]): Body => [
    file,
    before,
    replacement.get(file)?.[2] ?? after,
  ]),
  ...newProduction,
];
export const nativeOwnedImageProduction = nativeOwnedImageBodyPairs
  .map(([file]) => file)
  .toSorted();
assert.equal(nativeOwnedImageProduction.length, 121);
assert.equal(new Set(nativeOwnedImageProduction).size, 121);
for (const [file, previous, after] of successor.qualifiedHarnessBodies) {
  assert.equal(previous, original.qualifiedHarnessBodies.find(([name]) => name === file)?.[2]);
  assert.match(after, /^[a-f0-9]{64}$/u);
  assert.notEqual(after, previous);
}
for (const [, before, after] of successor.corpusBodies) {
  assert.equal(before, null);
  assert.match(after, /^[a-f0-9]{64}$/u);
}

/** Keep the original120 refusal unchanged; admit only this whole Linux-image successor. */
export function qualifyNativeTerminalHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: Digest,
) {
  const existing = qualifyOriginalHost(production, alreadyQualified, digest);
  if (existing) return existing;
  if (new Set(production).size !== production.length) return null;
  if (production.every((file) => alreadyQualified.has(file))) return null;
  const expected = nativeOwnedImageProduction;
  const files = new Set(expected);
  if (production.some((file) => !alreadyQualified.has(file) && !files.has(file))) return null;
  if (
    JSON.stringify(production.filter((file) => files.has(file)).toSorted()) !==
    JSON.stringify(expected.toSorted())
  )
    return null;
  for (const [file, before, after] of [...nativeOwnedImageBodyPairs, ...successor.corpusBodies])
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  const harness = original.qualifiedHarnessBodies.map(([file, before, after]): Body => [
    file,
    before,
    successor.qualifiedHarnessBodies.find(([name]) => name === file)?.[2] ?? after,
  ]);
  for (const [file, originalBody] of original.preservedBodies) {
    const qualified = harness.find(([name]) => name === file);
    if (qualified && qualified[1] !== originalBody) return null;
    if (
      digest("before", file) !== originalBody ||
      digest("after", file) !== (qualified?.[2] ?? originalBody)
    )
      return null;
  }
  for (const [file, before, after] of harness)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-reviewed-native-owned-linux-image-repair",
    manifestSHA256,
    originalManifestSHA256: successor.originalManifestSHA256,
    originalSource: original.originalSource,
    previousSource: successor.previousSource,
    reviewedSource: successor.reviewedSource,
    completeBodyPairs: nativeOwnedImageBodyPairs.length,
    additiveCorpusBodyPairs: successor.corpusBodies.length,
    wholeSourceBodyPairs: nativeOwnedImageBodyPairs.length + successor.corpusBodies.length,
    preservedBodyPairs: original.preservedBodies.length,
    qualifiedHarnessBodyPairs: harness.length,
    files: expected,
  };
}

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type Digest = (side: "before" | "after", file: string) => string | null;
type LockDigests = Record<string, string>;
const metadata = JSON.parse(
  fs.readFileSync(
    new URL("./warm-type-backed-native-terminal-manifest.json", import.meta.url),
    "utf8",
  ),
) as {
  version: number;
  originalSource: string;
  reviewedSource: string;
  preservedBodies: Array<[string, string]>;
  qualifiedHarnessBodies: Array<[string, string | null, string]>;
};
export const nativeTerminalManifest = {
  ...metadata,
  files: [1, 2].flatMap(
    (part) =>
      JSON.parse(
        fs.readFileSync(
          new URL(`./warm-type-backed-native-terminal-members-${part}.json`, import.meta.url),
          "utf8",
        ),
      ) as Array<[string, string | null, string]>,
  ),
};
const manifest = nativeTerminalManifest;
const manifestSHA256 = createHash("sha256").update(JSON.stringify(manifest)).digest("hex");
assert.equal(manifestSHA256, "9cf1e55d33870c90456fb1e58cec379b2a5f6e39e6c4b09ddee26783b4b0783e");
assert.equal(manifest.version, 1);
assert.equal(manifest.originalSource, "006c564067bacd05be9eac5660e34992e0bdabe8");
assert.equal(manifest.files.length, 120);
assert.equal(new Set(manifest.files.map(([file]) => file)).size, manifest.files.length);
assert.equal(manifest.preservedBodies.length, 23);
assert.equal(manifest.qualifiedHarnessBodies.length, 2);
assert.equal(new Set(manifest.qualifiedHarnessBodies.map(([file]) => file)).size, 2);
for (const [, before, after] of manifest.files) {
  if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
  assert.match(after, /^[a-f0-9]{64}$/u);
}

/** Enroll this complete production repair; every original request and budget still runs. */
export function qualifyNativeTerminalHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: Digest,
) {
  if (new Set(production).size !== production.length) return null;
  const expected = manifest.files.map(([file]) => file);
  const files = new Set(expected);
  if (production.every((file) => alreadyQualified.has(file))) return null;
  if (production.some((file) => !alreadyQualified.has(file) && !files.has(file))) return null;
  if (
    JSON.stringify(production.filter((file) => files.has(file)).toSorted()) !==
    JSON.stringify(expected)
  )
    return null;
  for (const [file, before, after] of manifest.files)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  for (const [file, original] of manifest.preservedBodies) {
    const harness = manifest.qualifiedHarnessBodies.find(([path]) => path === file);
    if (harness && harness[1] !== original) return null;
    if (digest("before", file) !== original || digest("after", file) !== (harness?.[2] ?? original))
      return null;
  }
  for (const [file, before, after] of manifest.qualifiedHarnessBodies)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    authority: "complete-reviewed-native-terminal-repair",
    manifestSHA256,
    originalSource: manifest.originalSource,
    reviewedSource: manifest.reviewedSource,
    completeBodyPairs: manifest.files.length,
    preservedBodyPairs: manifest.preservedBodies.length,
    qualifiedHarnessBodyPairs: manifest.qualifiedHarnessBodies.length,
    files: expected,
  };
}

export function assertNativeTerminalLocks(
  before: LockDigests,
  after: LockDigests,
  qualified: ReturnType<typeof qualifyNativeTerminalHost>,
) {
  assert.deepEqual(Object.keys(before).toSorted(), Object.keys(after).toSorted());
  for (const file of Object.keys(before)) {
    if (qualified && ["Cargo.toml", "Cargo.lock"].includes(file)) {
      const entry = manifest.files.find(([path]) => path === file);
      assert.ok(entry, "the complete Cargo source pair is mandatory");
      assert.equal(before[file], entry[1], `Foreign original Cargo source: ${file}`);
      assert.equal(after[file], entry[2], `Foreign reviewed Cargo source: ${file}`);
    } else assert.equal(after[file], before[file], `Unchanged provider/recipe lock: ${file}`);
  }
}

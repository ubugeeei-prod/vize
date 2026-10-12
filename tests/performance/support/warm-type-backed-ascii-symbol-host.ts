import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type BodyDigest = (side: "before" | "after", file: string) => string | null;
const bytes = fs.readFileSync(
  new URL("./warm-type-backed-ascii-symbol-host.json", import.meta.url),
);
const manifestSHA256 = createHash("sha256").update(bytes).digest("hex");
assert.equal(manifestSHA256, "f0fb81fe88d1da2fa3e261c5da0700182d972621e645077dfcba39a9a85229cd");
const manifest = JSON.parse(bytes.toString("utf8")) as {
  version: number;
  originalSource: string;
  reviewedPatchSHA256: string;
  reviewedSourceProofSHA256: string;
  files: Array<[string, string | null, string]>;
  preservedBodies: Array<[string, string]>;
};
assert.equal(manifest.version, 1);
assert.equal(manifest.originalSource, "5c54b1195fa8c0bf151e0a2862bf4238a3597b4e");
assert.equal(
  manifest.reviewedPatchSHA256,
  "846ca1301220997d57a8a55546783ad8e2441fe32525108dc0c8d501d1f518b7",
);
assert.equal(
  manifest.reviewedSourceProofSHA256,
  "a061ed7061035e36dc6a45e744a1c4876a1469a8c9bdd358b26bb6ceceded9c0",
);
const expectedFiles = [
  "crates/vize_maestro/src/ide/workspace_symbols/script.rs",
  "crates/vize_maestro/src/ide/workspace_symbols/script/query_tests.rs",
];
assert.deepEqual(
  manifest.files.map(([file]) => file),
  expectedFiles,
);
assert.equal(manifest.preservedBodies.length, 16);
assert.equal(
  new Set([...manifest.files, ...manifest.preservedBodies].map(([file]) => file)).size,
  18,
);
for (const [, before, after] of manifest.files) {
  if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
  assert.match(after, /^[a-f0-9]{64}$/u);
}
for (const [, original] of manifest.preservedBodies) assert.match(original, /^[a-f0-9]{64}$/u);

/** Admit only the two reviewed producer/law bodies with their whole unchanged closure. */
export function qualifyAsciiSymbolHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: BodyDigest,
) {
  if (new Set(production).size !== production.length) return null;
  const files = new Set(expectedFiles);
  const preserved = new Set(manifest.preservedBodies.map(([file]) => file));
  if (production.every((file) => alreadyQualified.has(file))) return null;
  if (production.some((file) => preserved.has(file))) return null;
  if (production.some((file) => !files.has(file) && !alreadyQualified.has(file))) return null;
  const footprint = production.filter((file) => files.has(file)).toSorted();
  if (JSON.stringify(footprint) !== JSON.stringify(expectedFiles)) return null;
  for (const [file, before, after] of manifest.files)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  for (const [file, original] of manifest.preservedBodies)
    if (digest("before", file) !== original || digest("after", file) !== original) return null;
  return {
    authority: "exact-reviewed-ascii-symbol-query-source",
    manifestSHA256,
    originalSource: manifest.originalSource,
    reviewedPatchSHA256: manifest.reviewedPatchSHA256,
    completeBodyPairs: manifest.files.length,
    preservedBodyPairs: manifest.preservedBodies.length,
    runtimeCredit: false,
    performanceCredit: false,
    files: expectedFiles,
  };
}

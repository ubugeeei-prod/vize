import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type BodyDigest = (side: "before" | "after", file: string) => string | null;
const manifest = JSON.parse(
  fs.readFileSync(
    new URL("./warm-type-backed-lexical-dollar-manifest.json", import.meta.url),
    "utf8",
  ),
) as {
  version: number;
  originalSource: string;
  reviewedSourceTree: string;
  qualification: string;
  files: Array<[string, string | null, string]>;
  corpusBodies: Array<[string, null, string]>;
  preservedBodies: Array<[string, string]>;
};
const manifestSHA256 = createHash("sha256").update(JSON.stringify(manifest)).digest("hex");
assert.equal(manifestSHA256, "c6cee7b641ef65f599a9077cfce5936317f8bc14a5670ca4e1ad5291eb780229");
assert.equal(manifest.version, 1);
assert.equal(manifest.originalSource, "a6633fb68b110d07abd5ee5a6049bc15f20f9c7f");
assert.equal(manifest.reviewedSourceTree, "adf221274a50272c6fa354df37e78c2ca73615eb");
const expectedFiles = [
  "crates/vize_canon/src/batch/executor/cli.rs",
  "crates/vize_canon/src/batch/executor/cli/lexical_dollar_receipt.rs",
  "crates/vize_canon/src/batch/executor/session.rs",
  "crates/vize_canon/src/batch/executor/session/lexical_dollar_receipt.rs",
  "crates/vize_canon/src/batch/type_checker/tests/recent_issues.rs",
  "crates/vize_canon/src/batch/type_checker/tests/recent_issues/lexical_dollar_slots.rs",
  "crates/vize_canon/src/virtual_ts/scope/globals/instance.rs",
  "crates/vize_canon/src/virtual_ts/scope/globals/instance/tests.rs",
  "crates/vize_canon/src/virtual_ts/scope/globals/instance/tests/lexical_scope.rs",
];
assert.deepEqual(
  manifest.files.map(([file]) => file),
  expectedFiles,
);
assert.equal(manifest.corpusBodies.length, 21);
assert.equal(manifest.preservedBodies.length, 7);
assert.equal(
  new Set(
    [...manifest.files, ...manifest.corpusBodies, ...manifest.preservedBodies].map(
      ([file]) => file,
    ),
  ).size,
  37,
);
for (const [, before, after] of [...manifest.files, ...manifest.corpusBodies]) {
  if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
  assert.match(after, /^[a-f0-9]{64}$/u);
}

/** Admit these complete source bodies; the unchanged original400 still must execute. */
export function qualifyLexicalDollarHost(
  production: string[],
  alreadyQualified: ReadonlySet<string>,
  digest: BodyDigest,
) {
  if (new Set(production).size !== production.length) return null;
  const files = new Set(expectedFiles);
  if (production.every((file) => alreadyQualified.has(file))) return null;
  if (production.some((file) => !alreadyQualified.has(file) && !files.has(file))) return null;
  const footprint = production.filter((file) => files.has(file)).toSorted();
  if (JSON.stringify(footprint) !== JSON.stringify(expectedFiles)) return null;
  for (const [file, before, after] of [...manifest.files, ...manifest.corpusBodies])
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  for (const [file, original] of manifest.preservedBodies)
    if (digest("before", file) !== original || digest("after", file) !== original) return null;
  return {
    authority: "exact-reviewed-lexical-dollar-scope-source",
    manifestSHA256,
    originalSource: manifest.originalSource,
    reviewedSourceTree: manifest.reviewedSourceTree,
    completeBodyPairs: manifest.files.length,
    completeCorpusBodyPairs: manifest.corpusBodies.length,
    preservedBodyPairs: manifest.preservedBodies.length,
    runtimeCredit: false,
    performanceCredit: false,
    files: expectedFiles,
  };
}

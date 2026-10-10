import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type BodyDigest = (side: "before" | "after", file: string) => string | null;
const manifest = JSON.parse(
  fs.readFileSync(
    new URL("./warm-type-backed-default-migration-manifest.json", import.meta.url),
    "utf8",
  ),
) as {
  version: number;
  originalSource: string;
  reviewedSource: string;
  files: Array<[string, string | null, string]>;
  preservedBodies: Array<[string, string]>;
};
const manifestSHA256 = createHash("sha256").update(JSON.stringify(manifest)).digest("hex");
assert.equal(manifestSHA256, "c9afe6f838b54bbebacce0bc4ed1d476b57e5a1454eb341e4a61b8ab091acde2");
assert.equal(manifest.version, 1);
assert.equal(manifest.originalSource, "e5702be6d0119a716499827fa7b918ce8c9dcbae");
assert.equal(manifest.reviewedSource, "a7b5a205c44f25dee8f1105bc0d11077827b5f80");
const expectedFiles = [
  "crates/vize_maestro/src/ide/diagnostics.rs",
  "crates/vize_maestro/src/ide/diagnostics/default_migration_tests.rs",
  "crates/vize_patina/src/linter/compatibility.rs",
  "crates/vize_patina/src/linter/engine/rule_sets.rs",
  "crates/vize_patina/src/preset.rs",
  "crates/vize_patina/src/preset/default_migration_tests.rs",
  "crates/vize_patina/src/preset/snapshots/vize_patina__preset__tests__lint_preset_rule_membership.snap",
  "crates/vize_patina/src/rule.rs",
  "crates/vize_patina/src/rule/happy_path.rs",
  "crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs",
];
assert.deepEqual(
  manifest.files.map(([file]) => file),
  expectedFiles,
);
assert.equal(manifest.preservedBodies.length, 9);
for (const [, before, after] of manifest.files) {
  if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
  assert.match(after, /^[a-f0-9]{64}$/u);
}

/** Admit this complete reviewed slice for actual measurement, never a partial skip. */
export function qualifyDefaultMigrationHost(
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
  for (const [file, before, after] of manifest.files)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  for (const [file, original] of manifest.preservedBodies)
    if (digest("before", file) !== original || digest("after", file) !== original) return null;
  return {
    authority: "complete-reviewed-default-migration-slice",
    manifestSHA256,
    originalSource: manifest.originalSource,
    reviewedSource: manifest.reviewedSource,
    completeBodyPairs: manifest.files.length,
    preservedBodyPairs: manifest.preservedBodies.length,
    files: expectedFiles,
  };
}

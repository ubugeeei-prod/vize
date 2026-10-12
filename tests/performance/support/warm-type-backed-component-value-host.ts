import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type BodyDigest = (side: "before" | "after", file: string) => string | null;
const manifest = JSON.parse(
  fs.readFileSync(
    new URL("./warm-type-backed-component-value-manifest.json", import.meta.url),
    "utf8",
  ),
) as {
  version: number;
  originalSource: string;
  reviewedSourceBodySetSHA256: string;
  qualification: string;
  files: Array<[string, string | null, string]>;
  supplementalBodies: Array<[string, null, string]>;
  preservedBodies: Array<[string, string]>;
};
const manifestSHA256 = createHash("sha256").update(JSON.stringify(manifest)).digest("hex");
assert.equal(manifestSHA256, "b1c168d004be1f7410c1e8ec1a3097a7313dec4d0cf3e6921a2c354d51acb219");
assert.equal(manifest.version, 1);
assert.equal(manifest.originalSource, "b82f3080f1fa028ca07e03bcd42700742271613f");
const expectedFiles = [
  "crates/vize/tests/lint_component_value_reads_cli.rs",
  "crates/vize_croquis/src/binding_occurrences.rs",
  "crates/vize_croquis/src/drawer/helpers/identifiers.rs",
  "crates/vize_croquis/src/drawer/helpers/identifiers/ast.rs",
  "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk.rs",
  "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk/assignment_target.rs",
  "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk/facts.rs",
  "crates/vize_croquis/src/drawer/helpers/identifiers/ast/walk/scopes.rs",
  "crates/vize_croquis/src/drawer/occurrences.rs",
  "crates/vize_croquis/src/drawer/template/ids.rs",
  "crates/vize_croquis/src/setup_context/ssr.rs",
  "crates/vize_patina/src/linter/engine/rule_sets.rs",
  "crates/vize_patina/src/linter/engine/sfc.rs",
  "crates/vize_patina/src/rules/vue/no_unused_components.rs",
  "crates/vize_patina/tests/component_import_value_reads.rs",
];
assert.deepEqual(
  manifest.files.map(([file]) => file),
  expectedFiles,
);
assert.equal(manifest.supplementalBodies.length, 52);
assert.equal(manifest.preservedBodies.length, 13);
assert.equal(
  new Set(
    [...manifest.files, ...manifest.supplementalBodies, ...manifest.preservedBodies].map(
      ([file]) => file,
    ),
  ).size,
  80,
);
for (const [, before, after] of [...manifest.files, ...manifest.supplementalBodies]) {
  if (before !== null) assert.match(before, /^[a-f0-9]{64}$/u);
  assert.match(after, /^[a-f0-9]{64}$/u);
}

/** Finite complete source admission; this supplies no workload or performance verdict. */
export function qualifyComponentValueHost(
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
  for (const [file, before, after] of [...manifest.files, ...manifest.supplementalBodies])
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  for (const [file, original] of manifest.preservedBodies)
    if (digest("before", file) !== original || digest("after", file) !== original) return null;
  return {
    authority: "exact-reviewed-component-template-value-read-source",
    manifestSHA256,
    originalSource: manifest.originalSource,
    reviewedSourceBodySetSHA256: manifest.reviewedSourceBodySetSHA256,
    completeBodyPairs: 15,
    supplementalBodyPairs: 52,
    preservedBodyPairs: 13,
    runtimeCredit: false,
    performanceCredit: false,
    files: expectedFiles,
  };
}

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type BodyDigest = (side: "before" | "after", file: string) => string | null;
type ComponentManifest = {
  version: number;
  originalSource: string;
  reviewedSourceBodySetSHA256: string;
  qualification: string;
  files: Array<[string, string | null, string]>;
  supplementalBodies: Array<[string, null, string]>;
  preservedBodies: Array<[string, string]>;
};
const mainManifest = "tests/performance/support/warm-type-backed-component-value-manifest.json";
const originalsFile = "tests/performance/support/warm-type-backed-component-value-originals.json";
const originalsSHA256 = "2ea061f1ba1acd1ba3b5c55ac9df98f4da5e90ba93bf0812791f89737e6007f4";
const sha = (body: Buffer | string) => createHash("sha256").update(body).digest("hex");

/** Decode the same complete table, split solely to retain the 350-line source budget. */
export function loadComponentValueManifest(read: (file: string) => Buffer): ComponentManifest {
  const stored = JSON.parse(read(mainManifest).toString("utf8")) as Omit<
    ComponentManifest,
    "preservedBodies"
  > & { preservedBodies: string };
  assert.equal(
    sha(JSON.stringify(stored)),
    "27d090db7cfdfda0acc51bb6021d5974933f81f9e4c6c31b18802e7944382cd9",
  );
  assert.equal(stored.preservedBodies, originalsFile);
  const originalBytes = read(originalsFile);
  assert.equal(sha(originalBytes), originalsSHA256);
  const manifest: ComponentManifest = {
    ...stored,
    preservedBodies: JSON.parse(originalBytes.toString("utf8")),
  };
  assert.equal(
    sha(JSON.stringify(manifest)),
    "79f03fc6a588acecbeb71ce6081dfc590bfe20322771312e019e5982db95d494",
  );
  return manifest;
}
export const componentValueMetadataPairs: Array<[string, null, string]> = [
  [originalsFile, null, originalsSHA256],
];
const manifest = loadComponentValueManifest((file) =>
  fs.readFileSync(new URL("../../../" + file, import.meta.url)),
);
const manifestSHA256 = sha(JSON.stringify(manifest));
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
  for (const [file, before, after] of [
    ...manifest.files,
    ...manifest.supplementalBodies,
    ...componentValueMetadataPairs,
  ])
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
    sourceMetadataBodyPairs: 1,
    runtimeCredit: false,
    performanceCredit: false,
    files: expectedFiles,
  };
}

export const componentValueRetainedGuardPairs: Array<[string, string | null, string]> = [
  [
    "tests/tooling/props-deep-import-source-custody.test.mjs",
    "4c23bdcb6fbbcc6080f77c47f0bcb16a846c529e6d3f0af9877aed867d0c5250",
    "14d48e407eeea2d5d52cf5dbe08b03109d2176c77e8648ee79a4f155e0940e26",
  ],
  [
    "tests/tooling/props-deep-trait-scope-source-custody.test.mjs",
    "1094a2ceb84b0d6f98f1d1b5d6d8ec9a88f7ce17090c413236e53cd47b54949b",
    "69921f0604d47615da51294fc092e2186c173f802d09881a6892c0ec66c57f6a",
  ],
  [
    "tests/tooling/props-deep-destructure-reference.test.mjs",
    "d6db39c5ffc0757c8e9b274534f2ec7c5ce0a01519693e1121a48247b083efa8",
    "612a60de4457d5ccaa7a0518177c7067d74547bbf2f567dc6ea67c45e247eaef",
  ],
  [
    "tests/tooling/runtime-props-api-source-custody.test.mjs",
    "80fa07710caeeff2624bba8d1e97aa9eefc3a0d835026cf6b8c85bf6b766082e",
    "771379c8843e5690313c727195062c8b5fa6c6744c0437c99b5ba188a62771eb",
  ],
  [
    "tests/tooling/setup-export-default-reference.test.mjs",
    "63b83fe1266802edad926ab8f4f171e402862ea74a42ae54e3a49834c8fc2e0b",
    "e2d2a0cb8ffa671b2cf0a28e85ae9334e543d30b71bc95ed869cf59c14173770",
  ],
  [
    "tests/tooling/reserved-prefix-api-order.test.mjs",
    "412308fd44bd32e0ad0a7f6c48b8ef6acc6c6b82901a32b15c88d5f28a5e7b1a",
    "10a9c04e3a67becca8e72c05ab99d70bf10068f706e19801ad36c6e734cda2b8",
  ],
  [
    "tests/tooling/reserved-suppression-default-routes.test.mjs",
    "c484cd05219c4c8d1e34664544cf9886743c3c2d401c29bcec4660c6537a7243",
    "b365bd95b170e0dfc4521ef7b46ee3abd2dab4bed2772aeb12f7441a2fae3ae9",
  ],
  [
    "tests/tooling/bare-reserved-default-route-reference.test.mjs",
    "bdf0b7cc533a3eefc3ec086b12e5c10de9bb65f79711b909a8daa914a658f772",
    "9edff9a447f71a4dbb856bd69841980e6855a15f3f01e07b5965f70059066283",
  ],
  [
    "tests/tooling/warm-type-backed-default-migration-host.test.ts",
    "5084578fa77c215ee7555708d9cc6284826a36dced1a9f31c5e6b417a1447704",
    "a81dd0613ce3715f7d759e00039f444963b3edb2462ee82f5dadfd1bbc4e1704",
  ],
  [
    "tests/tooling/lint-props-deep-destructure.test.mjs",
    "1a13bd091f9a2e0108dcd5455896039952ba3e420cd843c37d81aef477c53274",
    "f6a32e6597d3592df94faa940cd2f44a8cd2dc437bf4b68fd5c5971474043333",
  ],
  [
    "tests/tooling/support/component-value-retained-guards.ts",
    null,
    "30fb49b0260eee2f5292d8edc7a5445d049a0c583e39eee019873b5ed1b6a847",
  ],
  [
    "tests/_fixtures/differential/linter/component-import-value-host-before/retained-guard-callers.json",
    null,
    "9d0ed3a82979fe10f0ff4afed1747f063c646e77b575346fa2e68c016be0944c",
  ],
  [
    "tests/tooling/component-value-retained-guards.test.ts",
    null,
    "0ce8aabec375b654a4299b1da3493715cc4346f57cab743f0afe4e22076c50e4",
  ],
];

/** Exact test-only caller custody; the original component workload qualifier stays separate. */
export function qualifyComponentValueRetainedGuards(files: string[], digest: BodyDigest) {
  const required = componentValueRetainedGuardPairs.map(([file]) => file).toSorted();
  if (
    new Set(files).size !== files.length ||
    JSON.stringify(files.toSorted()) !== JSON.stringify(required)
  )
    return null;
  for (const [file, before, after] of componentValueRetainedGuardPairs)
    if (digest("before", file) !== before || digest("after", file) !== after) return null;
  return {
    completeGuardBodyPairs: 13,
    originalCallerSource: "5bc5759ef97f574fec526ea7ef8664e8592bd54c",
    runtimeCredit: false,
    performanceCredit: false,
  };
}

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { hash, record, run, save } from "./cross-file-finalization/capture.ts";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const output = path.join(repo, "target/vize-tests/cross-file-finalization-order");
const physical = path.join(output, "shared-physical-inputs");
const baseRoot = path.join(repo, "target/finalization-order-base");
fs.mkdirSync(physical, { recursive: true });
const git = (cwd: string, ...args: string[]) =>
  execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const headSha = git(repo, "rev-parse", "HEAD");
const baseSha = git(baseRoot, "rev-parse", "HEAD");
assert.equal(headSha, process.env.EXPECTED_SOURCE_SHA);
assert.match(baseSha, /^[a-f0-9]{40}$/u);
assert.equal(baseSha, process.env.BASE_SOURCE_SHA);
const overlay = path.join(repo, "tests/tooling/support/cross-file-finalization-order/observer");
const observerInputs = fs.readdirSync(overlay).map((name) => ({
  name,
  source: fs.readFileSync(path.join(overlay, name), "utf8"),
  sha256: hash(fs.readFileSync(path.join(overlay, name))),
}));
const providers = [
  {
    name: "base",
    sha: baseSha,
    root: baseRoot,
    binary: path.join(
      repo,
      "target/finalization-order-base-build/ci/examples/cross_file_finalization_order",
    ),
  },
  {
    name: "head",
    sha: headSha,
    root: repo,
    binary: path.join(
      repo,
      "target/finalization-order-head-build/ci/examples/cross_file_finalization_order",
    ),
  },
];
const identities = providers.map((provider) => {
  assert.equal(
    git(
      provider.root,
      "status",
      "--porcelain",
      "--untracked-files=no",
      "--",
      "crates",
      "davinci",
      "Cargo.lock",
    ),
    "",
  );
  const installed = path.join(provider.root, "crates/vize/examples/cross_file_finalization_order");
  for (const input of observerInputs)
    assert.equal(hash(fs.readFileSync(path.join(installed, input.name))), input.sha256);
  const accessors = fs.readFileSync(
    path.join(provider.root, "crates/vize_croquis_cf/src/analyzer/core/accessors.rs"),
    "utf8",
  );
  const deps = fs.readFileSync(
    path.join(provider.root, "crates/vize_croquis_cf/src/analyzer/core/deps.rs"),
    "utf8",
  );
  const resetStaleCapabilities = {
    resetTemplateFacts: accessors.includes("self.template_complexity.clear();"),
    reconcileObsoleteImports: deps.includes("fn clear_obsolete_import_edges("),
  };
  assert.ok(
    Object.values(resetStaleCapabilities).every(Boolean),
    "both providers must include both integrated correctness fixes",
  );
  return {
    ...provider,
    binarySha256: hash(fs.readFileSync(provider.binary)),
    lockSha256: hash(fs.readFileSync(path.join(provider.root, "Cargo.lock"))),
    cargoManifestSha256: hash(fs.readFileSync(path.join(provider.root, "Cargo.toml"))),
    cargoConfigTree: git(provider.root, "rev-parse", "HEAD:.cargo"),
    resetStaleCapabilities,
    unchangedProduction: Object.fromEntries(
      [
        "crates/vize_croquis_cf/src/analyzer/core/accessors.rs",
        "crates/vize_croquis_cf/src/analyzer/core/deps.rs",
      ].map((name) => [name, hash(fs.readFileSync(path.join(provider.root, name)))]),
    ),
    unchangedTrees: Object.fromEntries(
      ["crates/vize_croquis/src", "crates/vize_atelier_sfc/src", "davinci/vize_l0"].map((name) => [
        name,
        git(provider.root, "rev-parse", `HEAD:${name}`),
      ]),
    ),
    observerInputs,
    sourceTrees: Object.fromEntries(
      ["crates/vize_croquis_cf/src", "crates/vize/src", "davinci/vize_l0"].map((name) => [
        name,
        git(provider.root, "rev-parse", `HEAD:${name}`),
      ]),
    ),
    build: [
      "cargo",
      "build",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize",
      "--example",
      "cross_file_finalization_order",
      "--target-dir",
      path.dirname(path.dirname(path.dirname(provider.binary))),
    ],
  };
});
const [baseIdentity, headIdentity] = identities;
assert.ok(baseIdentity && headIdentity);
assert.deepEqual(headIdentity.resetStaleCapabilities, baseIdentity.resetStaleCapabilities);
assert.deepEqual(headIdentity.unchangedProduction, baseIdentity.unchangedProduction);
assert.deepEqual(headIdentity.unchangedTrees, baseIdentity.unchangedTrees);
assert.equal(headIdentity.lockSha256, baseIdentity.lockSha256);
assert.equal(headIdentity.cargoManifestSha256, baseIdentity.cargoManifestSha256);
assert.equal(headIdentity.cargoConfigTree, baseIdentity.cargoConfigTree);
const changedCore = git(
  repo,
  "diff",
  "--name-only",
  baseSha,
  "HEAD",
  "--",
  "crates/vize_croquis_cf/src",
)
  .split("\n")
  .filter(Boolean);
assert.deepEqual(
  new Set(changedCore),
  new Set([
    "crates/vize_croquis_cf/src/analyzer/core/files.rs",
    "crates/vize_croquis_cf/src/graph.rs",
    "crates/vize_croquis_cf/src/graph/order.rs",
    "crates/vize_croquis_cf/src/graph/order/tests.rs",
  ]),
);
save(output, "provider-identities.json", {
  identities,
  combinedResetStaleQualified: true,
  rustc: execFileSync("rustc", ["--version", "--verbose"], { encoding: "utf8" }),
  note: "Identical observer-only example overlays are explicitly hashed. Tracked production trees remain clean. Both providers run on the identical physical input path with identical byte contents and fixed input mtimes.",
});

for (const provider of identities) {
  const actual = run(output, provider.name, provider.binary, [physical], repo);
  assert.equal(actual.packet.status, 0);
  fs.copyFileSync(
    path.join(physical, "complete-observations.json"),
    path.join(output, provider.name + ".complete.json"),
  );
  fs.cpSync(
    path.join(physical, "fixtures"),
    path.join(output, provider.name + "-physical-fixtures"),
    { recursive: true },
  );
  assert.equal(hash(fs.readFileSync(provider.binary)), provider.binarySha256);
}
const before = record(JSON.parse(fs.readFileSync(path.join(output, "base.complete.json"), "utf8")));
const after = record(JSON.parse(fs.readFileSync(path.join(output, "head.complete.json"), "utf8")));
assert.equal(
  record(before.summary).rawGraphChanges,
  8,
  "actual old one/two-pass ordering regression must reproduce",
);
assert.equal(record(after.summary).rawGraphChanges, 0);
assert.ok(Array.isArray(before.observations) && Array.isArray(after.observations));
assert.equal(before.observations.length, 62);
assert.equal(after.observations.length, 62);
const fields = [
  "registry",
  "rawNodes",
  "allForwardRelations",
  "allReverseRelations",
  "orderedPerKind",
  "orderedComponentRelations",
  "entries",
  "cycles",
  "analysisCompleteWithTimeSplit",
];
for (const observed of [before, after]) {
  const retained = record(observed.retainedOwner);
  assert.equal(
    retained.staleImportRetained,
    false,
    "actual retained-owner edit must remove the stale import",
  );
  assert.deepEqual(
    retained.orphanTemplateIdsAfterClear,
    [],
    "actual reset must clear both old template facts",
  );
}
for (const state of ["original", "edited", "cleared", "repaired"]) {
  const previous = record(record(before.retainedOwner)[state]);
  const current = record(record(after.retainedOwner)[state]);
  for (const field of fields)
    assert.deepEqual(current[field], previous[field], `retained/${state}/${field}`);
}
const comparisons: unknown[] = [];
for (const [index, raw] of after.observations.entries()) {
  const current = record(raw);
  const previous = record(before.observations[index]);
  for (const key of ["case", "order", "rawScriptRegistry", "inputs"])
    assert.deepEqual(current[key], previous[key]);
  for (const pass of ["first", "second", "third"]) {
    const state = record(current[pass]);
    for (const reference of [
      record(previous.second),
      record(previous.third),
      record(current.first),
    ]) {
      for (const field of fields)
        assert.deepEqual(
          state[field],
          reference[field],
          `${index}/${pass}/${field}: complete actual state`,
        );
    }
    assert.equal(typeof state.analysisCompleteRaw, "string");
    assert.equal(typeof state.analysisTimeMs, "number");
  }
  comparisons.push({
    case: current.case,
    order: current.order,
    rawScriptRegistry: current.rawScriptRegistry,
    newFirstEqualsOldSecondAndThird: true,
    newFirstSecondThirdIdenticalGraphAndSemanticFacts: true,
  });
}
save(output, "qualification.json", {
  passed: true,
  combinedResetStaleQualified: true,
  sourceSha: headSha,
  baseSha,
  cases: comparisons.length,
  comparisons,
  identities,
  note: "All original ordered graph vectors, registry facts and complete native analysis observations remain retained. Full graph identity includes ordering; no vector normalization or sorting. Native elapsed analysis duration remains an independent raw observation; the observer also retains its explicit separate time-split semantic projection. This is functional source qualification, not a performance claim.",
});
process.stdout.write(
  "cross-file finalization ordering: 62 actual source states preserve old second/third raw vectors\n",
);

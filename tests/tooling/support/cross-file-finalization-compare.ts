import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { files } from "./cross-file-finalization/corpus.ts";
import { hash, run, save, sourceProfile } from "./cross-file-finalization/capture.ts";

const repo = fileURLToPath(new URL("../../../", import.meta.url));
const output = path.join(repo, "target/vize-tests/cross-file-finalization-compare");
const project = path.join(output, "physical-project");
fs.mkdirSync(project, { recursive: true });
const git = (cwd: string, ...args: string[]) =>
  execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
const baseSha = process.env.BASE_SOURCE_SHA;
assert.match(baseSha ?? "", /^[a-f0-9]{40}$/u);
assert.ok(baseSha);
const providers = [
  {
    name: "base",
    sha: baseSha,
    root: path.join(repo, "target/finalization-compare-base"),
    binary: path.join(repo, "target/finalization-compare-base-build/ci-opt/vize"),
  },
  {
    name: "head",
    sha: process.env.EXPECTED_SOURCE_SHA,
    root: repo,
    binary: path.join(repo, "target/finalization-compare-head-build/ci-opt/vize"),
  },
];
const sameTrees = [
  "crates/vize_croquis_cf/src",
  "crates/vize_croquis/src",
  "crates/vize_atelier_sfc/src",
  "davinci/vize_l0",
];
const identities = providers.map((provider) => {
  assert.equal(git(provider.root, "rev-parse", "HEAD"), provider.sha);
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
  const accessors = fs.readFileSync(
    path.join(provider.root, "crates/vize_croquis_cf/src/analyzer/core/accessors.rs"),
    "utf8",
  );
  const deps = fs.readFileSync(
    path.join(provider.root, "crates/vize_croquis_cf/src/analyzer/core/deps.rs"),
    "utf8",
  );
  return {
    ...provider,
    binarySha256: hash(fs.readFileSync(provider.binary)),
    lockSha256: hash(fs.readFileSync(path.join(provider.root, "Cargo.lock"))),
    cargoManifestSha256: hash(fs.readFileSync(path.join(provider.root, "Cargo.toml"))),
    cargoConfigTree: git(provider.root, "rev-parse", "HEAD:.cargo"),
    productionTrees: Object.fromEntries(
      sameTrees.map((name) => [name, git(provider.root, "rev-parse", `HEAD:${name}`)]),
    ),
    resetStaleCapabilities: {
      resetTemplateFacts: accessors.includes("self.template_complexity.clear();"),
      reconcileObsoleteImports: deps.includes("fn clear_obsolete_import_edges("),
    },
    profile: { name: "ci-opt", inherits: "release", lto: "thin", codegenUnits: 16 },
    compilation: [
      "cargo",
      "build",
      "--locked",
      "--profile",
      "ci-opt",
      "-p",
      "vize",
      "--target-dir",
      path.dirname(path.dirname(provider.binary)),
    ],
  };
});
const [base, head] = identities;
assert.ok(base && head);
assert.deepEqual(
  head.productionTrees,
  base.productionTrees,
  "both providers must have identical core/parser/profiler source",
);
assert.deepEqual(
  head.resetStaleCapabilities,
  base.resetStaleCapabilities,
  "reset/stale fixes must match on both sides",
);
assert.equal(head.lockSha256, base.lockSha256);
assert.equal(head.cargoManifestSha256, base.cargoManifestSha256);
assert.equal(head.cargoConfigTree, base.cargoConfigTree);
const changed = git(repo, "diff", "--name-only", baseSha, "HEAD", "--", "crates", "davinci")
  .split("\n")
  .filter(Boolean);
assert.deepEqual(
  new Set(changed),
  new Set([
    "crates/vize/src/commands/lint/cross_file.rs",
    "crates/vize_vitrine/src/wasm/cross_file.rs",
    "crates/vize_croquis_cf/benches/cross_file.rs",
  ]),
);
const combinedResetStaleQualified = Object.values(head.resetStaleCapabilities).every(Boolean);
save(output, "provider-identities.json", {
  identities,
  combinedResetStaleQualified,
  rustc: execFileSync("rustc", ["--version", "--verbose"], { encoding: "utf8" }),
  cargo: execFileSync("cargo", ["--version"], { encoding: "utf8" }),
  note: "Final integration requires both actual providers to include the same reset/stale fixes. The observed capability flags remain retained even if this precondition fails.",
});
assert.ok(
  combinedResetStaleQualified,
  "final comparison requires both integrated reset/stale correctness fixes on both providers",
);

for (const name of ["corpus.ts", "capture.ts"]) {
  const relative = `tests/tooling/support/cross-file-finalization/${name}`;
  assert.equal(
    hash(fs.readFileSync(path.join(repo, relative))),
    hash(fs.readFileSync(path.join(base.root, relative))),
    "frozen corpus/observer bytes",
  );
}
const inputs = Object.entries(files()).map(([name, source]) => {
  fs.writeFileSync(path.join(project, name), source);
  const stat = fs.statSync(path.join(project, name), { bigint: true });
  return {
    path: name,
    source,
    bytes: Buffer.byteLength(source),
    sha256: hash(source),
    mtimeNs: stat.mtimeNs.toString(),
    mode: stat.mode.toString(),
  };
});
assert.equal(inputs.length, 241);
save(output, "complete-inputs.json", inputs);
const flags = ["lint", "--no-config", "--cross-file-complexity", "--help-level", "short"];
const beforeJson = run(
  output,
  "base-json",
  base.binary,
  [...flags, "--format", "json", "*.vue"],
  project,
);
const afterJson = run(
  output,
  "head-json",
  head.binary,
  [...flags, "--format", "json", "*.vue"],
  project,
);
assert.deepEqual(afterJson.stdout, beforeJson.stdout, "entire diagnostic packet");
assert.equal(afterJson.packet.status, beforeJson.packet.status);
const expected = run(
  output,
  "base-plain",
  base.binary,
  [...flags, "--format", "plain", "*.vue"],
  project,
);

function counts(profile: ReturnType<typeof sourceProfile>) {
  const all = new Map(profile.counters.map((row) => [row.key, row]));
  const value = (key: string) => {
    const observed = all.get("croquis_cf.dependencies." + key)?.total;
    assert.equal(typeof observed, "number");
    return observed as number;
  };
  return {
    finalizations: value("finalize.calls"),
    finalizedFiles: value("finalize.files"),
    updates: value("update.calls"),
  };
}
const observations: {
  ordinal: number;
  warmup: boolean;
  provider: string;
  enabled: boolean;
  packet: unknown;
  sourcePhases: ReturnType<typeof sourceProfile> | null;
}[] = [];
for (let ordinal = 0; ordinal < 9; ordinal++) {
  const order = ordinal % 2 ? [head, base] : [base, head];
  for (const enabled of ordinal % 2 ? [true, false] : [false, true]) {
    for (const provider of order) {
      const label = `${ordinal}-${provider.name}-${enabled ? "profiled" : "disabled"}`;
      const profileFlags = enabled
        ? ["--profile-json", path.join(output, label + ".profile.json")]
        : [];
      const actual = run(
        output,
        label,
        provider.binary,
        [...flags, "--format", "plain", ...profileFlags, "*.vue"],
        project,
      );
      assert.deepEqual(actual.stdout, expected.stdout, label + ": entire report/diagnostics");
      assert.equal(actual.packet.status, expected.packet.status);
      observations.push({
        ordinal,
        warmup: ordinal < 2,
        provider: provider.name,
        enabled,
        packet: actual.packet,
        sourcePhases: enabled ? sourceProfile(output, label) : null,
      });
      save(output, "complete-observations.json", observations);
    }
  }
}
const measured = observations.filter((row) => row.enabled && !row.warmup);
const actualCounts = measured.map((row) => {
  assert.ok(row.sourcePhases);
  return { ordinal: row.ordinal, provider: row.provider, counts: counts(row.sourcePhases) };
});
save(output, "actual-work-counts.json", actualCounts);
for (let ordinal = 2; ordinal < 9; ordinal++) {
  const before = actualCounts.find((row) => row.ordinal === ordinal && row.provider === "base");
  const after = actualCounts.find((row) => row.ordinal === ordinal && row.provider === "head");
  assert.ok(before && after);
  assert.equal(before.counts.finalizations, 2);
  assert.equal(after.counts.finalizations, 1);
  assert.ok(
    after.counts.updates < before.counts.updates,
    "actual updates must fall, not inferred static inputs",
  );
  assert.equal(
    before.counts.updates - before.counts.finalizedFiles,
    after.counts.updates - after.counts.finalizedFiles,
    "actual registration work unchanged",
  );
}
for (const provider of identities)
  assert.equal(hash(fs.readFileSync(provider.binary)), provider.binarySha256);
for (const input of inputs) {
  const stat = fs.statSync(path.join(project, input.path), { bigint: true });
  assert.equal(stat.mtimeNs.toString(), input.mtimeNs);
  assert.equal(stat.mode.toString(), input.mode);
  assert.equal(hash(fs.readFileSync(path.join(project, input.path))), input.sha256);
}
save(output, "qualification.json", {
  passed: true,
  sourceSha: head.sha,
  baseSha,
  combinedResetStaleQualified,
  identities,
  files: inputs.length,
  measuredRounds: 7,
  warmupRounds: 2,
  actualCounts,
  observations,
  diagnosticsSha256: hash(afterJson.stdout),
  reportSha256: hash(expected.stdout),
  note: "Actual isolated optimized source providers and counters. All original profiles, packets and whole-process timings retained; setup/compilation excluded. No whole-tool speedup or instruction-ceiling claim. Combined sibling integration is qualified only when explicitly true above.",
});
process.stdout.write(
  "cross-file finalization: complete packets match and actual repeated work decreases\n",
);

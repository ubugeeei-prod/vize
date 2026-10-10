// Davinci bench artifacts are only comparable when they describe the same
// fixture, platform, and harness version.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const comparePath = path.join(repoRoot, "tools", "commands", "davinci", "bench-compare.rs");
const fixtureDir = path.join(repoRoot, "tests", "_fixtures", "davinci-bench-compare");

function runCompare(root: string, budgets: string, baseline: string, current: string) {
  const env = { ...process.env };
  delete env.DAVINCI_BASELINE_REFRESH;
  return spawnSync(
    "rust-script",
    [comparePath, "--budgets", budgets, "--baseline", baseline, "--results", current],
    { cwd: root, encoding: "utf8", env },
  );
}

function writeBudget(pathname: string) {
  fs.writeFileSync(
    pathname,
    "[bench]\nfixture_parse_small = { wall_p50_ns = 100000, allocs = 42, " +
      "rss_peak_bytes = 8192, wall_tolerance = 0.05 }\n",
  );
}

function runIdentityMismatch(field: string, value: string) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-bench-identity-"));
  const baseline = path.join(root, "baseline");
  const current = path.join(root, "current");
  const budgets = path.join(root, "budgets.toml");
  fs.mkdirSync(baseline);
  fs.mkdirSync(current);
  writeBudget(budgets);
  const report = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "baseline", "fixture_parse_small.json"), "utf8"),
  );
  fs.writeFileSync(path.join(baseline, "fixture_parse_small.json"), `${JSON.stringify(report)}\n`);
  const changed = structuredClone(report);
  changed[field] = value;
  fs.writeFileSync(path.join(current, "fixture_parse_small.json"), `${JSON.stringify(changed)}\n`);
  const result = runCompare(root, budgets, baseline, current);
  fs.rmSync(root, { recursive: true, force: true });
  return { budgets, baseline, current, result };
}

test("bench-compare rejects mismatched baseline/current artifact identity", () => {
  for (const { field, value, original } of [
    {
      field: "fixture",
      original: "synthetic:bench-compare-fixture",
      value: "synthetic:other-fixture",
    },
    { field: "platform", original: "linux", value: "macos" },
    { field: "harness_version", original: "0.0.0-fixture", value: "0.0.1-fixture" },
  ]) {
    const { budgets, baseline, current, result } = runIdentityMismatch(field, value);
    assert.equal(result.status, 2, result.stderr);
    assert.equal(
      result.stdout,
      `bench-compare: budgets=${budgets} baseline=${baseline} results=${current}\n`,
    );
    assert.equal(
      result.stderr,
      `bench-compare: fixture_parse_small baseline/current ${field} mismatch: ${original} vs ${value}\n`,
    );
  }
});

// Additional opt-in source-law controls; the original default oracle above is unchanged.
const { sourceJudgeFixture } = await import("./support/bench-source-harness-fixture.mjs");
function sourceCompare(f: ReturnType<typeof sourceJudgeFixture>, args: string[] = []) {
  return spawnSync(
    "rust-script",
    [comparePath, ...f.args, "--source-harness-pair", f.pair, ...args],
    { cwd: repoRoot, encoding: "utf8", env: { ...process.env, DAVINCI_BASELINE_REFRESH: "0" } },
  );
}

await test("source-owned different raw versions retain the closed default and all numeric gates", (t) => {
  const f = sourceJudgeFixture(t);
  const original = spawnSync("rust-script", [comparePath, ...f.args], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(original.status, 2, original.stderr);
  assert.match(original.stderr, /harness_version mismatch/);
  const positive = sourceCompare(f);
  assert.equal(positive.status, 0, positive.stderr);
  assert.match(positive.stdout, /breaches=0 .*registered=100/);
  const id = Object.keys(f.budgets.bench).find((id) => f.budgets.bench[id].allocs > 0)!;
  for (const delta of [1, -1]) {
    f.writeReport("head", 1, id, { allocs: f.budgets.bench[id].allocs + delta });
    f.save();
    const result = sourceCompare(f);
    assert.equal(result.status, 1, result.stderr);
    assert.match(result.stdout, /FAIL .* allocs .*exact gate/);
  }
  f.writeReport("head", 1, id, { allocs: f.budgets.bench[id].allocs });
  f.save();
  f.carrier.current = { arm: "base", repetition: 2 };
  f.carrier.baseline = { arm: "base", repetition: 2 };
  f.write(f.pair, f.carrier);
  f.args[f.args.indexOf("--results") + 1] = path.join(f.out, "base/run-2/level-reports");
  f.args[f.args.indexOf("--baseline") + 1] = path.join(f.out, "base/run-2/level-reports");
  const self = sourceCompare(f);
  assert.equal(self.status, 0, self.stderr);
});

await test("source-owned comparison refuses claim-only, incomplete, foreign and wrong-plane custody", (t) => {
  const f = sourceJudgeFixture(t);
  const refuse = () => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
  };
  f.write(f.pair, { baseline_version: "0.441.0", current_version: "0.442.0" });
  refuse();
  f.write(f.pair, f.carrier);
  f.measurement.observations_complete = false;
  f.save();
  refuse();
  f.measurement.observations_complete = true;
  const id = Object.keys(f.measurement.measurements.head[0])[0];
  const original = f.measurement.measurements.head[0][id];
  delete f.measurement.measurements.head[0][id];
  f.save();
  refuse();
  f.measurement.measurements.head[0][id] = original;
  f.measurement.measurements.head[0][id] = {
    ...original,
    file: path.join(f.arms.base.root, "tools/benchmarks/results/davinci", `${id}.json`),
  };
  f.save();
  refuse();
  f.measurement.measurements.head[0][id] = original;
  f.save();
  f.writeReport("head", 1, id, { harness_version: "9.9.9" });
  f.save();
  refuse();
  f.writeReport("head", 1, id, { harness_version: "0.442.0" });
  f.save();
  f.carrier.current = { arm: "base", repetition: 1 };
  f.write(f.pair, f.carrier);
  refuse();
  f.carrier.current = { arm: "head", repetition: 1 };
  f.write(f.pair, f.carrier);
  const formatId = [...f.authority.planes.formatter.registry][0];
  const dest = path.join(f.out, "head/run-1/level-reports", `${formatId}.json`);
  fs.copyFileSync(path.join(f.out, "head/run-1/formatter-reports", `${formatId}.json`), dest);
  refuse();
  fs.unlinkSync(dest);
  assert.equal(sourceCompare(f, ["--bench", id]).status, 2);
  assert.equal(sourceCompare(f, ["--update-baseline"]).status, 2);
});

await test("source-owned comparison refuses legitimate source protocol changes and raw process drift", (t) => {
  const f = sourceJudgeFixture(t);
  const refuse = () => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
  };
  for (const relative of [
    "tools/benchmarks/crates/davinci_harness/src/report.rs",
    "tools/benchmarks/crates/davinci_harness/benches/selfcheck.rs",
  ]) {
    const file = path.join(f.arms.head.root, relative),
      original = fs.readFileSync(file);
    fs.appendFileSync(file, "\n// changed protocol source\n");
    f.refresh("head");
    f.save();
    refuse();
    fs.writeFileSync(file, original);
    f.refresh("head");
    f.save();
  }
  const extra = path.join(f.arms.head.root, "tools/benchmarks/crates/davinci_harness/src/extra.rs");
  fs.writeFileSync(extra, "// additional protocol path\n");
  f.refresh("head");
  f.save();
  refuse();
  fs.unlinkSync(extra);
  f.refresh("head");
  f.save();
  const file = path.join(f.arms.head.build, "cargo-metadata/process.json"),
    original = fs.readFileSync(file),
    receipt = JSON.parse(original.toString());
  receipt.status = 7;
  f.write(file, receipt);
  refuse();
  fs.writeFileSync(file, original);
  receipt.status = 0;
  receipt.imageAfter.sha256 = "0".repeat(64);
  f.write(file, receipt);
  refuse();
  fs.writeFileSync(file, original);
  assert.equal(sourceCompare(f).status, 0);
});

await test("source-owned comparison binds inherited profiles, executable images and exact dev requirements", (t) => {
  const f = sourceJudgeFixture(t);
  const refuse = () => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
  };
  const workspace = path.join(f.arms.head.root, "Cargo.toml"),
    originalWorkspace = fs.readFileSync(workspace);
  for (const replacement of [
    ["opt-level = 3", "opt-level = 2"],
    ['panic = "unwind"', 'panic = "abort"'],
    ["overflow-checks = false", "overflow-checks = true"],
  ]) {
    fs.writeFileSync(
      workspace,
      originalWorkspace.toString().replace(replacement[0], replacement[1]),
    );
    f.refresh("head");
    f.save();
    refuse();
  }
  for (const name of ["vize_l0", "vize_l1", "vize_l1_to_l2", "vize_l2"]) {
    const line = new RegExp(`^${name} = .*?$`, "m").exec(originalWorkspace.toString())![0];
    for (const requirement of ['"0.442.0"', '"=9.9.9"']) {
      fs.writeFileSync(
        workspace,
        originalWorkspace.toString().replace(line, line.replace('"=0.442.0"', requirement)),
      );
      f.refresh("head");
      f.save();
      refuse();
    }
  }
  fs.writeFileSync(workspace, originalWorkspace);
  f.refresh("head");
  f.save();
  const foreignImage = path.join(f.out, "foreign-image");
  for (const relative of ["cargo-metadata", "cargo", "rustc"]) {
    const receiptFile = path.join(f.arms.head.build, relative, "process.json"),
      original = fs.readFileSync(receiptFile),
      receipt = JSON.parse(original.toString());
    fs.copyFileSync(receipt.imageBefore.resolved, foreignImage);
    for (const changes of [
      { path: foreignImage },
      { path: foreignImage, resolved: foreignImage },
    ]) {
      const image = { ...receipt.imageBefore, ...changes };
      f.write(receiptFile, { ...receipt, imageBefore: image, imageAfter: image });
      refuse();
    }
    fs.writeFileSync(receiptFile, original);
  }
  const allToolReceipts = ["base", "head"].flatMap((arm) =>
    ["cargo-metadata", "cargo", "rustc"].map((relative) => {
      const file = path.join(f.arms[arm].build, relative, "process.json");
      return { file, body: fs.readFileSync(file) };
    }),
  );
  for (const { file, body } of allToolReceipts) {
    const receipt = JSON.parse(body.toString()),
      image = { ...receipt.imageBefore, path: foreignImage, resolved: foreignImage };
    f.write(file, { ...receipt, imageBefore: image, imageAfter: image });
  }
  refuse();
  for (const { file, body } of allToolReceipts) fs.writeFileSync(file, body);
  assert.equal(sourceCompare(f).status, 0);
});

await test("source-owned comparison binds actual inheritance, metadata and library artifacts", (t) => {
  const f = sourceJudgeFixture(t),
    metadataDir = path.join(f.arms.head.build, "cargo-metadata"),
    cargoDir = path.join(f.arms.head.build, "cargo");
  const refuse = () => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
  };
  const original = JSON.stringify(f.arms.head.metadata);
  const variants = [
    { workspace_root: f.arms.base.root, packages: f.arms.head.metadata.packages },
    { workspace_root: f.arms.head.root, packages: [] },
    {
      workspace_root: f.arms.head.root,
      packages: [...f.arms.head.metadata.packages, ...f.arms.head.metadata.packages],
    },
    ...["version", "id", "manifest_path"].map((field) => ({
      workspace_root: f.arms.head.root,
      packages: [
        {
          ...f.arms.head.metadata.packages[0],
          [field]: field === "version" ? "9.9.9" : path.join(f.arms.base.root, "foreign"),
        },
      ],
    })),
  ];
  for (const variant of variants) {
    f.rewriteStream(metadataDir, "stdout", JSON.stringify(variant));
    refuse();
  }
  f.rewriteStream(metadataDir, "stdout", original);
  const artifacts = fs.readFileSync(path.join(cargoDir, "stdout.bin"), "utf8");
  const messages = artifacts
    .trim()
    .split("\n")
    .map((line) => JSON.parse(line));
  for (const variant of [
    messages.slice(1),
    [messages[0], ...messages],
    [{ ...messages[0], package_id: "foreign" }, ...messages.slice(1)],
    [
      { ...messages[0], target: { ...messages[0].target, src_path: f.arms.base.library } },
      ...messages.slice(1),
    ],
  ]) {
    f.rewriteStream(cargoDir, "stdout", variant.map((x) => JSON.stringify(x)).join("\n") + "\n");
    refuse();
  }
  f.rewriteStream(cargoDir, "stdout", artifacts);
  const workspace = path.join(f.arms.head.root, "Cargo.toml"),
    body = fs.readFileSync(workspace);
  fs.writeFileSync(workspace, body.toString().replaceAll('"0.442.0"', '"0.443.0"'));
  f.refresh("head");
  f.save();
  refuse();
  fs.writeFileSync(workspace, body);
  f.refresh("head");
  f.save();
  const manifest = f.arms.head.manifest,
    manifestBody = fs.readFileSync(manifest);
  fs.writeFileSync(
    manifest,
    manifestBody.toString().replace("version.workspace = true", 'version = "0.442.0"'),
  );
  f.refresh("head");
  f.save();
  refuse();
  fs.writeFileSync(manifest, manifestBody);
  f.refresh("head");
  f.save();
  assert.equal(sourceCompare(f).status, 0);
});

await test("source-owned comparison refuses source pins, source roots and completed-process substitutions", (t) => {
  const f = sourceJudgeFixture(t);
  const refuse = () => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
  };
  const witness = f.measurement.source_harness.head;
  for (const field of [
    "root",
    "manifest_path",
    "library_source",
    "package_id",
    "version",
    "workspace_manifest_sha256",
  ]) {
    f.measurement.source_harness.head = {
      ...witness,
      [field]: field === "root" ? f.arms.base.root : "foreign",
    };
    f.save();
    refuse();
  }
  f.measurement.source_harness.head = witness;
  f.save();
  const before = path.join(f.arms.head.build, "harness-source-before.json"),
    pin = fs.readFileSync(before);
  for (const field of ["git_blob", "sha256", "path"]) {
    const changed = JSON.parse(pin.toString());
    changed.files[0][field] = "foreign";
    f.write(before, changed);
    refuse();
  }
  fs.writeFileSync(before, pin);
  const source = f.arms.head.library,
    body = fs.readFileSync(source);
  fs.appendFileSync(source, "\n// unpinned source\n");
  refuse();
  fs.writeFileSync(source, body);
  const head = path.join(f.arms.head.provenance, "head");
  f.rewriteStream(head, "stdout", "3".repeat(40) + "\n");
  refuse();
  f.refresh("head");
  f.save();
  for (const relative of ["cargo-metadata", "cargo", "rustc", "resolve-rustc", "resolve-cargo"]) {
    const file = path.join(f.arms.head.build, relative, "process.json"),
      original = fs.readFileSync(file);
    for (const change of [
      { signal: "SIGSEGV", status: null },
      { pid: null },
      { cwd: f.arms.base.root },
      { args: [] },
    ]) {
      f.write(file, { ...JSON.parse(original.toString()), ...change });
      refuse();
    }
    fs.writeFileSync(file, original);
  }
  const stderr = path.join(f.arms.head.build, "cargo-metadata/stderr.bin"),
    raw = fs.readFileSync(stderr);
  fs.unlinkSync(stderr);
  refuse();
  fs.writeFileSync(stderr, raw);
  const normal = path.join(f.out, "head/run-1/suite-0/process/process.json"),
    normalBody = fs.readFileSync(normal);
  for (const change of [
    { status: 11 },
    { signal: "SIGSEGV", status: null },
    { args: ["--quick"] },
    { recipeEnvironment: {} },
  ]) {
    f.write(normal, { ...JSON.parse(normalBody.toString()), ...change });
    refuse();
  }
  fs.writeFileSync(normal, normalBody);
  assert.equal(sourceCompare(f).status, 0);
});

await test("source-owned comparison retains complete protocol, dependencies and raw report custody", (t) => {
  const f = sourceJudgeFixture(t);
  const refuse = () => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
  };
  for (const relative of [
    "tools/benchmarks/crates/davinci_harness/src/alloc.rs",
    "tools/benchmarks/crates/davinci_harness/src/report.rs",
    "tests/davinci_test_support/src/lib.rs",
    path.relative(f.arms.head.root, f.arms.head.binaries[1].source),
  ]) {
    const file = path.join(f.arms.head.root, relative),
      original = fs.readFileSync(file);
    fs.appendFileSync(file, "\n// altered protocol body\n");
    f.refresh("head");
    f.save();
    refuse();
    fs.writeFileSync(file, original);
    f.refresh("head");
    f.save();
  }
  const removed = path.join(
      f.arms.head.root,
      "tools/benchmarks/crates/davinci_harness/src/report.rs",
    ),
    removedBody = fs.readFileSync(removed);
  fs.unlinkSync(removed);
  f.refresh("head");
  f.save();
  refuse();
  fs.writeFileSync(removed, removedBody);
  f.refresh("head");
  f.save();
  const workspace = path.join(f.arms.head.root, "Cargo.toml"),
    workspaceBody = fs.readFileSync(workspace);
  for (const replacement of [
    'vize_l0 = { path = "davinci/vize_l0", version = "9.9.9" }',
    'vize_l0 = { path = "davinci/vize_l0", version = "=0.442.0", features = ["foreign"] }',
  ]) {
    fs.writeFileSync(workspace, workspaceBody.toString().replace(/^vize_l0 = .*$/m, replacement));
    f.refresh("head");
    f.save();
    refuse();
  }
  fs.writeFileSync(workspace, workspaceBody);
  f.refresh("head");
  f.save();
  const lock = path.join(f.arms.head.root, "Cargo.lock"),
    lockBody = fs.readFileSync(lock);
  fs.writeFileSync(
    lock,
    lockBody.toString().replace(/checksum = "[a-f0-9]+"/, 'checksum = "' + "0".repeat(64) + '"'),
  );
  f.refresh("head");
  f.save();
  refuse();
  fs.writeFileSync(lock, lockBody);
  f.refresh("head");
  f.save();
  const id = Object.keys(f.measurement.measurements.head[0])[0],
    row = f.measurement.measurements.head[0][id];
  const rawFile = row.raw_file,
    raw = fs.readFileSync(rawFile);
  fs.writeFileSync(rawFile, Buffer.concat([raw, Buffer.from("\n")]));
  refuse();
  fs.writeFileSync(rawFile, raw);
  f.measurement.measurements.head[0].foreign = { ...row };
  f.save();
  refuse();
  delete f.measurement.measurements.head[0].foreign;
  f.save();
  f.writeReport("head", 1, id, { platform: "macos" });
  f.save();
  refuse();
  f.writeReport("head", 1, id, { platform: "linux" });
  f.save();
  f.measurement.measurements.head[0][id] = {
    ...row,
    identity: { ...row.identity, window: "foreign" },
  };
  f.save();
  refuse();
  f.measurement.measurements.head[0][id] = row;
  f.save();
  assert.equal(sourceCompare(f).status, 0);
});

await test("source-owned version admission leaves platform peak equality and conditional wall behavior intact", (t) => {
  const f = sourceJudgeFixture(t),
    id = Object.keys(f.budgets.allocation_peak)[0],
    peak = f.budgets.allocation_peak[id].linux;
  for (const delta of [1, -1]) {
    f.writeReport("head", 1, id, { alloc_bytes_peak: peak + delta });
    f.save();
    const result = sourceCompare(f);
    assert.equal(result.status, 1, result.stderr);
    assert.match(result.stdout, /exact platform-aware peak gate/);
  }
  f.writeReport("head", 1, id, { alloc_bytes_peak: peak });
  f.save();
  // The original100 source registry currently has no recorded wall baselines.
  // A large wall observation remains report-only under that unchanged authority;
  // the original default comparator fixtures independently exercise wall failures.
  assert.ok(Object.values(f.budgets.bench).every((budget: any) => budget.wall_p50_ns === 0));
  f.writeReport("head", 1, id, { wall_ns: { p50: 1_000_000_000, p95: 1_000_000_000 } });
  f.save();
  const result = sourceCompare(f);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /wall baseline not yet recorded/);
});

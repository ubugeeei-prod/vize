import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
const { sourceJudgeFixture } = await import("./support/bench-source-harness-fixture.mjs");
import { sourceCompare } from "./support/bench-source-compare.ts";

await test("source-owned comparison binds actual inheritance, metadata and library artifacts", (t) => {
  const f = sourceJudgeFixture(t),
    metadataDir = path.join(f.arms.head.build, "cargo-metadata"),
    cargoDir = path.join(f.arms.head.build, "cargo");
  const refuse = (reason: RegExp) => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
    assert.match(result.stderr, reason);
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
  for (const [index, variant] of variants.entries()) {
    f.rewriteStream(metadataDir, "stdout", JSON.stringify(variant));
    refuse(
      index === 0
        ? /foreign Cargo metadata workspace/
        : index < 3
          ? /missing or duplicate harness metadata/
          : /metadata differs from actual source inheritance/,
    );
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
    refuse(/missing or foreign compiled harness library/);
  }
  f.rewriteStream(cargoDir, "stdout", artifacts);
  const workspace = path.join(f.arms.head.root, "Cargo.toml"),
    body = fs.readFileSync(workspace);
  fs.writeFileSync(workspace, body.toString().replaceAll('"0.442.0"', '"0.443.0"'));
  f.refresh("head");
  f.save();
  refuse(/metadata differs from actual source inheritance/);
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
  refuse(/missing TOML workspace/);
  fs.writeFileSync(manifest, manifestBody);
  f.refresh("head");
  f.save();
  assert.equal(sourceCompare(f).status, 0);
});

await test("source-owned comparison refuses source pins, source roots and completed-process substitutions", (t) => {
  const f = sourceJudgeFixture(t);
  const refuse = (reason: RegExp) => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
    assert.match(result.stderr, reason);
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
    refuse(
      field === "root"
        ? /foreign or changed provider image/
        : field === "workspace_manifest_sha256"
          ? /source witness digest differs/
          : /source witness differs from metadata\/artifact/,
    );
  }
  f.measurement.source_harness.head = witness;
  f.save();
  const before = path.join(f.arms.head.build, "harness-source-before.json"),
    pin = fs.readFileSync(before);
  for (const field of ["git_blob", "sha256", "path"]) {
    const changed = JSON.parse(pin.toString());
    changed.files[0][field] = "foreign";
    f.write(before, changed);
    refuse(/missing or changed pre-build source pin/);
  }
  fs.writeFileSync(before, pin);
  const source = f.arms.head.library,
    body = fs.readFileSync(source);
  fs.appendFileSync(source, "\n// unpinned source\n");
  refuse(/source differs from exact Git blob/);
  fs.writeFileSync(source, body);
  const head = path.join(f.arms.head.provenance, "head");
  f.rewriteStream(head, "stdout", "3".repeat(40) + "\n");
  refuse(/foreign source revision/);
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
      refuse(
        "signal" in change
          ? /incomplete or failed process/
          : "pid" in change
            ? /missing process PID/
            : relative === "cargo-metadata"
              ? "cwd" in change
                ? /foreign source Cargo process/
                : /changed Cargo metadata recipe/
              : relative === "cargo"
                ? "cwd" in change
                  ? /foreign source Cargo process/
                  : /changed provider build recipe/
                : /foreign process recipe or source root/,
      );
    }
    fs.writeFileSync(file, original);
  }
  const stderr = path.join(f.arms.head.build, "cargo-metadata/stderr.bin"),
    raw = fs.readFileSync(stderr);
  fs.unlinkSync(stderr);
  refuse(/cargo-metadata\/stderr\.bin/);
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
    refuse(
      "status" in change
        ? /incomplete or failed process/
        : "args" in change
          ? /foreign process recipe or source root/
          : /missing or extra fields/,
    );
  }
  fs.writeFileSync(normal, normalBody);
  assert.equal(sourceCompare(f).status, 0);
});

await test("source-owned comparison retains complete protocol, dependencies and raw report custody", (t) => {
  const f = sourceJudgeFixture(t);
  const refuse = (reason: RegExp) => {
    const result = sourceCompare(f);
    assert.equal(result.status, 2, result.stderr);
    assert.match(result.stderr, /source harness/);
    assert.match(result.stderr, reason);
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
    refuse(/changed reporter\/scenario\/provider\/protocol\/dependency/);
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
  refuse(/changed reporter\/scenario\/provider\/protocol\/dependency/);
  fs.writeFileSync(removed, removedBody);
  f.refresh("head");
  f.save();
  const workspace = path.join(f.arms.head.root, "Cargo.toml"),
    workspaceBody = fs.readFileSync(workspace);
  for (const [index, replacement] of [
    'vize_l0 = { path = "davinci/vize_l0", version = "9.9.9" }',
    'vize_l0 = { path = "davinci/vize_l0", version = "=0.442.0", features = ["foreign"] }',
  ].entries()) {
    fs.writeFileSync(workspace, workspaceBody.toString().replace(/^vize_l0 = .*$/m, replacement));
    f.refresh("head");
    f.save();
    refuse(
      index === 0 ? /foreign dev-product release version/ : /changed protocol workspace dependency/,
    );
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
  refuse(/changed reporter\/scenario\/provider\/protocol\/dependency/);
  fs.writeFileSync(lock, lockBody);
  f.refresh("head");
  f.save();
  const id = Object.keys(f.measurement.measurements.head[0])[0],
    row = f.measurement.measurements.head[0][id];
  const rawFile = row.raw_file,
    raw = fs.readFileSync(rawFile);
  fs.writeFileSync(rawFile, Buffer.concat([raw, Buffer.from("\n")]));
  refuse(/changed raw report custody/);
  fs.writeFileSync(rawFile, raw);
  f.measurement.measurements.head[0].foreign = { ...row };
  f.save();
  refuse(/missing extra or wrong-plane reports/);
  delete f.measurement.measurements.head[0].foreign;
  f.save();
  f.writeReport("head", 1, id, { platform: "macos" });
  f.save();
  refuse(/report differs from own source version\/platform\/metrics/);
  f.writeReport("head", 1, id, { platform: "linux" });
  f.save();
  f.measurement.measurements.head[0][id] = {
    ...row,
    identity: { ...row.identity, window: "foreign" },
  };
  f.save();
  refuse(/changed original input\/window identity/);
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
  // The complete101 source registry currently has no recorded wall baselines.
  // A large wall observation remains report-only under that unchanged authority;
  // the original default comparator fixtures independently exercise wall failures.
  assert.ok(Object.values(f.budgets.bench).every((budget: any) => budget.wall_p50_ns === 0));
  f.writeReport("head", 1, id, { wall_ns: { p50: 1_000_000_000, p95: 1_000_000_000 } });
  f.save();
  const result = sourceCompare(f);
  assert.equal(result.status, 0, result.stderr);
  assert.match(result.stdout, /wall baseline not yet recorded/);
});

await test("paired memory builds select the single contributor toolchain pin", (t) => {
  const f = sourceJudgeFixture(t);
  // A retired measurement manifest in an older arm no longer selects the compiler.
  fs.writeFileSync(
    path.join(f.arms.base.root, "rust-measurement-toolchain.toml"),
    '[toolchain]\nchannel = "1.99.0"\n',
  );
  f.refresh("base");
  f.save();
  const retired = sourceCompare(f);
  assert.equal(retired.status, 0, retired.stderr);
  const head = path.join(f.arms.head.root, "rust-toolchain.toml");
  fs.writeFileSync(head, fs.readFileSync(head, "utf8").replace("1.99.1", "1.99.0"));
  f.refresh("head");
  f.save();
  const changed = sourceCompare(f);
  assert.equal(changed.status, 2, changed.stderr);
  assert.match(changed.stderr, /foreign source toolchain pin/);
});

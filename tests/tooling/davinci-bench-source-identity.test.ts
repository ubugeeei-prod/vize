import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
const { sourceJudgeFixture } = await import("./support/bench-source-harness-fixture.mjs");
import { spawnSync } from "node:child_process";
import { repoRoot, comparePath, sourceCompare } from "./support/bench-source-compare.ts";

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

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import {
  collectFreshReports,
  validatePair,
} from "../../tools/benchmarks/scripts/level-memory-lib.mjs";
import { collectNormal } from "../../tools/benchmarks/scripts/level-memory-runner.mjs";
import * as memory from "../../tools/benchmarks/scripts/level-memory-lib.mjs";
import {
  fixture,
  sourceHarnessFixture,
  originalPopulation,
} from "./support/level-memory-fixtures.mjs";

await test("different released harness versions require both exact source/build witnesses", (t) => {
  const f = fixture(t);
  const base = collectFreshReports(f.root, f.markers, f.identities);
  const head = structuredClone(base);
  for (const row of head.values()) row.report.harness_version = "1.1.0";
  // The original two-argument boundary still refuses changed raw versions.
  assert.throws(() => validatePair(base, head), /harness_version/);
  const oldSource = sourceHarnessFixture(t, "1.0.0", f.root);
  const newSource = sourceHarnessFixture(t, "1.1.0");
  const binding = (source) =>
    memory.bindHarnessSource(
      source.root,
      source.metadata,
      source.artifacts,
      memory.captureHarnessSource(source.root, source.wholeTree),
    );
  const versions = { base: binding(oldSource), head: binding(newSource) };
  for (const [id, row] of head)
    row.file = path.join(newSource.root, memory.REPORT_DIRECTORY, `${id}.json`);
  validatePair(base, head, versions);
  const comparisons = memory.compareMemoryMetrics(base, head, versions);
  assert.equal(base.get("one").report.harness_version, "1.0.0");
  assert.equal(head.get("one").report.harness_version, "1.1.0");
  assert.deepEqual(comparisons.one.allocs, { base: 0, head: 0, delta: 0 });
  assert.throws(
    () => validatePair(base, head, { base: versions.head, head: versions.base }),
    /source arm|harness_version/,
  );
  assert.throws(
    () => validatePair(base, head, { base: { ...versions.base }, head: versions.head }),
    /source.*witness/,
  );
  assert.throws(() => validatePair(base, head, { base: versions.base }), /fields|witness/);
  for (const side of ["base", "head"]) {
    const forged = structuredClone(side === "base" ? base : head);
    forged.get("one").report.harness_version = "9.9.9";
    assert.throws(
      () =>
        side === "base"
          ? validatePair(forged, head, versions)
          : validatePair(base, forged, versions),
      /harness_version/,
    );
  }
});

await test("equal versions cannot substitute a different source arm", (t) => {
  const f = fixture(t);
  const rows = collectFreshReports(f.root, f.markers, f.identities);
  const own = sourceHarnessFixture(t, "1.0.0", f.root);
  const foreign = sourceHarnessFixture(t, "1.0.0");
  const bind = (source) =>
    memory.bindHarnessSource(
      source.root,
      source.metadata,
      source.artifacts,
      memory.captureHarnessSource(source.root, source.wholeTree),
    );
  assert.throws(
    () => validatePair(rows, rows, { base: bind(own), head: bind(foreign) }),
    /report.*source|source.*report/,
  );
});

await test("foreign metadata and a missing or mismatched harness library never mint a witness", (t) => {
  const f = sourceHarnessFixture(t, "1.0.0");
  const bind = (metadata = f.metadata, artifacts = f.artifacts) =>
    memory.bindHarnessSource(
      f.root,
      metadata,
      artifacts,
      memory.captureHarnessSource(f.root, f.wholeTree),
    );
  for (const change of [
    { workspace_root: path.dirname(f.root) },
    { packages: [] },
    { packages: [...f.metadata.packages, ...f.metadata.packages] },
    { packages: [{ ...f.metadata.packages[0], name: "other_harness" }] },
    { packages: [{ ...f.metadata.packages[0], manifest_path: path.join(f.root, "foreign.toml") }] },
  ])
    assert.throws(() => bind({ ...f.metadata, ...change }), /workspace|harness|manifest/);
  assert.throws(() => bind(f.metadata, []), /library/);
  assert.throws(() => bind(f.metadata, [...f.artifacts, ...f.artifacts]), /library/);
  assert.throws(
    () => bind(f.metadata, [{ ...f.artifacts[0], package_id: "foreign-package" }]),
    /package|library/,
  );
  assert.throws(
    () =>
      bind(f.metadata, [
        {
          ...f.artifacts[0],
          target: { ...f.artifacts[0].target, src_path: path.join(f.root, "foreign.rs") },
        },
      ]),
    /source|library/,
  );
});

await test("harness version authority requires pre-build Git blobs and unchanged source bodies", (t) => {
  const f = sourceHarnessFixture(t, "1.0.0");
  const pin = memory.captureHarnessSource(f.root, f.wholeTree);
  const bind = (sourcePin = pin, root = f.root) =>
    memory.bindHarnessSource(root, f.metadata, f.artifacts, sourcePin);
  assert.equal(bind().version, "1.0.0");
  for (const listing of [
    f.wholeTree.replace(/ [a-f0-9]{40}\t/, ` ${"0".repeat(40)}\t`),
    f.wholeTree.replace("100644", "120000"),
    f.wholeTree.replace("\tCargo.toml", "\tforeign\tCargo.toml"),
    f.wholeTree.split("\n").slice(1).join("\n"),
    `${f.wholeTree}${f.wholeTree.split("\n")[0]}\n`,
  ])
    assert.throws(() => memory.captureHarnessSource(f.root, listing), /source|Git|blob|mode/);
  assert.throws(() => bind({ ...pin }), /source.*pin/);
  assert.throws(() => memory.bindHarnessSource(f.root, f.metadata, f.artifacts), /source.*pin/);
  const foreign = sourceHarnessFixture(t, "1.0.0");
  const foreignPin = memory.captureHarnessSource(foreign.root, foreign.wholeTree);
  assert.throws(() => bind(foreignPin), /source.*root/);
  const witness = bind();
  fs.appendFileSync(path.join(f.root, "Cargo.toml"), "# changed after pre-build capture\n");
  assert.throws(() => bind(), /source.*changed|Git.*blob/);
  assert.throws(
    () => memory.validateHarnessReports(new Map(), witness),
    /source.*changed|Git.*blob/,
  );
});

await test("a refused own-source version retains all completed raw reports before admission", (t) => {
  const f = fixture(t);
  const own = sourceHarnessFixture(t, "1.1.0", f.root);
  const witness = memory.bindHarnessSource(
    own.root,
    own.metadata,
    own.artifacts,
    memory.captureHarnessSource(own.root, own.wholeTree),
  );
  const executable = path.join(f.root, "synthetic-normal-probe");
  const provider = path.join(f.root, "synthetic-provider.rs");
  fs.writeFileSync(provider, "// collector source-law provider only\n");
  const stderr = `option 'reserve_os_memory': 131072 KiB\noption 'purge_delay': -1 \n${f.markers}\n`;
  fs.writeFileSync(
    executable,
    `#!${process.execPath}\nprocess.stderr.write(${JSON.stringify(stderr)});\n`,
  );
  fs.chmodSync(executable, 0o755);
  const suite = {
    executable,
    source: provider,
    sha256: memory.sha256(fs.readFileSync(executable)),
    source_sha256: memory.sha256(fs.readFileSync(provider)),
    harness_source: witness,
  };
  const out = path.join(f.root, "refused-version");
  assert.throws(
    () => collectNormal(f.root, suite, executable, out, f.identities),
    /harness_version/,
  );
  const retained = JSON.parse(fs.readFileSync(path.join(out, "rows.json")));
  assert.deepEqual(Object.keys(retained), [...f.identities.keys()]);
  for (const [id, row] of Object.entries(retained)) {
    assert.equal(row.report.harness_version, "1.0.0");
    assert.deepEqual(
      fs.readFileSync(row.raw_file),
      fs.readFileSync(path.join(f.dir, `${id}.json`)),
    );
  }
  const processReceipt = JSON.parse(fs.readFileSync(path.join(out, "process/process.json")));
  assert.equal(processReceipt.status, 0);
  assert.equal(processReceipt.signal, null);
});

await test("source-owned judge handoff waits for all630 and refuses unissued witnesses", (t) => {
  const f = fixture(t);
  const own = sourceHarnessFixture(t, "1.0.0", f.root);
  const head = sourceHarnessFixture(t, "1.1.0");
  const bind = (source) =>
    memory.bindHarnessSource(
      source.root,
      source.metadata,
      source.artifacts,
      memory.captureHarnessSource(source.root, source.wholeTree),
    );
  const witnesses = { base: bind(own), head: bind(head) };
  const { rows } = originalPopulation();
  const measurements = {};
  for (const side of ["base", "head"])
    measurements[side] = Array.from({ length: 3 }, () =>
      Object.fromEntries(
        [...rows].map(([id, row]) => [
          id,
          {
            ...structuredClone(row),
            file: path.join(witnesses[side].root, memory.REPORT_DIRECTORY, `${id}.json`),
            report: { ...row.report, harness_version: witnesses[side].version },
          },
        ]),
      ),
    );
  const out = path.join(f.root, "judge-handoff");
  fs.mkdirSync(out);
  const manifest = { observations_complete: true, measurements };
  fs.writeFileSync(path.join(out, "measurement.json"), JSON.stringify(manifest));
  const file = memory.writeHarnessJudgePair(out, manifest, witnesses, "head", 2);
  assert.deepEqual(JSON.parse(fs.readFileSync(file)), {
    schema_version: 1,
    measurement: path.join(out, "measurement.json"),
    baseline: { arm: "base", repetition: 2 },
    current: { arm: "head", repetition: 2 },
  });
  for (const arm of ["base", "head"])
    for (const repetition of [1, 2, 3]) {
      const pairFile =
        arm === "head" && repetition === 2
          ? file
          : memory.writeHarnessJudgePair(out, manifest, witnesses, arm, repetition);
      const pair = JSON.parse(fs.readFileSync(pairFile));
      assert.deepEqual(pair.baseline, { arm: "base", repetition });
      assert.deepEqual(pair.current, { arm, repetition });
    }
  assert.throws(
    () =>
      memory.writeHarnessJudgePair(
        out,
        { ...manifest, observations_complete: false },
        witnesses,
        "head",
        1,
      ),
    /complete/,
  );
  const incomplete = structuredClone(manifest);
  delete incomplete.measurements.head[0][Object.keys(incomplete.measurements.head[0])[0]];
  assert.throws(
    () => memory.writeHarnessJudgePair(out, incomplete, witnesses, "head", 1),
    /105|complete/,
  );
  assert.throws(
    () =>
      memory.writeHarnessJudgePair(
        out,
        manifest,
        { base: { ...witnesses.base }, head: witnesses.head },
        "head",
        1,
      ),
    /witness/,
  );
  assert.throws(() => memory.writeHarnessJudgePair(out, manifest, witnesses, "foreign", 1), /arm/);
  assert.throws(
    () => memory.writeHarnessJudgePair(out, manifest, witnesses, "head", 0),
    /repetition/,
  );
  assert.throws(
    () => memory.writeHarnessJudgePair(out, manifest, witnesses, "head", 4),
    /repetition/,
  );
});

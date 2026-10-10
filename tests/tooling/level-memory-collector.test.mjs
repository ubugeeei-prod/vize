import assert from "node:assert/strict";
import fs from "node:fs";
import { createHash } from "node:crypto";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { loadRegistry } from "../../tools/benchmarks/scripts/instruction-counts-lib.mjs";
import {
  levelInstructionSuites,
  formatterInstructionSuites,
} from "../../tools/benchmarks/scripts/instruction-counts-suites.mjs";
import {
  collectFreshReports,
  validatePair,
  MEMORY_ENVIRONMENT,
} from "../../tools/benchmarks/scripts/level-memory-lib.mjs";
import { collectNormal, run } from "../../tools/benchmarks/scripts/level-memory-runner.mjs";
import * as memory from "../../tools/benchmarks/scripts/level-memory-lib.mjs";

const report = (id) => ({
  bench_id: id,
  fixture: `synthetic:${id}`,
  platform: "linux",
  wall_ns: { p50: 1, p95: 2 },
  allocs: 0,
  alloc_bytes_peak: 0,
  rss_peak_bytes: 0,
  harness_version: "1.0.0",
});
function fixture(t) {
  const root = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "level-memory-law-")));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const dir = path.join(root, "tools/benchmarks/results/davinci");
  fs.mkdirSync(dir, { recursive: true });
  const identities = new Map(
    ["one", "two"].map((id) => [
      id,
      {
        bench_id: id,
        fixture: `synthetic:${id}`,
        fixture_sha256: "a".repeat(64),
        window: "routine-return",
      },
    ]),
  );
  for (const id of identities.keys())
    fs.writeFileSync(path.join(dir, `${id}.json`), JSON.stringify(report(id)));
  const markers = [...identities.keys()]
    .map((id) => `davinci-harness: wrote ${path.join(dir, `${id}.json`)}`)
    .join("\n");
  return { root, dir, identities, markers };
}

await test("fresh complete reports retain whole packets and original identities", (t) => {
  const f = fixture(t);
  const rows = collectFreshReports(f.root, f.markers, f.identities);
  assert.equal(rows.size, 2);
  assert.deepEqual(rows.get("one").report, report("one"));
  assert.equal(rows.get("one").identity.window, "routine-return");
  assert.match(rows.get("one").raw_sha256, /^[a-f0-9]{64}$/);
  validatePair(rows, rows);
  assert.ok(!Object.hasOwn(MEMORY_ENVIRONMENT, "VIZE_INSTRUCTION_COUNTS"));
});

await test("tracked or cached reports without actual emitted writes are rejected", (t) => {
  const f = fixture(t);
  assert.throws(() => collectFreshReports(f.root, "", f.identities), /emitted|missing/);
  assert.throws(
    () => collectFreshReports(f.root, f.markers.split("\n")[0], f.identities),
    /missing/,
  );
});

await test("duplicate, extra and foreign emitted reports cannot qualify", (t) => {
  const f = fixture(t);
  assert.throws(
    () => collectFreshReports(f.root, `${f.markers}\n${f.markers}`, f.identities),
    /duplicate/,
  );
  fs.writeFileSync(path.join(f.dir, "three.json"), JSON.stringify(report("three")));
  assert.throws(
    () =>
      collectFreshReports(
        f.root,
        `${f.markers}\ndavinci-harness: wrote ${path.join(f.dir, "three.json")}`,
        f.identities,
      ),
    /unregistered|extra/,
  );
  assert.throws(
    () =>
      collectFreshReports(
        f.root,
        `${f.markers}\ndavinci-harness: wrote /tmp/foreign.json`,
        f.identities,
      ),
    /outside|foreign/,
  );
});

await test("null metrics, forged fields and wrong fixture identities remain red", (t) => {
  const f = fixture(t);
  for (const change of [
    { allocs: null },
    { alloc_bytes_peak: null },
    { rss_peak_bytes: null },
    { fixture: "synthetic:other" },
    { bench_id: "two" },
    { platform: "darwin" },
    { accepted: true },
    { wall_ns: { p50: 3, p95: 2 } },
  ]) {
    fs.writeFileSync(path.join(f.dir, "one.json"), JSON.stringify({ ...report("one"), ...change }));
    assert.throws(() => collectFreshReports(f.root, f.markers, f.identities));
  }
});

await test("symlink output and changed same-byte source/window populations are rejected", (t) => {
  const f = fixture(t);
  const rows = collectFreshReports(f.root, f.markers, f.identities);
  const changed = structuredClone(rows);
  changed.get("one").identity.window = "stage-return";
  assert.throws(() => validatePair(rows, changed), /identity|window/);
  changed.get("one").identity.window = "routine-return";
  changed.get("one").identity.fixture_sha256 = "b".repeat(64);
  assert.throws(() => validatePair(rows, changed), /identity|fixture/);
  changed.delete("two");
  assert.throws(() => validatePair(rows, changed), /missing/);
  fs.unlinkSync(path.join(f.dir, "one.json"));
  fs.symlinkSync(path.join(f.dir, "two.json"), path.join(f.dir, "one.json"));
  assert.throws(() => collectFreshReports(f.root, f.markers, f.identities), /symlink/);
});

await test("a failing process preserves complete raw streams before refusing admission", (t) => {
  const f = fixture(t);
  const directory = path.join(f.root, "failed-process");
  assert.throws(
    () =>
      run(
        process.execPath,
        [
          "-e",
          "process.stdout.write(Buffer.from([0,255,10]));process.stderr.write('whole stderr\\n');process.exit(7)",
        ],
        f.root,
        directory,
        { ...process.env, MEMORY_TEST_SECRET: "must-not-escape" },
      ),
    /exit 7/,
  );
  assert.deepEqual(fs.readFileSync(path.join(directory, "stdout.bin")), Buffer.from([0, 255, 10]));
  assert.equal(fs.readFileSync(path.join(directory, "stderr.bin"), "utf8"), "whole stderr\n");
  const receipt = JSON.parse(fs.readFileSync(path.join(directory, "process.json")));
  assert.equal(receipt.status, 7);
  assert.equal(receipt.signal, null);
  assert.deepEqual(receipt.imageBefore, receipt.imageAfter);
  assert.ok(!JSON.stringify(receipt).includes("must-not-escape"));
});

await test("launch failures retain unknown status and actual resolution absence", (t) => {
  const f = fixture(t);
  const directory = path.join(f.root, "missing-process");
  assert.throws(
    () => run("missing-level-memory-image", [], f.root, directory, { PATH: f.root }),
    /spawn/,
  );
  const receipt = JSON.parse(fs.readFileSync(path.join(directory, "process.json")));
  assert.equal(receipt.status, null);
  assert.equal(receipt.imageBefore, null);
  assert.equal(receipt.error.code, "ENOENT");
  assert.equal(fs.readFileSync(path.join(directory, "stdout.bin")).length, 0);
});

await test("normal measurements select Criterion benchmark sampling explicitly", () => {
  assert.deepEqual(memory.NORMAL_ARGUMENTS, ["--bench", "--quick"]);
  assert.ok(!Object.hasOwn(MEMORY_ENVIRONMENT, "VIZE_INSTRUCTION_COUNTS"));
});

const repository = fileURLToPath(new URL("../..", import.meta.url));
const originalLevelRegistry = loadRegistry(path.join(repository, "docs/davinci/plan/budgets.toml"));
const originalFormatterRegistry = loadRegistry(
  path.join(repository, "docs/davinci/plan/formatter-instruction-registry.toml"),
);

function originalPopulation() {
  const authority = memory.loadMemoryAuthority(repository);
  const rows = new Map(
    [...authority.registry].map((id) => [
      id,
      {
        report: report(id),
        identity: {
          bench_id: id,
          fixture: `synthetic:${id}`,
          fixture_sha256: "a".repeat(64),
          window: "routine-return",
        },
      },
    ]),
  );
  return { authority, rows };
}

await test("both original source registries and thirteen providers supply the complete population", () => {
  const { authority, rows } = originalPopulation();
  assert.deepEqual(authority.planes.level.registry, originalLevelRegistry);
  assert.deepEqual(authority.planes.formatter.registry, originalFormatterRegistry);
  assert.equal(rows.size, originalLevelRegistry.size + originalFormatterRegistry.size);
  assert.deepEqual(memory.memorySuites(repository), [
    ...levelInstructionSuites(repository).map(([pkg, bench]) => ({ plane: "level", pkg, bench })),
    ...formatterInstructionSuites().map(([pkg, bench]) => ({ plane: "formatter", pkg, bench })),
  ]);
  assert.equal(authority.planes.level.allocation_budgets, true);
  assert.equal(authority.planes.formatter.allocation_budgets, false);
  const planes = memory.partitionMemoryReports(rows, authority);
  assert.deepEqual(new Set(planes.level.keys()), originalLevelRegistry);
  assert.deepEqual(new Set(planes.formatter.keys()), originalFormatterRegistry);
});

await test("missing or extra rows in either real source plane cannot qualify the whole observation", () => {
  const { authority, rows } = originalPopulation();
  for (const plane of [originalLevelRegistry, originalFormatterRegistry]) {
    const missing = new Map(rows);
    missing.delete(plane.values().next().value);
    assert.throws(() => memory.partitionMemoryReports(missing, authority), /missing/);
  }
  const extra = new Map(rows);
  extra.set("unregistered_extra", rows.values().next().value);
  assert.throws(() => memory.partitionMemoryReports(extra, authority), /unregistered/);
});

await test("formatter observations cannot enter the original level allocation budget plane", () => {
  const { authority, rows } = originalPopulation();
  const planes = memory.partitionMemoryReports(rows, authority);
  const wrongPlane = new Map(planes.level);
  const formatterId = originalFormatterRegistry.values().next().value;
  wrongPlane.set(formatterId, planes.formatter.get(formatterId));
  assert.throws(
    () => memory.validateReportPlane(wrongPlane, authority.planes.level.registry, "level budgets"),
    /unregistered/,
  );
  const forged = structuredClone(rows);
  forged.get(formatterId).report.bench_id = originalLevelRegistry.values().next().value;
  assert.throws(() => memory.partitionMemoryReports(forged, authority), /bench id/);
});

await test("every original paired metric is recorded without inventing formatter allocation caps", () => {
  const { authority, rows } = originalPopulation();
  const candidate = structuredClone(rows);
  for (const id of originalFormatterRegistry) candidate.get(id).report.allocs = 17;
  const comparisons = memory.compareMemoryMetrics(rows, candidate);
  assert.deepEqual(new Set(Object.keys(comparisons)), authority.registry);
  for (const id of authority.registry) {
    assert.deepEqual(Object.keys(comparisons[id]), [
      "allocs",
      "alloc_bytes_peak",
      "rss_peak_bytes",
      "wall_p50",
      "wall_p95",
    ]);
    assert.deepEqual(comparisons[id].allocs, {
      base: 0,
      head: originalFormatterRegistry.has(id) ? 17 : 0,
      delta: originalFormatterRegistry.has(id) ? 17 : 0,
    });
  }
  assert.equal(authority.planes.formatter.allocation_budgets, false);
});

function sourceHarnessFixture(t, version, existingRoot) {
  const root =
    existingRoot ??
    fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "memory-source-harness-")));
  if (existingRoot === undefined) t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const directory = path.join(root, "tools/benchmarks/crates/davinci_harness");
  fs.mkdirSync(path.join(directory, "src"), { recursive: true });
  fs.writeFileSync(path.join(root, "Cargo.toml"), `[workspace.package]\nversion = "${version}"\n`);
  const manifest = path.join(directory, "Cargo.toml");
  fs.writeFileSync(manifest, '[package]\nname = "davinci_harness"\nversion.workspace = true\n');
  const source = path.join(directory, "src/lib.rs");
  fs.writeFileSync(source, 'pub const VERSION: &str = env!("CARGO_PKG_VERSION");\n');
  const sourcePaths = [
    "Cargo.toml",
    "tools/benchmarks/crates/davinci_harness/Cargo.toml",
    "tools/benchmarks/crates/davinci_harness/src/lib.rs",
  ];
  const wholeTree = sourcePaths
    .map((relative) => {
      const bytes = fs.readFileSync(path.join(root, relative));
      const blob = createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex");
      return `100644 blob ${blob}\t${relative}\n`;
    })
    .join("");
  const id = `path+file://${directory}#${version}`;
  return {
    root,
    wholeTree,
    metadata: {
      workspace_root: root,
      packages: [{ name: "davinci_harness", version, id, manifest_path: manifest }],
    },
    artifacts: [
      {
        reason: "compiler-artifact",
        package_id: id,
        manifest_path: manifest,
        target: { kind: ["lib"], src_path: source },
      },
    ],
  };
}

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

await test("source-owned judge handoff waits for all624 and refuses unissued witnesses", (t) => {
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
    /104|complete/,
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

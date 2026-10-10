import assert from "node:assert/strict";
import fs from "node:fs";
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
import { run } from "../../tools/benchmarks/scripts/level-memory-runner.mjs";
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

test("fresh complete reports retain whole packets and original identities", (t) => {
  const f = fixture(t);
  const rows = collectFreshReports(f.root, f.markers, f.identities);
  assert.equal(rows.size, 2);
  assert.deepEqual(rows.get("one").report, report("one"));
  assert.equal(rows.get("one").identity.window, "routine-return");
  assert.match(rows.get("one").raw_sha256, /^[a-f0-9]{64}$/);
  validatePair(rows, rows);
  assert.ok(!Object.hasOwn(MEMORY_ENVIRONMENT, "VIZE_INSTRUCTION_COUNTS"));
});

test("tracked or cached reports without actual emitted writes are rejected", (t) => {
  const f = fixture(t);
  assert.throws(() => collectFreshReports(f.root, "", f.identities), /emitted|missing/);
  assert.throws(
    () => collectFreshReports(f.root, f.markers.split("\n")[0], f.identities),
    /missing/,
  );
});

test("duplicate, extra and foreign emitted reports cannot qualify", (t) => {
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

test("null metrics, forged fields and wrong fixture identities remain red", (t) => {
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

test("symlink output and changed same-byte source/window populations are rejected", (t) => {
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

test("a failing process preserves complete raw streams before refusing admission", (t) => {
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

test("launch failures retain unknown status and actual resolution absence", (t) => {
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

test("normal measurements select Criterion benchmark sampling explicitly", () => {
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

test("both original source registries and thirteen providers supply the complete population", () => {
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

test("missing or extra rows in either real source plane cannot qualify the whole observation", () => {
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

test("formatter observations cannot enter the original level allocation budget plane", () => {
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

test("every original paired metric is recorded without inventing formatter allocation caps", () => {
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

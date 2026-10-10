import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import {
  GUEST_ENVIRONMENT,
  ID,
  loadBudgets,
  loadRegistry,
  reconcile,
} from "./instruction-counts-lib.mjs";
import {
  formatterInstructionSuites,
  levelInstructionSuites,
} from "./instruction-counts-suites.mjs";

// Normal harness mode does not perform the instruction-only 64 MiB preinit.
// The allocator options, libc dispatch, suite order and windows stay fixed.
const normalEnvironment = { ...GUEST_ENVIRONMENT };
delete normalEnvironment.VIZE_INSTRUCTION_COUNTS;
export const MEMORY_ENVIRONMENT = Object.freeze(normalEnvironment);
export const NORMAL_ARGUMENTS = Object.freeze(["--bench", "--quick"]);
export const REPORT_DIRECTORY = "tools/benchmarks/results/davinci";
export const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");

export function memorySuites(root) {
  return [
    ...levelInstructionSuites(root).map(([pkg, bench]) => ({ plane: "level", pkg, bench })),
    ...formatterInstructionSuites().map(([pkg, bench]) => ({ plane: "formatter", pkg, bench })),
  ];
}

export function loadMemoryAuthority(root) {
  const files = {
    level: ["budgets.toml", "instruction-budgets.toml"],
    formatter: ["formatter-instruction-registry.toml", "formatter-instruction-budgets.toml"],
  };
  const registry = new Set();
  const planes = {};
  for (const [plane, names] of Object.entries(files)) {
    const [registryFile, instructionFile] = names.map((name) =>
      path.join(root, "docs/davinci/plan", name),
    );
    const original = loadRegistry(registryFile);
    for (const id of original) {
      assert.ok(!registry.has(id), `overlapping original source planes: ${id}`);
      registry.add(id);
    }
    planes[plane] = {
      registry: original,
      budgets: loadBudgets(instructionFile, original),
      registry_file: registryFile,
      instruction_file: instructionFile,
      allocation_budgets: plane === "level",
    };
  }
  assert.equal(planes.level.registry.size, 100, "original100 level registry required");
  assert.equal(planes.formatter.registry.size, 4, "original4 formatter registry required");
  return { planes, registry };
}

export function validateReportPlane(rows, registry, label) {
  reconcile(new Set(rows.keys()), registry, label);
  for (const [id, row] of rows) {
    assert.equal(row.report.bench_id, id, `${label}: wrong report bench id`);
    assert.equal(row.identity.bench_id, id, `${label}: wrong identity bench id`);
  }
}

export function partitionMemoryReports(rows, authority) {
  validateReportPlane(rows, authority.registry, "whole original104 memory population");
  return Object.fromEntries(
    Object.entries(authority.planes).map(([plane, original]) => {
      const selected = new Map([...rows].filter(([id]) => original.registry.has(id)));
      validateReportPlane(selected, original.registry, `${plane} source plane`);
      return [plane, selected];
    }),
  );
}

function fields(value, keys, label) {
  assert.ok(
    value && typeof value === "object" && !Array.isArray(value),
    `${label}: expected object`,
  );
  assert.deepEqual(Object.keys(value).sort(), [...keys].sort(), `${label}: unexpected fields`);
}
function integer(value, label) {
  assert.ok(Number.isSafeInteger(value) && value >= 0, `${label}: expected nonnegative integer`);
}
export function validateReport(report, identity) {
  fields(identity, ["bench_id", "fixture", "fixture_sha256", "window"], "actual source identity");
  assert.ok(ID.test(identity.bench_id), "invalid actual bench id");
  assert.match(identity.fixture_sha256, /^[a-f0-9]{64}$/, "invalid fixture digest");
  assert.ok(["routine-return", "stage-return"].includes(identity.window), "invalid actual window");
  fields(
    report,
    [
      "bench_id",
      "fixture",
      "platform",
      "wall_ns",
      "allocs",
      "alloc_bytes_peak",
      "rss_peak_bytes",
      "harness_version",
    ],
    "report",
  );
  assert.ok(ID.test(report.bench_id), "invalid report bench id");
  assert.equal(report.bench_id, identity.bench_id, "wrong report bench id");
  assert.equal(report.fixture, identity.fixture, "wrong report fixture");
  assert.equal(report.platform, "linux", "normal memory reports require Linux");
  assert.ok(
    typeof report.harness_version === "string" && report.harness_version.length,
    "missing harness version",
  );
  fields(report.wall_ns, ["p50", "p95"], "wall_ns");
  for (const metric of ["allocs", "alloc_bytes_peak", "rss_peak_bytes"])
    integer(report[metric], metric);
  for (const percentile of ["p50", "p95"]) integer(report.wall_ns[percentile], percentile);
  assert.ok(report.wall_ns.p95 >= report.wall_ns.p50, "p95 below p50");
}

function regularFileWithin(file, directory) {
  assert.equal(path.dirname(file), directory, `foreign report outside ${directory}`);
  const parent = fs.lstatSync(directory);
  assert.ok(parent.isDirectory() && !parent.isSymbolicLink(), "symlink report directory");
  assert.equal(fs.realpathSync(directory), directory, "symlink report parent");
  const stat = fs.lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink(), "symlink or nonregular report output");
}

// A tracked report is insufficient: every admitted file needs the exact write
// marker from this completed suite invocation. Never read un-emitted files.
export function collectFreshReports(root, stderr, identities) {
  const directory = path.join(path.resolve(root), REPORT_DIRECTORY);
  const emitted = new Map();
  for (const line of stderr.split("\n")) {
    if (!line.startsWith("davinci-harness: wrote ")) continue;
    const file = line.slice("davinci-harness: wrote ".length);
    assert.ok(path.isAbsolute(file), "foreign relative report marker");
    regularFileWithin(file, directory);
    const id = path.basename(file, ".json");
    assert.equal(path.basename(file), `${id}.json`, "non-JSON report marker");
    assert.ok(identities.has(id), `unregistered emitted report ${id}`);
    assert.ok(!emitted.has(id), `duplicate emitted report ${id}`);
    const raw = fs.readFileSync(file);
    const report = JSON.parse(raw);
    const identity = identities.get(id);
    validateReport(report, identity);
    emitted.set(id, { report, identity: { ...identity }, raw_sha256: sha256(raw), file });
  }
  reconcile(new Set(emitted.keys()), new Set(identities.keys()), "fresh emitted reports");
  return emitted;
}

export function validatePair(base, head) {
  reconcile(new Set(head.keys()), new Set(base.keys()), "paired memory population");
  for (const [id, original] of base) {
    const current = head.get(id);
    assert.deepEqual(current.identity, original.identity, `${id}: changed input/window identity`);
    for (const field of ["fixture", "platform", "harness_version"]) {
      assert.equal(current.report[field], original.report[field], `${id}: changed ${field}`);
    }
  }
}

export function compareMemoryMetrics(base, head) {
  validatePair(base, head);
  return Object.fromEntries(
    [...base].map(([id, original]) => {
      const current = head.get(id);
      const metrics = {};
      for (const key of ["allocs", "alloc_bytes_peak", "rss_peak_bytes", "wall_p50", "wall_p95"]) {
        const metric = (row) =>
          key.startsWith("wall_") ? row.report.wall_ns[key.slice(5)] : row.report[key];
        const before = metric(original);
        const after = metric(current);
        integer(before, `${id}: baseline ${key}`);
        integer(after, `${id}: candidate ${key}`);
        metrics[key] = { base: before, head: after, delta: after - before };
      }
      return [id, metrics];
    }),
  );
}

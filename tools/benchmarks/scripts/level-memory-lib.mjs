import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
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
export const compareStrings = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
const harnessSourceWitnesses = new WeakSet();
const harnessSourcePins = new WeakSet();
const HARNESS_SOURCE_PATHS = Object.freeze([
  "Cargo.toml",
  "tools/benchmarks/crates/davinci_harness/Cargo.toml",
  "tools/benchmarks/crates/davinci_harness/src/lib.rs",
]);

function harnessSourceFile(root, relative) {
  const file = path.join(root, relative);
  const stat = fs.lstatSync(file);
  assert.ok(stat.isFile() && !stat.isSymbolicLink(), "foreign harness source file");
  assert.equal(fs.realpathSync(file), file, "foreign harness source parent");
  const bytes = fs.readFileSync(file);
  return {
    path: relative,
    git_blob: createHash("sha1").update(`blob ${bytes.length}\0`).update(bytes).digest("hex"),
    sha256: sha256(bytes),
  };
}

// Reuse the already retained exact-head Git listing before Cargo runs. These
// are before/after source observations, not compiler-open-time telemetry.
export function captureHarnessSource(root, wholeTree) {
  root = path.resolve(root);
  assert.equal(fs.realpathSync(root), root, "foreign harness source root");
  const lines = wholeTree.split("\n");
  const files = HARNESS_SOURCE_PATHS.map((relative) => {
    const matches = lines.filter((line) => line.endsWith(`\t${relative}`));
    assert.equal(matches.length, 1, "missing or duplicate harness Git source");
    const entry = /^100644 blob ([a-f0-9]{40})\t/.exec(matches[0]);
    assert.ok(entry, "wrong harness Git source mode or blob");
    assert.equal(
      matches[0],
      `100644 blob ${entry[1]}\t${relative}`,
      "malformed harness Git source entry",
    );
    const observed = harnessSourceFile(root, relative);
    assert.equal(observed.git_blob, entry[1], "harness source differs from pinned Git blob");
    return Object.freeze(observed);
  });
  const pin = Object.freeze({ root, files: Object.freeze(files) });
  harnessSourcePins.add(pin);
  return pin;
}

function validateHarnessSourcePin(root, pin) {
  assert.ok(harnessSourcePins.has(pin), "missing authenticated harness source pin");
  assert.equal(pin.root, root, "foreign harness source pin root");
  for (const original of pin.files)
    assert.deepEqual(harnessSourceFile(root, original.path), original, "harness source changed");
}

// Cargo owns version inheritance. Join its exact source metadata to the actual
// library compiler artifact; a copied or caller-written object is no witness.
export function bindHarnessSource(root, metadata, artifacts, sourcePin) {
  root = path.resolve(root);
  assert.equal(fs.realpathSync(root), root, "foreign harness source root");
  validateHarnessSourcePin(root, sourcePin);
  assert.equal(metadata.workspace_root, root, "foreign Cargo metadata workspace");
  assert.ok(Array.isArray(metadata.packages), "missing harness source packages");
  assert.ok(Array.isArray(artifacts), "missing harness build artifacts");
  const manifest = path.join(root, "tools/benchmarks/crates/davinci_harness/Cargo.toml");
  const source = path.join(path.dirname(manifest), "src/lib.rs");
  const packages = metadata.packages.filter((pkg) => pkg.name === "davinci_harness");
  assert.equal(packages.length, 1, "missing or duplicate source harness package");
  const pkg = packages[0];
  assert.equal(pkg.manifest_path, manifest, "foreign harness source manifest");
  assert.ok(
    typeof pkg.version === "string" && pkg.version.length,
    "missing harness source version",
  );
  assert.ok(
    typeof pkg.id === "string" && pkg.id.startsWith("path+file:"),
    "foreign harness package ID",
  );
  const packageURL = new URL(pkg.id.slice("path+".length));
  assert.equal(fileURLToPath(packageURL), path.dirname(manifest), "foreign harness package source");
  assert.equal(
    packageURL.hash,
    `#${pkg.version}`,
    "harness source version differs from package ID",
  );
  const libraries = artifacts.filter(
    (row) =>
      row.reason === "compiler-artifact" &&
      row.manifest_path === manifest &&
      Array.isArray(row.target?.kind) &&
      row.target.kind.includes("lib"),
  );
  assert.equal(libraries.length, 1, "missing or duplicate harness library artifact");
  assert.equal(
    libraries[0].package_id,
    pkg.id,
    "harness library package differs from source metadata",
  );
  assert.equal(libraries[0].target.src_path, source, "foreign harness library source");
  const witness = Object.freeze({
    root,
    version: pkg.version,
    package_id: pkg.id,
    manifest_path: manifest,
    manifest_sha256: sourcePin.files[1].sha256,
    workspace_manifest_sha256: sourcePin.files[0].sha256,
    library_source: source,
    library_source_sha256: sourcePin.files[2].sha256,
    source_pin: sourcePin,
  });
  harnessSourceWitnesses.add(witness);
  return witness;
}

export function validateHarnessReports(rows, witness) {
  assert.ok(harnessSourceWitnesses.has(witness), "missing authenticated harness source witness");
  validateHarnessSourcePin(witness.root, witness.source_pin);
  for (const [id, row] of rows) {
    assert.equal(
      row.file,
      path.join(witness.root, REPORT_DIRECTORY, `${id}.json`),
      `${id}: report differs from own source arm`,
    );
    assert.equal(
      row.report.harness_version,
      witness.version,
      `${id}: changed own-source harness_version`,
    );
  }
}

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
  assert.deepEqual(
    Object.keys(value).sort(compareStrings),
    [...keys].sort(compareStrings),
    `${label}: unexpected fields`,
  );
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

export function validatePair(base, head, sourceHarness) {
  if (sourceHarness !== undefined) {
    fields(sourceHarness, ["base", "head"], "paired harness source witness");
    validateHarnessReports(base, sourceHarness.base);
    validateHarnessReports(head, sourceHarness.head);
  }
  reconcile(new Set(head.keys()), new Set(base.keys()), "paired memory population");
  for (const [id, original] of base) {
    const current = head.get(id);
    assert.deepEqual(current.identity, original.identity, `${id}: changed input/window identity`);
    const shared =
      sourceHarness === undefined
        ? ["fixture", "platform", "harness_version"]
        : ["fixture", "platform"];
    for (const field of shared) {
      assert.equal(current.report[field], original.report[field], `${id}: changed ${field}`);
    }
  }
}

export function compareMemoryMetrics(base, head, sourceHarness) {
  validatePair(base, head, sourceHarness);
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

// This carrier only selects complete stored source/report roles. The Rust
// judge independently validates source, build and protocol custody; no JSON
// version claim or serialized WeakSet witness grants comparison authority.
export function writeHarnessJudgePair(out, manifest, witnesses, arm, repetition) {
  assert.equal(manifest.observations_complete, true, "all624 observations must be complete");
  assert.ok(["base", "head"].includes(arm), "foreign judge source arm");
  assert.ok(
    Number.isInteger(repetition) && repetition >= 1 && repetition <= 3,
    "foreign judge repetition",
  );
  fields(witnesses, ["base", "head"], "judge source witnesses");
  for (const side of ["base", "head"]) {
    assert.equal(manifest.measurements[side].length, 3, "three complete repetitions required");
    for (const rows of manifest.measurements[side]) {
      assert.equal(Object.keys(rows).length, 104, "all104 rows required before judge");
      validateHarnessReports(new Map(Object.entries(rows)), witnesses[side]);
    }
  }
  out = path.resolve(out);
  assert.equal(fs.realpathSync(out), out, "foreign judge evidence directory");
  const measurement = path.join(out, "measurement.json");
  assert.deepEqual(JSON.parse(fs.readFileSync(measurement)), manifest, "changed judge measurement");
  const directory = path.join(out, "source-harness-pairs");
  fs.mkdirSync(directory, { recursive: true });
  assert.equal(fs.realpathSync(directory), directory, "foreign judge pair directory");
  const file = path.join(directory, `${arm}-${repetition}.json`);
  fs.writeFileSync(
    file,
    `${JSON.stringify(
      {
        schema_version: 1,
        measurement,
        baseline: { arm: "base", repetition },
        current: { arm, repetition },
      },
      null,
      2,
    )}\n`,
    { flag: "wx" },
  );
  return file;
}

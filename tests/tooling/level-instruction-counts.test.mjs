import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  baselineToml,
  checkMeasurement,
  fixtureDigest,
  loadBudgets,
  parseCallgrind,
  parseIdentities,
  ratchetBudgets,
  reconcile,
  validateBudgets,
  validateMeasurement,
  METHODOLOGY_KEYS,
  BENCH_KEYS,
} from "../../tools/benchmarks/scripts/instruction-counts-lib.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const registry = new Set(["parse_small", "transform_small"]);
const row = (instructions, window = "routine-return") => ({
  fixture: "synthetic:small",
  fixture_sha256: "a".repeat(64),
  instructions,
  window,
});
const method = {
  allocator: "counting-mimalloc",
  flags: "target-cpu=x86-64;instr-atstart=no;cache-sim=no;branch-sim=no",
  libc: "glibc 2.39",
  profile: "ci-opt",
  rustc: "rustc 1.98.0 (fixture)",
  target: "x86_64-unknown-linux-gnu",
  valgrind: "valgrind-3.22.0",
  window_protocol: "callgrind-client-v1",
};
const measurement = () => ({
  schema_version: 1,
  source_commit: "b".repeat(40),
  recorded_run: "https://github.com/ubugeeei-prod/vize/actions/runs/123",
  methodology: { ...method },
  runs: Array.from({ length: 3 }, () => ({
    parse_small: row(100),
    transform_small: row(200, "stage-return"),
  })),
});
const budgets = () => {
  const { runs, ...metadata } = measurement();
  return { ...metadata, instruction: runs[0] };
};
const dump = (trigger = "Client Request: parse_small", events = "Ir", total = "100") =>
  `# callgrind format\nversion: 1\ncreator: callgrind-3.22.0\npart: 1\n` +
  `desc: Trigger: ${trigger}\npositions: line\nevents: ${events}\nsummary: ${total}\n` +
  `fn=caller\n1 40\ncfn=callee\ncalls=1 1\n1 60\nfn=callee\n1 60\ntotals: ${total}\n`;

test("Callgrind parser uses exclusive totals, preserving named stage identity", () => {
  assert.deepEqual(parseCallgrind(dump()), { bench_id: "parse_small", instructions: 100 });
  assert.equal(parseCallgrind(dump("Program termination", "Ir", "0")), null);
  assert.throws(() => parseCallgrind(dump("Program termination")), /leaked/);
  assert.throws(() => parseCallgrind(dump("Client Request: parse_small", "Ir Dr")), /only Ir/);
  assert.throws(() => parseCallgrind(`${dump()}totals: 100\n`), /exactly one totals/);
  assert.throws(() => parseCallgrind(dump("Client Request: parse_small", "Ir", "0")), /positive/);
  for (const malformed of ["NaN", "-1", "1.5", "9007199254740992", "1 2"]) {
    assert.throws(() => parseCallgrind(dump("Client Request: parse_small", "Ir", malformed)));
  }
  assert.throws(() => parseCallgrind(dump("Client Request: ../escape")), /invalid named dump/);
  assert.throws(() => parseCallgrind(dump("Periodic dump")), /unexpected trigger/);
});

test("named dump identities cannot disappear, duplicate, or introduce unregistered stages", () => {
  const line = `VIZE_INSTRUCTION_BENCH ${JSON.stringify({ bench_id: "parse_small", fixture: "small.vue", window: "routine-return" })}`;
  assert.equal(parseIdentities(`ordinary stderr\n${line}`).size, 1);
  assert.throws(() => parseIdentities(`${line}\n${line}`), /duplicate identity/);
  assert.throws(() => parseIdentities("ordinary stderr"), /no instrumented/);
  assert.throws(
    () => reconcile(new Set(["unknown"]), registry, "fixture"),
    /missing.*parse_small.*unregistered.*unknown/,
  );
  assert.throws(() => reconcile(new Set(["parse_small"]), registry, "fixture"), /transform_small/);
});

test("only three identical complete measurements can produce a baseline", () => {
  assert.equal(validateMeasurement(measurement(), registry).runs.length, 3);
  const unstable = measurement();
  unstable.runs[2].transform_small.instructions += 1;
  assert.throws(() => validateMeasurement(unstable, registry), /unstable.*transform_small/);
  const missing = measurement();
  delete missing.runs[1].parse_small;
  assert.throws(() => validateMeasurement(missing, registry), /missing.*parse_small/);
  const two = measurement();
  two.runs.pop();
  assert.throws(() => validateMeasurement(two, registry), /three measurement/);
  for (const invalid of [0, -1, NaN, Infinity, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    const bad = measurement();
    bad.runs[0].parse_small.instructions = invalid;
    assert.throws(() => validateMeasurement(bad, registry), /positive safe integer/);
  }
});

test("provenance, fixture identity, stage window, and methodology fail closed", () => {
  for (const mutate of [
    (r) => {
      r.source_commit = "main";
    },
    (r) => {
      r.recorded_run = "local";
    },
    (r) => {
      r.methodology.target = "aarch64-apple-darwin";
    },
    (r) => {
      r.methodology.flags = "native";
    },
    (r) => {
      r.runs[0].parse_small.fixture_sha256 = "missing";
    },
    (r) => {
      r.runs[0].parse_small.window = "entire-process";
    },
    (r) => {
      r.runs[0].parse_small.extra = 1;
    },
  ]) {
    const bad = measurement();
    mutate(bad);
    assert.throws(() => validateMeasurement(bad, registry));
  }
  const changed = measurement();
  changed.methodology.rustc = "rustc newer";
  assert.throws(() => checkMeasurement(changed, budgets()), /methodology changed/);
  const fixture = measurement();
  fixture.runs[0].parse_small.fixture_sha256 = "c".repeat(64);
  assert.throws(() => checkMeasurement(fixture, budgets()), /identity changed/);
});

test("ceilings enforce instruction regressions and permit improvements", () => {
  checkMeasurement(measurement(), budgets());
  const improvement = measurement();
  improvement.runs[0].parse_small.instructions = 99;
  checkMeasurement(improvement, budgets());
  const regression = measurement();
  regression.runs[0].transform_small.instructions = 201;
  assert.throws(() => checkMeasurement(regression, budgets()), /transform_small: 201 > 200/);
});

test("ratchet rejects increased or removed ceilings, including a freshly edited baseline", () => {
  const lowered = budgets();
  lowered.instruction.parse_small.instructions = 99;
  ratchetBudgets(lowered, budgets());
  const increased = budgets();
  increased.instruction.parse_small.instructions = 101;
  assert.throws(() => ratchetBudgets(increased, budgets()), /loosened: parse_small/);
  const removed = budgets();
  delete removed.instruction.parse_small;
  assert.throws(() => ratchetBudgets(removed, budgets()), /dropped: parse_small/);
});

test("measured TOML round-trips strictly and CLI refuses a missing baseline", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-budget-test-"));
  try {
    const baseline = path.join(temporary, "budgets.toml");
    fs.writeFileSync(baseline, baselineToml(measurement()));
    assert.deepEqual(loadBudgets(baseline, registry), validateBudgets(budgets(), registry));
    fs.appendFileSync(baseline, "parse_small = { instructions = 200 }\n");
    assert.throws(() => loadBudgets(baseline, registry), /duplicate/);
    const report = path.join(temporary, "report.json");
    const registered = path.join(temporary, "registry.toml");
    fs.writeFileSync(report, JSON.stringify(measurement()));
    fs.writeFileSync(registered, "[bench]\nparse_small = {}\ntransform_small = {}\n");
    const child = spawnSync(
      process.execPath,
      [
        "tools/benchmarks/scripts/instruction-counts.mjs",
        "--check",
        "--measurement",
        report,
        "--registry",
        registered,
        "--budgets",
        path.join(temporary, "absent.toml"),
      ],
      { cwd: root, encoding: "utf8" },
    );
    assert.equal(child.status, 1);
    assert.match(child.stderr, /absent.toml/);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("synthetic fixture digests also track included source fixtures", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-identity-test-"));
  try {
    const source = path.join(temporary, "bench.rs");
    fs.writeFileSync(source, 'include_str!("fixture.vue");');
    fs.writeFileSync(path.join(temporary, "fixture.vue"), "<div/>");
    const before = fixtureDigest(temporary, "synthetic:div", source);
    fs.writeFileSync(path.join(temporary, "fixture.vue"), "<span/>");
    assert.notEqual(fixtureDigest(temporary, "synthetic:div", source), before);
    assert.throws(() => fixtureDigest(temporary, "../escape", source), /escapes workspace/);
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("the independent JSON schema preserves the gate's strict field contract", () => {
  const schema = JSON.parse(
    fs.readFileSync(
      path.join(
        root,
        "tools/benchmarks/crates/davinci_harness/schema/instruction-measurement.schema.json",
      ),
      "utf8",
    ),
  );
  assert.deepEqual(schema.required.sort(), Object.keys(measurement()).sort());
  assert.deepEqual(schema.properties.methodology.required.sort(), [...METHODOLOGY_KEYS].sort());
  const bench = schema.properties.runs.items.patternProperties["^[A-Za-z0-9._-]+$"];
  assert.deepEqual(bench.required.sort(), [...BENCH_KEYS].sort());
  assert.equal(bench.additionalProperties, false);
  assert.equal(bench.properties.instructions.maximum, Number.MAX_SAFE_INTEGER);
  assert.equal(schema.properties.runs.minItems, 3);
  assert.equal(schema.properties.runs.maxItems, 3);
});

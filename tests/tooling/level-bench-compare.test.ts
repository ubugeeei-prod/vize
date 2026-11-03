// The davinci bench budget compare gate (plan/phase-0.md P0-4).
//
// Exact stdout/stderr/exit oracles for tools/commands/davinci/bench-compare.rs over
// the committed fixture pairs in tests/_fixtures/davinci-bench-compare/,
// including the DAVINCI_BASELINE_REFRESH refusal path. The registry side of
// P0-4 (budgets.toml reconciliation and the ratchet header) lives in
// tests/tooling/davinci-budgets.test.ts.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const comparePath = path.join(repoRoot, "tools", "commands", "davinci", "bench-compare.rs");
const fixtureRel = "tests/_fixtures/davinci-bench-compare";
const fixtureDir = path.join(repoRoot, fixtureRel);

function runCompare(
  args: string[],
  options: { cwd?: string; refreshEnv?: string } = {},
): ReturnType<typeof spawnSync<string>> {
  const env = { ...process.env };
  delete env.DAVINCI_BASELINE_REFRESH;
  if (options.refreshEnv != null) env.DAVINCI_BASELINE_REFRESH = options.refreshEnv;
  return spawnSync("rust-script", [comparePath, ...args], {
    cwd: options.cwd ?? repoRoot,
    encoding: "utf8",
    env,
  });
}

function compareArgs(scenario: string, overrides: { budgets?: string; baseline?: string } = {}) {
  return [
    "--budgets",
    overrides.budgets ?? `${fixtureRel}/budgets.toml`,
    "--baseline",
    overrides.baseline ?? `${fixtureRel}/baseline`,
    "--results",
    `${fixtureRel}/${scenario}/current`,
  ];
}

function header(scenario: string, overrides: { budgets?: string; baseline?: string } = {}) {
  return (
    `bench-compare: budgets=${overrides.budgets ?? `${fixtureRel}/budgets.toml`} ` +
    `baseline=${overrides.baseline ?? `${fixtureRel}/baseline`} ` +
    `results=${fixtureRel}/${scenario}/current\n`
  );
}

test("bench-compare exits 0 when the current run sits inside each bench's own tolerance", () => {
  const result = runCompare(compareArgs("within-tolerance"));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 0, result.stdout);
  // The +8% transform run is inside its 0.10 stage-window tolerance but would
  // breach the 0.05 whole-routine one, so this also pins per-bench tolerance.
  assert.equal(
    result.stdout,
    header("within-tolerance") +
      "ok fixture_parse_small wall_p50 104000ns (baseline 100000ns limit 105000ns) " +
      "allocs 42 rss 8192B\n" +
      "ok fixture_transform_medium wall_p50 216000ns (baseline 200000ns limit 220000ns) " +
      "allocs 1000 rss 16384B\n" +
      "bench-compare: breaches=0 gated_ok=2 alloc_gated=0 registered=2\n",
  );
});

test("bench-compare exits 1 when wall p50 breaches the baseline tolerance", () => {
  const result = runCompare(compareArgs("wall-breach"));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 1, result.stdout);
  assert.equal(
    result.stdout,
    header("wall-breach") +
      "FAIL fixture_parse_small wall_p50 106000ns > limit 105000ns " +
      "(baseline 100000ns + 5% tolerance)\n" +
      "ok fixture_transform_medium wall_p50 216000ns (baseline 200000ns limit 220000ns) " +
      "allocs 1000 rss 16384B\n" +
      "bench-compare: breaches=1 gated_ok=1 alloc_gated=0 registered=2\n",
  );
});

test("bench-compare exits 1 on any allocation-count change (exact gate)", () => {
  const result = runCompare(compareArgs("allocs-breach"));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 1, result.stdout);
  assert.equal(
    result.stdout,
    header("allocs-breach") +
      "FAIL fixture_parse_small allocs 42 -> 43 " +
      "(exact gate against budgets.toml: allocs are deterministic and machine-independent)\n" +
      "ok fixture_transform_medium wall_p50 216000ns (baseline 200000ns limit 220000ns) " +
      "allocs 1000 rss 16384B\n" +
      "bench-compare: breaches=1 gated_ok=1 alloc_gated=0 registered=2\n",
  );
});

test("bench-compare exits 1 on registry drift in either direction", () => {
  const result = runCompare(compareArgs("drift"));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 1, result.stdout);
  assert.equal(
    result.stdout,
    header("drift") +
      "ok fixture_parse_small wall_p50 104000ns (baseline 100000ns limit 105000ns) " +
      "allocs 42 rss 8192B\n" +
      "FAIL fixture_transform_medium bench disappeared " +
      "(budgets.toml entry has no current result)\n" +
      "FAIL fixture_unknown_new unregistered bench " +
      "(current result has no budgets.toml [bench] entry)\n" +
      "bench-compare: breaches=2 gated_ok=1 alloc_gated=0 registered=2\n",
  );
});

test("bench-compare still gates allocs when no baseline report exists", () => {
  const overrides = { baseline: `${fixtureRel}/no-baseline` };
  const result = runCompare(compareArgs("within-tolerance", overrides));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 0, result.stdout);
  assert.equal(
    result.stdout,
    header("within-tolerance", overrides) +
      "alloc-gated fixture_parse_small allocs 42 ok " +
      "(wall_p50 104000ns report-only: no committed baseline report) rss 8192B\n" +
      "alloc-gated fixture_transform_medium allocs 1000 ok " +
      "(wall_p50 216000ns report-only: no committed baseline report) rss 16384B\n" +
      "bench-compare: breaches=0 gated_ok=0 alloc_gated=2 registered=2\n",
  );
});

test("bench-compare fails closed when the baseline reports path is not a directory", () => {
  const overrides = { baseline: `${fixtureRel}/budgets.toml` };
  const result = runCompare(compareArgs("within-tolerance", overrides));
  assert.equal(result.status, 2, result.stderr);
  assert.equal(result.stdout, header("within-tolerance", overrides));
  assert.match(
    result.stderr,
    /^bench-compare: baseline reports directory tests\/_fixtures\/davinci-bench-compare\/budgets\.toml cannot be read: /u,
  );
  assert.match(result.stderr, /ENOTDIR|not a directory/u);
});

function runMalformedReport(target: "current" | "baseline", mutate: (report: any) => void) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-bench-report-shape-"));
  const baseline = path.join(root, "baseline");
  const current = path.join(root, "current");
  const budgets = path.join(root, "budgets.toml");
  fs.mkdirSync(baseline);
  fs.mkdirSync(current);
  fs.writeFileSync(
    budgets,
    "[bench]\nfixture_parse_small = { wall_p50_ns = 100000, allocs = 42, " +
      "rss_peak_bytes = 8192, wall_tolerance = 0.05 }\n",
  );
  const report = JSON.parse(
    fs.readFileSync(path.join(fixtureDir, "baseline/fixture_parse_small.json"), "utf8"),
  );
  for (const dir of [baseline, current]) {
    const copy = structuredClone(report);
    if ((target === "baseline") === (dir === baseline)) mutate(copy);
    fs.writeFileSync(path.join(dir, "fixture_parse_small.json"), `${JSON.stringify(copy)}\n`);
  }
  const result = runCompare(["--budgets", budgets, "--baseline", baseline, "--results", current]);
  fs.rmSync(root, { recursive: true, force: true });
  return result;
}

test("bench-compare rejects malformed bench report shapes before budget verdicts", () => {
  const current = runMalformedReport("current", (report) => {
    report.unexpected = true;
  });
  assert.equal(current.status, 2, current.stderr);
  assert.equal(current.stdout, "");
  assert.match(current.stderr, /current report .* has unknown fields unexpected/u);

  const harness = runMalformedReport("current", (report) => {
    report.harness_version = "";
  });
  assert.equal(harness.status, 2, harness.stderr);
  assert.match(harness.stderr, /current report .* has no valid harness_version/u);
  const wallOrder = runMalformedReport("current", (report) => {
    report.wall_ns.p95 = report.wall_ns.p50 - 1;
  });
  assert.equal(wallOrder.status, 2, wallOrder.stderr);
  assert.match(wallOrder.stderr, /current report .* has wall_ns.p95 below wall_ns.p50/u);

  const baseline = runMalformedReport("baseline", (report) => {
    report.wall_ns.p99 = 120000;
  });
  assert.equal(baseline.status, 2, baseline.stderr);
  assert.match(baseline.stdout, /^bench-compare: budgets=.* baseline=.* results=.*\n/u);
  assert.match(baseline.stderr, /baseline report .* wall_ns has unknown fields p99/u);
});

test("bench-compare reports the wall side but gates allocs when the wall baseline is unrecorded", () => {
  const overrides = { budgets: `${fixtureRel}/budgets-unrecorded.toml` };
  const result = runCompare(compareArgs("within-tolerance", overrides));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 0, result.stdout);
  assert.equal(
    result.stdout,
    header("within-tolerance", overrides) +
      "alloc-gated fixture_parse_small allocs 42 ok " +
      "(wall_p50 104000ns report-only: budgets.toml wall baseline not yet recorded) rss 8192B\n" +
      "alloc-gated fixture_transform_medium allocs 1000 ok " +
      "(wall_p50 216000ns report-only: budgets.toml wall baseline not yet recorded) rss 16384B\n" +
      "bench-compare: breaches=0 gated_ok=0 alloc_gated=2 registered=2\n",
  );
});

test("--bench selects the storage probe and gates its deterministic peak", () => {
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-storage-peak-"));
  const budgetsPath = path.join(tmpRoot, "budgets.toml");
  const results = path.join(tmpRoot, "results");
  fs.mkdirSync(results);
  fs.writeFileSync(
    budgetsPath,
    "[bench]\n" +
      "fixture_parse_small = { wall_p50_ns = 100000, allocs = 42, " +
      "rss_peak_bytes = 0, wall_tolerance = 0.05 }\n" +
      "fixture_transform_medium = { wall_p50_ns = 200000, allocs = 1000, " +
      "rss_peak_bytes = 0, wall_tolerance = 0.10 }\n" +
      "[allocation_peak]\nfixture_parse_small = { linux = 4096, macos = 4096 }\n",
  );
  const report = JSON.parse(
    fs.readFileSync(
      path.join(fixtureDir, "within-tolerance/current/fixture_parse_small.json"),
      "utf8",
    ),
  );
  const reportPath = path.join(results, "fixture_parse_small.json");
  fs.writeFileSync(reportPath, `${JSON.stringify(report)}\n`);
  const args = [
    "--budgets",
    budgetsPath,
    "--baseline",
    path.join(fixtureDir, "baseline"),
    "--results",
    results,
    "--bench",
    "fixture_parse_small",
  ];
  try {
    const passing = runCompare(args);
    assert.equal(passing.stderr, "");
    assert.equal(passing.status, 0, passing.stdout);
    assert.match(
      passing.stdout,
      /ok fixture_parse_small .*allocs 42 alloc_bytes_peak\[linux\] 4096B .*\n/u,
    );
    assert.match(passing.stdout, /registered=1\n$/u);
    assert.doesNotMatch(passing.stdout, /fixture_transform_medium/u);

    report.alloc_bytes_peak = 4097;
    fs.writeFileSync(reportPath, `${JSON.stringify(report)}\n`);
    const breach = runCompare(args);
    assert.equal(breach.status, 1, breach.stdout);
    assert.match(
      breach.stdout,
      /FAIL fixture_parse_small alloc_bytes_peak\[linux\] 4096 -> 4097 \(exact platform-aware peak gate\)/u,
    );
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
});

test("the alloc gate bites before the reference runner records any wall baseline", () => {
  // The state phase 1 exits in: wall numbers await Blacksmith, allocation
  // counts are already the ratchet. An alloc regression must fail here.
  const overrides = { budgets: `${fixtureRel}/budgets-unrecorded.toml` };
  const result = runCompare(compareArgs("allocs-breach", overrides));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 1, result.stdout);
  assert.equal(
    result.stdout,
    header("allocs-breach", overrides) +
      "FAIL fixture_parse_small allocs 42 -> 43 " +
      "(exact gate against budgets.toml: allocs are deterministic and machine-independent)\n" +
      "alloc-gated fixture_transform_medium allocs 1000 ok " +
      "(wall_p50 216000ns report-only: budgets.toml wall baseline not yet recorded) rss 16384B\n" +
      "bench-compare: breaches=1 gated_ok=0 alloc_gated=1 registered=2\n",
  );
});

const refusalMessage =
  "bench-compare: refusing --update-baseline without DAVINCI_BASELINE_REFRESH=1.\n" +
  "The committed baseline is the reference every PR is gated against; refreshing it\n" +
  "must be a deliberate act on the reference runner, not a side effect. Re-run with\n" +
  "DAVINCI_BASELINE_REFRESH=1 in the environment to proceed.\n";

function baselineSnapshot(dir: string): Record<string, string> {
  const snapshot: Record<string, string> = {};
  for (const name of fs.readdirSync(dir).sort()) {
    snapshot[name] = fs.readFileSync(path.join(dir, name), "utf8");
  }
  return snapshot;
}

test("--update-baseline refuses without DAVINCI_BASELINE_REFRESH=1 and writes nothing", () => {
  const baselineDir = path.join(fixtureDir, "baseline");
  const before = baselineSnapshot(baselineDir);
  for (const refreshEnv of [undefined, "0"]) {
    const result = runCompare([...compareArgs("within-tolerance"), "--update-baseline"], {
      refreshEnv,
    });
    assert.equal(result.status, 2, result.stderr);
    assert.equal(result.stdout, "");
    assert.equal(result.stderr, refusalMessage);
  }
  assert.deepEqual(baselineSnapshot(baselineDir), before, "refusal must not touch the baseline");
});

test("--update-baseline copies current over baseline when the refresh env var is set", () => {
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "davinci-bench-compare-"));
  try {
    fs.cpSync(fixtureDir, tmpRoot, { recursive: true });
    const result = runCompare(
      [
        "--budgets",
        "budgets.toml",
        "--baseline",
        "baseline",
        "--results",
        "within-tolerance/current",
        "--update-baseline",
      ],
      { cwd: tmpRoot, refreshEnv: "1" },
    );
    assert.equal(result.stderr, "");
    assert.equal(result.status, 0, result.stdout);
    assert.equal(
      result.stdout,
      "bench-compare: budgets=budgets.toml baseline=baseline results=within-tolerance/current\n" +
        "updated baseline fixture_parse_small\n" +
        "updated baseline fixture_transform_medium\n" +
        "bench-compare: baseline updated (2 benches) under baseline\n",
    );
    assert.deepEqual(
      baselineSnapshot(path.join(tmpRoot, "baseline")),
      baselineSnapshot(path.join(tmpRoot, "within-tolerance", "current")),
      "the refreshed baseline must be a byte copy of the current reports",
    );
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
});

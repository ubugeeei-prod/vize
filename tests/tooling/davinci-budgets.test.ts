// Davinci bench budget registry + compare gate (plan/phase-0.md P0-4).
//
// Three suites:
//   1. Registry reconciliation — every bench id constructed in the davinci
//      bench sources (cstr!-built or literal) has a [bench.<id>] entry in
//      davinci-road/plan/budgets.toml, and vice versa, with the offending id
//      named on mismatch. Entries carry exactly the documented field set and
//      hold the tolerance ceilings (0.05, 0.10 for `_transform_` stage
//      windows) — ceilings, not equalities, because the ratchet allows
//      tightening.
//   2. The ratchet header — budgets.toml carries the exact machine-checked
//      ratchet line (the documented-in-file variant of the P0-4 ratchet rule).
//   3. The bench-compare gate — exact stdout/stderr/exit oracles over the
//      committed fixture pairs in tests/_fixtures/davinci-bench-compare/,
//      including the DAVINCI_BASELINE_REFRESH refusal path.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parseTomlLite } from "../../tools/davinci/toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const budgetsPath = path.join(repoRoot, "davinci-road", "plan", "budgets.toml");
const comparePath = path.join(repoRoot, "tools", "davinci", "bench-compare.mjs");
const fixtureRel = "tests/_fixtures/davinci-bench-compare";
const fixtureDir = path.join(repoRoot, fixtureRel);

const budgetsText = fs.readFileSync(budgetsPath, "utf8");
const budgets = parseTomlLite(budgetsText) as {
  bench: Record<string, Record<string, unknown>>;
};

// --- bench-id enumeration from the bench sources -------------------------
//
// Bench ids are constructed either as a direct string literal passed to
// `bench_with_metrics(criterion, "...")` or via `cstr!("template", args…)`
// with `{}` placeholders. The reconciler expands the templates over the
// argument domains it understands (`fixture.name` — the harness LADDER;
// `preset_name` — the file's `presets` tuple array) and fails loudly on any
// construction it cannot resolve, so a new bench pattern extends this test
// instead of silently escaping the registry.

function ladderNames(): string[] {
  const fixturesRs = fs.readFileSync(
    path.join(repoRoot, "benchmarks", "davinci_harness", "src", "fixtures.rs"),
    "utf8",
  );
  const start = fixturesRs.indexOf("pub const LADDER");
  assert.ok(start >= 0, "fixtures.rs must declare the LADDER const");
  const block = fixturesRs.slice(start, fixturesRs.indexOf("];", start));
  const names = [...block.matchAll(/name: "([^"]+)"/g)].map(([, name]) => name);
  assert.ok(names.length > 0, "the LADDER const must name at least one fixture");
  assert.deepEqual([...new Set(names)], names, "LADDER fixture names must be unique");
  return names;
}

function benchSources(): { file: string; text: string }[] {
  const sources: { file: string; text: string }[] = [];
  for (const root of ["crates", "benchmarks"]) {
    for (const pkg of fs.readdirSync(path.join(repoRoot, root))) {
      const benchesDir = path.join(repoRoot, root, pkg, "benches");
      if (!fs.existsSync(benchesDir)) continue;
      for (const name of fs.readdirSync(benchesDir)) {
        if (!name.endsWith(".rs")) continue;
        const file = path.join(root, pkg, "benches", name);
        const text = fs.readFileSync(path.join(repoRoot, file), "utf8");
        if (text.includes("bench_with_metrics")) sources.push({ file, text });
      }
    }
  }
  assert.ok(sources.length > 0, "no davinci bench sources found (bench_with_metrics users)");
  return sources;
}

function argDomain(arg: string, source: { file: string; text: string }): string[] {
  if (arg === "fixture.name") return ladderNames();
  if (arg === "preset_name") {
    const start = source.text.indexOf("let presets = [");
    assert.ok(start >= 0, `${source.file}: expected a \`let presets = [\` array for preset_name`);
    const block = source.text.slice(start, source.text.indexOf("];", start));
    const names = [...block.matchAll(/\(\s*"([^"]+)"/g)].map(([, name]) => name);
    assert.ok(names.length > 0, `${source.file}: presets array names no presets`);
    return names;
  }
  assert.fail(
    `${source.file}: cstr! argument \`${arg}\` is not understood by the bench-id ` +
      "reconciler — extend tests/tooling/davinci-budgets.test.ts",
  );
}

function sourceBenchIds(): string[] {
  const ids: string[] = [];
  for (const source of benchSources()) {
    for (const [, literal] of source.text.matchAll(
      /bench_with_metrics\(\s*criterion,\s*"([^"]+)"/g,
    )) {
      ids.push(literal);
    }
    for (const [, template, argsRaw] of source.text.matchAll(
      /cstr!\(\s*"([^"]+)"((?:\s*,[^)]*)?)\)/g,
    )) {
      const args = argsRaw
        .split(",")
        .map((arg) => arg.trim())
        .filter((arg) => arg.length > 0);
      const placeholders = template.split("{}").length - 1;
      assert.equal(
        placeholders,
        args.length,
        `${source.file}: cstr!("${template}") has ${placeholders} placeholders but ${args.length} arguments`,
      );
      let variants = [template];
      for (const arg of args) {
        const domain = argDomain(arg, source);
        variants = variants.flatMap((variant) =>
          domain.map((value) => variant.replace("{}", value)),
        );
      }
      ids.push(...variants);
    }
  }
  assert.deepEqual([...new Set(ids)], ids, "bench sources must not construct duplicate ids");
  return ids.sort();
}

test("every davinci bench id has a budgets.toml entry, and vice versa", () => {
  const sourceIds = sourceBenchIds();
  const budgetIds = Object.keys(budgets.bench).sort();
  const missingFromBudgets = sourceIds.filter((id) => !budgetIds.includes(id));
  const withoutBenchSource = budgetIds.filter((id) => !sourceIds.includes(id));
  assert.deepEqual(
    { missingFromBudgets, withoutBenchSource },
    { missingFromBudgets: [], withoutBenchSource: [] },
    "budgets.toml [bench] and the bench sources must reconcile exactly",
  );
  assert.deepEqual(budgetIds, sourceIds);
});

test("every budget entry carries exactly the documented fields within the tolerance ceilings", () => {
  const entries = Object.entries(budgets.bench);
  assert.ok(entries.length > 0, "budgets.toml must register at least one bench");
  for (const [id, entry] of entries) {
    assert.deepEqual(
      Object.keys(entry).sort(),
      ["allocs", "rss_peak_bytes", "wall_p50_ns", "wall_tolerance"],
      `[bench.${id}] must have exactly the documented field set`,
    );
    for (const field of ["wall_p50_ns", "allocs", "rss_peak_bytes"]) {
      const value = entry[field];
      assert.ok(
        Number.isSafeInteger(value) && (value as number) >= 0,
        `[bench.${id}] ${field} must be a non-negative integer, got ${String(value)}`,
      );
    }
    const tolerance = entry.wall_tolerance;
    assert.ok(typeof tolerance === "number", `[bench.${id}] wall_tolerance must be a number`);
    assert.ok(tolerance > 0, `[bench.${id}] wall_tolerance must be positive`);
    const ceiling = id.includes("_transform_") ? 0.1 : 0.05;
    assert.ok(
      tolerance <= ceiling,
      `[bench.${id}] wall_tolerance ${tolerance} exceeds the ${ceiling} ceiling ` +
        "(ratchet: tolerances only tighten; stage windows get 0.10, whole routines 0.05)",
    );
  }
});

test("budgets.toml carries the exact ratchet header line", () => {
  const ratchetLine =
    "# ratchet: numbers may only tighten; loosening requires " +
    "budget-loosen: <charter ref> in the commit body";
  assert.ok(
    budgetsText.split("\n").includes(ratchetLine),
    `budgets.toml must contain the exact line:\n${ratchetLine}`,
  );
});

// --- bench-compare gate over the committed fixture pairs -----------------

function runCompare(
  args: string[],
  options: { cwd?: string; refreshEnv?: string } = {},
): ReturnType<typeof spawnSync<string>> {
  const env = { ...process.env };
  delete env.DAVINCI_BASELINE_REFRESH;
  if (options.refreshEnv != null) env.DAVINCI_BASELINE_REFRESH = options.refreshEnv;
  return spawnSync(process.execPath, [comparePath, ...args], {
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
      "bench-compare: breaches=0 gated_ok=2 report_only=0 registered=2\n",
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
      "bench-compare: breaches=1 gated_ok=1 report_only=0 registered=2\n",
  );
});

test("bench-compare exits 1 on any allocation-count change (exact gate)", () => {
  const result = runCompare(compareArgs("allocs-breach"));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 1, result.stdout);
  assert.equal(
    result.stdout,
    header("allocs-breach") +
      "FAIL fixture_parse_small allocs 42 -> 43 (exact gate: allocs are deterministic)\n" +
      "ok fixture_transform_medium wall_p50 216000ns (baseline 200000ns limit 220000ns) " +
      "allocs 1000 rss 16384B\n" +
      "bench-compare: breaches=1 gated_ok=1 report_only=0 registered=2\n",
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
      "bench-compare: breaches=2 gated_ok=1 report_only=0 registered=2\n",
  );
});

test("bench-compare lists missing-baseline benches as unbaselined and report-only", () => {
  const overrides = { baseline: `${fixtureRel}/no-baseline` };
  const result = runCompare(compareArgs("within-tolerance", overrides));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 0, result.stdout);
  assert.equal(
    result.stdout,
    header("within-tolerance", overrides) +
      "unbaselined fixture_parse_small wall_p50 104000ns allocs 42 rss 8192B (report-only)\n" +
      "unbaselined fixture_transform_medium wall_p50 216000ns allocs 1000 rss 16384B " +
      "(report-only)\n" +
      "bench-compare: breaches=0 gated_ok=0 report_only=2 registered=2\n",
  );
});

test("bench-compare treats all-zero (unrecorded) budget entries as report-only", () => {
  const overrides = { budgets: `${fixtureRel}/budgets-unrecorded.toml` };
  const result = runCompare(compareArgs("within-tolerance", overrides));
  assert.equal(result.stderr, "");
  assert.equal(result.status, 0, result.stdout);
  assert.equal(
    result.stdout,
    header("within-tolerance", overrides) +
      "unrecorded fixture_parse_small wall_p50 104000ns allocs 42 rss 8192B " +
      "(report-only: budgets.toml baseline not yet recorded)\n" +
      "unrecorded fixture_transform_medium wall_p50 216000ns allocs 1000 rss 16384B " +
      "(report-only: budgets.toml baseline not yet recorded)\n" +
      "bench-compare: breaches=0 gated_ok=0 report_only=2 registered=2\n",
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

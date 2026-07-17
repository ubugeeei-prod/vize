import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  CRITERION_SUITES,
  renderSummary,
  resolveSuiteSelection,
} from "../../bench/criterion-ab.mjs";
import {
  compareBaselineExports,
  critcmpArgs,
  critcmpExportArgs,
  parseCritcmpExport,
  validateComparisonTable,
} from "../../bench/criterion-baselines.mjs";
import {
  changedPathsBetween,
  parseNameStatusZ,
  selectCriterionSuites,
} from "../../bench/criterion-impact.mjs";

const repoDir = "/repo";
const suiteNames = CRITERION_SUITES.map((suite) => suite.package);

function git(cwd: string, args: string[]): string {
  const result = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, `${result.stderr}\n${result.stdout}`.trim());
  return result.stdout.trim();
}

function commit(cwd: string, message: string): string {
  git(cwd, ["add", "."]);
  git(cwd, ["-c", "user.name=Vize", "-c", "user.email=vize@example.com", "commit", "-qm", message]);
  return git(cwd, ["rev-parse", "HEAD"]);
}

function metadata(dependencies: Record<string, string[]> = {}) {
  const names = ["vize", "vize_atelier_core", ...suiteNames];
  const packages = names.map((name) => ({
    id: `${name}@0.1.0`,
    name,
    manifest_path: `${repoDir}/crates/${name}/Cargo.toml`,
  }));
  return {
    packages,
    workspace_members: packages.map((pkg) => pkg.id),
    resolve: {
      nodes: packages.map((pkg) => ({
        id: pkg.id,
        dependencies: (dependencies[pkg.name] ?? []).map((name) => `${name}@0.1.0`),
      })),
    },
  };
}

test("Criterion impact parser preserves both sides of renamed Rust paths", () => {
  assert.deepEqual(
    parseNameStatusZ(
      "M\0crates/vize/src/main.rs\0R100\0crates/old/src/lib.rs\0crates/new/src/lib.rs\0",
    ),
    ["crates/vize/src/main.rs", "crates/old/src/lib.rs", "crates/new/src/lib.rs"],
  );
});

test("Criterion impact diff excludes base-only changes after the feature fork", () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-criterion-impact-"));
  fs.mkdirSync(path.join(root, "crates", "vize"), { recursive: true });
  git(root, ["init", "-q", "--initial-branch=main"]);
  fs.writeFileSync(path.join(root, "README.md"), "base\n");
  commit(root, "base");

  git(root, ["switch", "-qc", "feature"]);
  fs.writeFileSync(path.join(root, "crates", "vize", "feature.rs"), "feature\n");
  const headSha = commit(root, "feature");

  git(root, ["switch", "-q", "main"]);
  fs.writeFileSync(path.join(root, "Cargo.lock"), "base advanced\n");
  const baseSha = commit(root, "advance base");

  assert.deepEqual(changedPathsBetween(root, baseSha, headSha), ["crates/vize/feature.rs"]);
});

test("direct Criterion package changes select only their suite", () => {
  const result = selectCriterionSuites({
    changedPaths: ["crates/vize_glyph/src/lib.rs"],
    metadata: metadata(),
    repoDir,
  });

  assert.equal(result.mode, "scoped");
  assert.deepEqual(result.selected, ["vize_glyph"]);
});

test("reverse dependency impact selects every suite that consumes a changed package", () => {
  const dependencies = Object.fromEntries(suiteNames.map((name) => [name, ["vize_atelier_core"]]));
  const result = selectCriterionSuites({
    changedPaths: ["crates/vize_atelier_core/src/lib.rs"],
    metadata: metadata(dependencies),
    repoDir,
  });

  assert.deepEqual(result.selected, suiteNames);
  assert.deepEqual(result.skipped, []);
});

test("CLI-only changes skip unrelated Criterion suites", () => {
  const result = selectCriterionSuites({
    changedPaths: ["crates/vize/src/build/input.rs"],
    metadata: metadata(),
    repoDir,
  });

  assert.equal(result.mode, "scoped");
  assert.deepEqual(result.selected, []);
  assert.deepEqual(result.skipped, suiteNames);
  assert.match(result.reason, /vize/);
});

test("lockfiles and shared benchmark infrastructure select the full inventory", () => {
  for (const changedPath of ["Cargo.lock", "bench/criterion-ab.mjs"]) {
    const result = selectCriterionSuites({
      changedPaths: [changedPath],
      metadata: metadata(),
      repoDir,
    });
    assert.equal(result.mode, "full", changedPath);
    assert.deepEqual(result.selected, suiteNames, changedPath);
  }
});

test("unowned foundational Rust paths fail safe to the full inventory", () => {
  const result = selectCriterionSuites({
    changedPaths: ["crates/foundation/config.rs"],
    metadata: metadata(),
    repoDir,
  });

  assert.equal(result.mode, "full");
  assert.deepEqual(result.selected, suiteNames);
  assert.match(result.reason, /not owned/);
});

test("incomplete workspace metadata fails closed", () => {
  const incomplete = metadata();
  incomplete.packages = incomplete.packages.filter((pkg) => pkg.name !== "vize_glyph");
  incomplete.workspace_members = incomplete.packages.map((pkg) => pkg.id);

  assert.throws(
    () =>
      selectCriterionSuites({
        changedPaths: ["crates/vize/src/main.rs"],
        metadata: incomplete,
        repoDir,
      }),
    /Criterion package\(s\) missing/,
  );
});

test("Criterion driver validates scoped suite manifests", () => {
  const selection = resolveSuiteSelection({
    selected: ["vize_glyph", "vize_atelier_sfc"],
    reason: "fixture",
  });
  assert.deepEqual(selection.selected, ["vize_atelier_sfc", "vize_glyph"]);
  assert.throws(
    () => resolveSuiteSelection({ selected: ["unknown"], reason: "fixture" }),
    /unknown suites/,
  );
});

test("Criterion driver snapshots both baselines before comparing them", () => {
  assert.deepEqual(critcmpExportArgs({ targetDir: "/work/head/target", baseline: "base" }), [
    "--target-dir",
    "/work/head/target",
    "--export",
    "base",
  ]);
  assert.deepEqual(
    critcmpArgs({
      targetDir: "/work/head/target",
      baselinePaths: ["/work/base.json", "/work/head.json"],
    }),
    ["--target-dir", "/work/head/target", "/work/base.json", "/work/head.json"],
  );
});

test("Criterion baseline comparison fails closed and uses exported medians", () => {
  const base = parseCritcmpExport(baselineExport("base", { shared: 100 }), "base");
  const head = parseCritcmpExport(baselineExport("head", { shared: 125 }), "head");

  assert.deepEqual(compareBaselineExports(base, head, 10), [{ name: "shared", changePercent: 25 }]);
  assert.throws(
    () => parseCritcmpExport(baselineExport("base", {}), "base"),
    /contains no benchmarks/,
  );
  assert.throws(
    () =>
      compareBaselineExports(
        base,
        parseCritcmpExport(baselineExport("head", { other: 90 }), "head"),
        10,
      ),
    /no shared benchmarks/,
  );
});

test("Criterion comparison table requires both base and head columns", () => {
  assert.doesNotThrow(() =>
    validateComparisonTable("group  base  head\n-----  ----  ----\nshared  1.00  1.12\n"),
  );
  assert.throws(
    () => validateComparisonTable("group  head\n-----  ----\nshared  1.00\n"),
    /did not produce base\/head columns/,
  );
});

test("Criterion driver reports a useful summary when no suite is affected", () => {
  const selection = resolveSuiteSelection({ selected: [], reason: "CLI-only change." });
  const summary = renderSummary({ table: "", threshold: undefined, regressions: [], selection });

  assert.match(summary, /Selection: CLI-only change\./);
  assert.match(summary, /Ran: none/);
  assert.match(summary, /Skipped: vize_atelier_sfc, vize_atelier_jsx/);
  assert.match(summary, /timing execution was skipped/);
});

function baselineExport(name: string, medians: Record<string, number>): string {
  return JSON.stringify({
    name,
    benchmarks: Object.fromEntries(
      Object.entries(medians).map(([benchmark, pointEstimate]) => [
        benchmark,
        { criterion_estimates_v1: { median: { point_estimate: pointEstimate } } },
      ]),
    ),
  });
}

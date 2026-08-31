import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { CRITERION_SUITES, resolveSuiteSelection } from "../../tools/benchmarks/scripts/criterion-ab.mjs";
import {
  changedPathsBetween,
  parseNameStatusZ,
  selectCriterionSuites,
} from "../../tools/benchmarks/scripts/criterion-impact.mjs";

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
    manifest_path:
      name === "vize_benchmarks"
        ? `${repoDir}/tools/benchmarks/crates/vize/Cargo.toml`
        : `${repoDir}/crates/${name}/Cargo.toml`,
  }));
  const effectiveDependencies = { vize_benchmarks: ["vize"], ...dependencies };
  return {
    packages,
    workspace_members: packages.map((pkg) => pkg.id),
    resolve: {
      nodes: packages.map((pkg) => ({
        id: pkg.id,
        dependencies: (effectiveDependencies[pkg.name] ?? []).map((name) => `${name}@0.1.0`),
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

test("Doctor reporter benchmarks are enrolled in scoped Criterion A/B runs", () => {
  const suite = CRITERION_SUITES.find(({ package: packageName }) => packageName === "vize_doctor");
  assert.deepEqual(suite, {
    package: "vize_doctor",
    benches: ["reporter"],
    label: "Doctor reporters",
  });

  const result = selectCriterionSuites({
    changedPaths: ["crates/vize_doctor/src/reporter/json.rs"],
    metadata: metadata(),
    repoDir,
  });

  assert.equal(result.mode, "scoped");
  assert.deepEqual(result.selected, ["vize_doctor"]);
});

test("Doctor TUI benchmarks carry explicit reference-runner latency budgets", () => {
  const suite = CRITERION_SUITES.find(
    ({ package: packageName }) => packageName === "vize_benchmarks",
  );
  assert.deepEqual(suite, {
    package: "vize_benchmarks",
    benches: ["doctor_tui"],
    label: "Doctor TUI",
    absoluteBudgets: [
      { name: "doctor_tui_10k/first_frame_120x40", maxMedianNs: 20_000_000 },
      { name: "doctor_tui_input_to_frame_10k/selection", maxMedianNs: 1_000_000 },
      { name: "doctor_tui_input_to_frame_10k/search", maxMedianNs: 1_000_000 },
    ],
  });

  const direct = selectCriterionSuites({
    changedPaths: ["tools/benchmarks/crates/vize/doctor_tui.rs"],
    metadata: metadata(),
    repoDir,
  });
  assert.deepEqual(direct.selected, ["vize_benchmarks"]);

  const dependency = selectCriterionSuites({
    changedPaths: ["crates/vize/src/commands/doctor/tui.rs"],
    metadata: metadata(),
    repoDir,
  });
  assert.deepEqual(dependency.selected, ["vize_benchmarks"]);
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

test("non-Rust CLI fixture changes skip unrelated Criterion suites", () => {
  const result = selectCriterionSuites({
    changedPaths: ["docs/cli.md"],
    metadata: metadata(),
    repoDir,
  });

  assert.equal(result.mode, "scoped");
  assert.deepEqual(result.selected, []);
  assert.deepEqual(result.skipped, suiteNames);
  assert.match(result.reason, /none/);
});

test("lockfiles and shared benchmark infrastructure select the full inventory", () => {
  for (const changedPath of [
    "Cargo.lock",
    "tools/benchmarks/scripts/criterion-ab.mjs",
    "tools/benchmarks/scripts/criterion-baselines.mjs",
    "tools/benchmarks/scripts/criterion-summary.mjs",
  ]) {
    const result = selectCriterionSuites({
      changedPaths: [changedPath],
      metadata: metadata(),
      repoDir,
    });
    assert.equal(result.mode, "full", changedPath);
    assert.deepEqual(result.selected, suiteNames, changedPath);
  }
});

test("hosted fallback smoke-runs Criterion infrastructure-only changes", () => {
  const result = selectCriterionSuites({
    changedPaths: [
      ".github/workflows/check.yml",
      ".github/workflows/criterion-bench.yml",
      "tools/benchmarks/scripts/criterion-ab.mjs",
      "crates/vize_canon/tests/tier_l_incremental.rs",
      "tests/tooling/criterion-baselines.test.ts",
    ],
    metadata: metadata(),
    repoDir,
    hostedFallback: true,
  });

  assert.equal(result.mode, "hosted-smoke");
  assert.deepEqual(result.selected, ["vize_glyph"]);
  assert.deepEqual(
    result.skipped,
    suiteNames.filter((suite) => suite !== "vize_glyph"),
  );
  assert.match(result.reason, /Criterion infrastructure changed without Rust benchmark subjects/);
});

test("hosted fallback preserves full Criterion coverage for Rust benchmark subjects", () => {
  const result = selectCriterionSuites({
    changedPaths: ["Cargo.lock", "tools/benchmarks/scripts/criterion-ab.mjs"],
    metadata: metadata(),
    repoDir,
    hostedFallback: true,
  });

  assert.equal(result.mode, "full");
  assert.deepEqual(result.selected, suiteNames);
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

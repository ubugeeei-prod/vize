import assert from "node:assert/strict";
import { test } from "node:test";

import {
  CRITERION_SUITES,
  renderSummary,
  resolveSuiteSelection,
} from "../../bench/criterion-ab.mjs";
import { parseNameStatusZ, selectCriterionSuites } from "../../bench/criterion-impact.mjs";

const repoDir = "/repo";
const suiteNames = CRITERION_SUITES.map((suite) => suite.package);

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

test("Criterion driver reports a useful summary when no suite is affected", () => {
  const selection = resolveSuiteSelection({ selected: [], reason: "CLI-only change." });
  const summary = renderSummary({ table: "", threshold: undefined, regressions: [], selection });

  assert.match(summary, /Selection: CLI-only change\./);
  assert.match(summary, /Ran: none/);
  assert.match(summary, /Skipped: vize_atelier_sfc, vize_atelier_jsx/);
  assert.match(summary, /timing execution was skipped/);
});

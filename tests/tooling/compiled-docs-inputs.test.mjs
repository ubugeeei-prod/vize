import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { planSourceChecks } from "../../tools/support/compat/github/plan-source-checks.mjs";
import { planAffectedRust } from "../../tools/support/compat/github/plan-affected-rust.mjs";

const root = fileURLToPath(new URL("../../", import.meta.url));
const packages = ["compiler", "unrelated"].map((name) => ({
  id: name,
  name,
  manifest_path: `${root}crates/${name}/Cargo.toml`,
  dependencies: [],
}));
const metadata = {
  packages,
  workspace_members: packages.map((pkg) => pkg.id),
  workspace_root: root,
};

void test("real compiled Markdown and budget witnesses select both Rust planners", () => {
  const witnesses = [
    [
      "crates/vize_l1_to_l2/src/pass/cfg/source.rs",
      "../../../../../docs/davinci/plan/complexity-metrics.md",
    ],
    ["crates/vize_davinci/tests/key_manifests.rs", "../../../docs/davinci/plan/key-manifests.md"],
    [
      "crates/vize_davinci/tests/fact_alpha/main.rs",
      "../../../../docs/davinci/plan/fact-alpha-schemas.md",
    ],
    ["crates/vize_resident/src/summary.rs", "../../../docs/davinci/plan/budgets.toml"],
  ];
  for (const [source, literal] of witnesses) {
    assert.ok(readFileSync(resolve(root, source), "utf8").includes(`include_str!("${literal}")`));
    const absolute = resolve(root, source, "..", literal);
    assert.ok(existsSync(absolute), absolute);
    const path = absolute.slice(root.length);
    assert.deepEqual(planSourceChecks([path]), {
      rust: true,
      js: true,
      tooling: true,
      playground: true,
    });
    const plan = planAffectedRust(metadata, [path]);
    assert.equal(plan.scope, "workspace", path);
    assert.deepEqual(plan.packages, ["compiler", "unrelated"], path);
    assert.ok(plan.reasons.includes(`shared or unknown input: ${path}`));
  }
});

void test("plan contracts remain conservative across renames while ordinary prose stays exempt", () => {
  for (const path of [
    "docs/davinci/plan/new-contract.mdx",
    "docs/davinci/plan/remarks.schema.json",
    "docs/davinci/plan/reach-budgets.toml",
  ]) {
    assert.equal(planSourceChecks([path]).rust, true, path);
    assert.equal(planSourceChecks([path]).playground, true, path);
    assert.equal(planAffectedRust(metadata, [path]).scope, "workspace", path);
  }
  for (const path of ["docs/guide/example.md", "docs/davinci/decisions/change.md"]) {
    assert.equal(planSourceChecks([path]).rust, false, path);
    assert.equal(planAffectedRust(metadata, [path]).scope, "none", path);
  }
});

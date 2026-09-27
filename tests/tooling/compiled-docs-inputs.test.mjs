import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  changedPaths,
  planSourceChecks,
} from "../../tools/support/compat/github/plan-source-checks.mjs";
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

void test("mutating the compiled CLI schema selects Rust while unrelated package schemas stay narrow", () => {
  const path = "npm/cli/schemas/vize.config.schema.json";
  const source = readFileSync(resolve(root, "crates/vize/src/config.rs"), "utf8");
  assert.ok(source.includes(`include_str!("../../../${path}")`));
  assert.ok(source.includes("fs::write(&schema_path, VIZE_CONFIG_SCHEMA)"));
  const cwd = mkdtempSync(join(tmpdir(), "vize-compiled-schema-"));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    mkdirSync(dirname(join(cwd, path)), { recursive: true });
    writeFileSync(join(cwd, path), '{"type":"object"}\n');
    git("add", ".");
    git("commit", "-qm", "initial schema");
    const base = git("rev-parse", "HEAD");
    writeFileSync(join(cwd, path), '{"type":"object","required":["compiler"]}\n');
    git("commit", "-qam", "mutated schema");
    const paths = changedPaths(base, git("rev-parse", "HEAD"), cwd);
    assert.deepEqual(paths, [path]);
    assert.deepEqual(planSourceChecks(paths), {
      rust: true,
      js: true,
      tooling: true,
      playground: true,
    });
    assert.equal(planAffectedRust(metadata, paths).scope, "workspace");
    assert.deepEqual(planAffectedRust(metadata, paths).packages, ["compiler", "unrelated"]);
    assert.equal(planSourceChecks(["npm/cli/schemas/unrelated.schema.json"]).rust, false);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

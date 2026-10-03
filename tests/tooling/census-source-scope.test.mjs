import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { planSourceChecks } from "../../tools/support/compat/github/plan-source-checks.mjs";
import { planAffectedRust } from "../../tools/support/compat/github/plan-affected-rust.mjs";
import { fixture } from "./_helpers/rust-workspace-fixture.mjs";

void test("known census renderers and owned reports retain tooling without product source lanes", () => {
  const paths = [
    "tools/support/compat/davinci/croquis-consumers.mjs",
    "tools/support/compat/davinci/lib/croquis-render.mjs",
    "tools/support/compat/davinci/lib/croquis-shards.mjs",
    "docs/davinci/plan/croquis-consumption.md",
    "docs/davinci/plan/croquis-consumption/vize.md",
    "docs/davinci/plan/croquis-consumption/vize_l0_derive.md",
    "docs/davinci/plan/croquis-consumption/vize_atelier_sfc.md",
  ];
  for (const path of paths) {
    assert.deepEqual(
      planSourceChecks([path]),
      { rust: false, js: false, tooling: true, playground: false },
      path,
    );
  }
  assert.deepEqual(planSourceChecks(paths, "merge_group"), {
    rust: true,
    js: true,
    tooling: true,
    playground: true,
  });
});

void test("mixed census changes retain every native, workflow and shared configuration lane", () => {
  for (const path of [
    "davinci/vize_l1/src/lib.rs",
    "crates/vize_atelier_sfc/src/native/setup.rs",
    "tests/expected/native/setup.snap",
    "tests/tooling/native-sfc-js-setup-reference.test.ts",
    ".github/actions/test-native-js-setup/action.yml",
    ".github/workflows/pr-source-checks.yml",
    "Cargo.lock",
    ".cargo/config.toml",
    "npm/cli/schemas/vize.config.schema.json",
    "docs/davinci/plan/key-manifests.md",
    "docs/davinci/plan/budgets.toml",
  ]) {
    assert.deepEqual(
      planSourceChecks(["docs/davinci/plan/croquis-consumption/vize_l1.md", path]),
      { ...planSourceChecks([path]), tooling: true },
      path,
    );
  }
});

void test("unknown census helpers and authored contracts keep the full conservative source fallback", () => {
  for (const path of [
    "tools/support/compat/davinci/lib/croquis-analysis.mjs",
    "tools/support/compat/davinci/lib/new-census-helper.mjs",
    "tools/support/compat/davinci/croquis-consumers.rs",
    "docs/davinci/plan/croquis-consumption.schema.json",
    "docs/davinci/plan/croquis-consumption/contract.md",
    "docs/davinci/plan/croquis-consumption/vize_l1.json",
    "docs/davinci/plan/croquis-consumption/vize_l1.rs.md",
    "docs/davinci/plan/croquis-consumption/nested/vize_l1.md",
    "docs/davinci/plan/croquis-consumption/../key-manifests.md",
  ]) {
    assert.deepEqual(
      planSourceChecks([path]),
      { rust: true, js: true, tooling: true, playground: true },
      path,
    );
  }
});

void test("named census renderers and reports remain outside workspace Rust source inputs", () => {
  const root = fileURLToPath(new URL("../../", import.meta.url));
  const workspace = readFileSync(join(root, "Cargo.toml"), "utf8")
    .split(/^\[/m)
    .find((section) => /^workspace\]\r?\n/.test(section));
  assert.ok(workspace, "workspace manifest section must be available");
  const members = /^members\s*=\s*\[([^\]]*)\]/m.exec(workspace);
  assert.ok(members, "workspace member list must be available");
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) visit(path);
      else if (entry.isFile() && entry.name.endsWith(".rs")) {
        assert.doesNotMatch(
          readFileSync(path, "utf8"),
          /croquis-(?:consumption|consumers|render|shards)/,
          path,
        );
      }
    }
  };
  const paths = members[1]
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  assert.ok(paths.length, "workspace member list must not be empty");
  for (const path of paths) {
    const member = /^"((?:[A-Za-z0-9_-]+\/)*[A-Za-z0-9_-]+)",?$/.exec(path);
    assert.ok(member, `workspace member syntax requires source-scope review: ${path}`);
    visit(join(root, member[1]));
  }
});

void test("census-only reports and exact Node renderers select no workspace Rust packages", () => {
  const plan = planAffectedRust(fixture(), [
    "docs/davinci/plan/croquis-consumption.md",
    "docs/davinci/plan/croquis-consumption/vize_atelier_sfc.md",
    "tools/support/compat/davinci/croquis-consumers.mjs",
    "tools/support/compat/davinci/lib/croquis-render.mjs",
    "tools/support/compat/davinci/lib/croquis-shards.mjs",
  ]);
  assert.equal(plan.scope, "none");
  assert.deepEqual(plan.packages, []);
  assert.deepEqual(plan.reasons, []);
});

void test("mixed census and Rust changes preserve every transitive actual consumer", () => {
  const path = "crates/syntax/src/lib.rs";
  assert.deepEqual(
    planAffectedRust(fixture(), [
      path,
      "docs/davinci/plan/croquis-consumption/vize_l1.md",
      "tools/support/compat/davinci/lib/croquis-render.mjs",
    ]),
    planAffectedRust(fixture(), [path]),
  );
});

void test("census exemptions never narrow merge groups, workflow changes or unknown contracts", () => {
  const metadata = fixture();
  const all = planAffectedRust(metadata, ["Cargo.lock"]);
  const census = "docs/davinci/plan/croquis-consumption/vize_l1.md";
  assert.deepEqual(planAffectedRust(metadata, [census], "merge_group").packages, all.packages);
  for (const path of [
    ".github/workflows/pr-source-checks.yml",
    ".github/actions/test-native-js-setup/action.yml",
    "Cargo.lock",
    ".cargo/config.toml",
    "npm/cli/schemas/vize.config.schema.json",
    "docs/davinci/plan/key-manifests.md",
    "docs/davinci/plan/croquis-consumption/contract.md",
    "docs/davinci/plan/croquis-consumption/vize_l1.json",
    "tools/support/compat/davinci/lib/new-census-helper.mjs",
  ]) {
    const plan = planAffectedRust(metadata, [census, path]);
    assert.equal(plan.scope, "workspace", path);
    assert.deepEqual(plan.packages, all.packages, path);
    assert.ok(plan.reasons.length, path);
  }
});

import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  mkdtempSync,
  rmSync,
  writeFileSync,
  unlinkSync,
  mkdirSync,
  renameSync,
  readFileSync,
  readdirSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  changedPaths,
  planSourceChecks,
} from "../../tools/support/compat/github/plan-source-checks.mjs";

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

void test("compiler and CI changes run both source gates", () => {
  for (const path of [
    "crates/vize_croquis_cf/src/rules/provide_inject/index.rs",
    ".github/workflows/check.yml",
    "Cargo.lock",
  ]) {
    assert.deepEqual(
      planSourceChecks([path]),
      {
        rust: true,
        js: true,
        tooling: path === ".github/workflows/check.yml",
        playground: path !== "crates/vize_croquis_cf/src/rules/provide_inject/index.rs",
      },
      path,
    );
  }
});

void test("package changes run package tests while documentation stays fast", () => {
  assert.deepEqual(planSourceChecks(["npm/builder/rspack/src/scoped-css.test.ts"]), {
    rust: false,
    js: true,
    tooling: false,
    playground: false,
  });
  assert.deepEqual(planSourceChecks(["docs/guide/example.md", "README.md", "AGENTS.md"]), {
    rust: false,
    js: false,
    tooling: false,
    playground: false,
  });
});

void test("merge groups run every source suite regardless of the changed paths", () => {
  for (const paths of [
    [],
    ["docs/guide/example.md"],
    ["npm/ui/src/index.ts"],
    ["davinci/vize_l1/src/lib.rs"],
  ]) {
    assert.deepEqual(planSourceChecks(paths, "merge_group"), {
      rust: true,
      js: true,
      tooling: true,
      playground: true,
    });
  }
  assert.deepEqual(planSourceChecks(["README.md"], "pull_request"), {
    rust: false,
    js: false,
    tooling: false,
    playground: false,
  });
});

void test("unknown planning contexts fail rather than selecting a partial suite", () => {
  for (const event of ["push", "workflow_dispatch", "", null]) {
    assert.throws(() => planSourceChecks(["README.md"], event), /planning context/);
  }
});

void test("the CLI applies merge-group scope to a real docs-only comparison", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-queue-plan-"));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  const script = fileURLToPath(
    new URL("../../tools/support/compat/github/plan-source-checks.mjs", import.meta.url),
  );
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    writeFileSync(join(cwd, "README.md"), "before\n");
    git("add", ".");
    git("commit", "-qm", "add docs");
    const base = git("rev-parse", "HEAD");
    writeFileSync(join(cwd, "README.md"), "after\n");
    git("commit", "-qam", "update docs");
    const head = git("rev-parse", "HEAD");
    for (const [context, expected] of [
      [[], false],
      [["merge_group"], true],
    ]) {
      const output = join(cwd, `output-${String(expected)}`);
      const run = spawnSync(process.execPath, [script, base, head, ...context], {
        cwd,
        encoding: "utf8",
        env: { ...process.env, GITHUB_OUTPUT: output, GITHUB_STEP_SUMMARY: "" },
      });
      assert.equal(run.status, 0, run.stderr);
      assert.equal(
        readFileSync(output, "utf8"),
        ["rust", "js", "tooling", "playground"]
          .map((lane) => `${lane}=${String(expected)}\n`)
          .join(""),
      );
    }
    const invalid = spawnSync(process.execPath, [script, base, head, "push"], {
      cwd,
      encoding: "utf8",
    });
    assert.notEqual(invalid.status, 0);
    assert.match(invalid.stderr, /invalid source planning context/);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("unknown source directories fail closed", () => {
  assert.deepEqual(planSourceChecks(["new-runtime/src/index.ts"]), {
    rust: true,
    js: true,
    tooling: true,
    playground: true,
  });
});

void test("release sources and tooling tests require the script gate", () => {
  for (const path of [
    "tools/support/release/pr_watch.rs",
    "tools/commands/release/promote.rs",
    "tests/tooling/release/release-pr.test.ts",
    ".github/workflows/release.yml",
  ]) {
    assert.deepEqual(
      planSourceChecks([path]),
      { rust: false, js: false, tooling: true, playground: false },
      path,
    );
  }
});

void test("coverage goldens run the Rust fixture gate", () => {
  assert.deepEqual(planSourceChecks(["tests/expected/vapor/element.snap"]), {
    rust: true,
    js: false,
    tooling: false,
    playground: false,
  });
});

void test("compiler, Vite, and playground changes run browser snapshots", () => {
  for (const path of [
    "crates/vize_atelier_vapor/src/generate.rs",
    "npm/builder/vite/src/index.ts",
    "playground/e2e/sfc-compile.test.ts",
  ]) {
    assert.equal(planSourceChecks([path]).playground, true, path);
  }
});

void test("every canonical level and conversion crate retains browser validation after renames", () => {
  for (const crate of [
    "vize_l0",
    "vize_l0_derive",
    "vize_l1",
    "vize_l2",
    "vize_l3",
    "vize_l4",
    "vize_l1_to_l2",
    "vize_l2_to_l3",
    "vize_l4_ssr",
  ]) {
    const path = `crates/${crate}/src/lib.rs`;
    assert.deepEqual(
      planSourceChecks([path]),
      { rust: true, js: true, tooling: false, playground: true },
      path,
    );
  }
});

void test("deleted and moved source files still select both gates", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-source-checks-"));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    mkdirSync(join(cwd, "crates"));
    writeFileSync(join(cwd, "crates", "lib.rs"), "pub fn example() {}\n");
    git("add", ".");
    git("commit", "-qm", "add source");
    const base = git("rev-parse", "HEAD");
    unlinkSync(join(cwd, "crates", "lib.rs"));
    git("add", "-u");
    git("commit", "-qm", "remove source");
    const paths = changedPaths(base, git("rev-parse", "HEAD"), cwd);
    assert.deepEqual(paths, ["crates/lib.rs"]);
    assert.deepEqual(planSourceChecks(paths), {
      rust: true,
      js: true,
      tooling: false,
      playground: false,
    });

    writeFileSync(join(cwd, "crates", "lib.rs"), "pub fn example() {}\n");
    git("add", ".");
    git("commit", "-qm", "restore source");
    const beforeMove = git("rev-parse", "HEAD");
    mkdirSync(join(cwd, "docs"));
    renameSync(join(cwd, "crates", "lib.rs"), join(cwd, "docs", "lib.rs"));
    git("add", "-A");
    git("commit", "-qm", "move source");
    const movedPaths = changedPaths(beforeMove, git("rev-parse", "HEAD"), cwd);
    assert.deepEqual(movedPaths, ["crates/lib.rs", "docs/lib.rs"]);
    assert.deepEqual(planSourceChecks(movedPaths), {
      rust: true,
      js: true,
      tooling: true,
      playground: true,
    });
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("only Markdown documentation exemptions can avoid the Rust fallback", () => {
  for (const file of [
    "docs/runtime.mjs",
    "docs/fixture.vue",
    "docs/contracts.json",
    ".changeset/check.ts",
  ]) {
    assert.equal(planSourceChecks([file]).rust, true, file);
  }
  for (const file of ["docs/design.md", "docs/design.mdx", ".changeset/release.md"]) {
    assert.equal(planSourceChecks([file]).rust, false, file);
  }
});

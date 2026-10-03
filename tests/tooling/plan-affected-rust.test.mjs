import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { test } from "node:test";

import { changedPaths } from "../../tools/support/compat/github/plan-source-checks.mjs";
import {
  planAffectedRust,
  readCargoMetadata,
} from "../../tools/support/compat/github/plan-affected-rust.mjs";
import { rustCommand } from "../../tools/support/compat/github/run-affected-rust.mjs";

import { fixture } from "./_helpers/rust-workspace-fixture.mjs";

void test("normal, renamed optional, target-specific build and dev edges retain transitive consumers", () => {
  const metadata = fixture();
  const plan = planAffectedRust(metadata, ["crates/syntax/src/lib.rs"]);
  assert.equal(plan.scope, "affected");
  assert.deepEqual(plan.changedPackages, ["syntax"]);
  assert.deepEqual(plan.packages, ["compiler", "consumer", "syntax", "tests"]);
  assert.equal(plan.packages.includes("unrelated"), false);
  // Cycles through dev dependencies must terminate without losing any consumer.
  metadata.packages[0].dependencies.push({
    path: "/repo/crates/compiler",
    name: "compiler",
    kind: "dev",
  });
  assert.deepEqual(
    planAffectedRust(metadata, ["crates/syntax/src/lib.rs"]).packages,
    plan.packages,
  );
});

void test("package examples, fixtures, manifests and Rust test helpers select their actual owners", () => {
  for (const path of [
    "crates/compiler/examples/demo.rs",
    "crates/compiler/tests/fixture.json",
    "crates/compiler/Cargo.toml",
  ]) {
    assert.deepEqual(planAffectedRust(fixture(), [path]).packages, [
      "compiler",
      "consumer",
      "tests",
    ]);
  }
  assert.deepEqual(planAffectedRust(fixture(), ["tests/shared_support/src/lib.rs"]).packages, [
    "tests",
  ]);
  assert.deepEqual(planAffectedRust(fixture(), ["crates/unrelated/src/lib.rs"]).packages, [
    "unrelated",
  ]);
});

void test("global inputs, shared corpora, deleted crates and unknown paths validate the whole workspace", () => {
  const expected = ["compiler", "consumer", "syntax", "tests", "unrelated"];
  for (const path of [
    "Cargo.toml",
    "Cargo.lock",
    ".cargo/config.toml",
    "rust-toolchain.toml",
    ".github/workflows/check.yml",
    ".config/nextest.toml",
    "tests/_fixtures/template.vue",
    "tests/expected/template.snap",
    "tests/helper.rs",
    "crates/deleted/src/lib.rs",
    "crates/syntax-other/src/lib.rs",
    "new-runtime/source.rs",
    "docs/fixture.rs",
    "docs/davinci/plan/corpus.json",
    "../crates/syntax/src/lib.rs",
    "/repo/crates/syntax/src/lib.rs",
    "crates\\syntax\\lib.rs",
  ]) {
    const plan = planAffectedRust(fixture(), [path]);
    assert.equal(plan.scope, "workspace", path);
    assert.deepEqual(plan.packages, expected, path);
    assert.ok(plan.reasons.length, path);
  }
  assert.deepEqual(planAffectedRust(fixture(), []).packages, expected);
  assert.deepEqual(
    planAffectedRust(fixture(), ["docs/guide.md"], "merge_group").packages,
    expected,
  );
});

void test("documentation is empty while the independent Zed workspace is explicitly recorded", () => {
  const plan = planAffectedRust(fixture(), [
    "docs/design.md",
    ".changeset/note.md",
    "README.md",
    "editors/zed/src/lib.rs",
  ]);
  assert.equal(plan.scope, "none");
  assert.deepEqual(plan.packages, []);
  assert.deepEqual(plan.excludedPaths, ["editors/zed/src/lib.rs"]);
  assert.throws(() => planAffectedRust(fixture(), [], "push"), /context/);
});

void test("missing members, duplicate package names and invalid metadata fail instead of selecting nothing", () => {
  const metadata = fixture();
  assert.throws(() => planAffectedRust({}, ["crates/syntax/lib.rs"]), /metadata/);
  assert.throws(
    () => planAffectedRust({ ...metadata, packages: [] }, []),
    /every workspace member/,
  );
  metadata.packages[1].name = "syntax";
  assert.throws(() => planAffectedRust(metadata, []), /ambiguous/);
  metadata.packages[1].name = "--workspace";
  assert.throws(() => planAffectedRust(metadata, []), /invalid/);
  metadata.packages[1].name = "compiler";
  metadata.packages[1].manifest_path = "/outside/Cargo.toml";
  assert.throws(() => planAffectedRust(metadata, []), /outside/);
});

void test("deleted and moved source paths preserve both sides of Git rename detection", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-rust-paths-"));
  const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  try {
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    mkdirSync(join(cwd, "crates", "syntax"), { recursive: true });
    mkdirSync(join(cwd, "crates", "unrelated"), { recursive: true });
    writeFileSync(join(cwd, "crates", "syntax", "lib.rs"), "pub fn example() {}\n");
    git("add", ".");
    git("commit", "-qm", "source");
    const base = git("rev-parse", "HEAD");
    renameSync(join(cwd, "crates", "syntax", "lib.rs"), join(cwd, "crates", "unrelated", "lib.rs"));
    git("add", "-A");
    git("commit", "-qm", "move");
    const head = git("rev-parse", "HEAD");
    const paths = changedPaths(base, head, cwd);
    assert.deepEqual(paths, ["crates/syntax/lib.rs", "crates/unrelated/lib.rs"]);
    assert.equal(planAffectedRust(fixture(cwd), paths).packages.length, 5);
    unlinkSync(join(cwd, "crates", "unrelated", "lib.rs"));
    git("add", "-u");
    git("commit", "-qm", "delete");
    assert.deepEqual(
      planAffectedRust(fixture(cwd), changedPaths(head, git("rev-parse", "HEAD"), cwd)).packages,
      ["unrelated"],
    );
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("runner inserts argument arrays at the Cargo subcommand without invoking a shell", () => {
  const plan = planAffectedRust(fixture(), ["crates/unrelated/src/lib.rs"]);
  assert.deepEqual(
    rustCommand(plan, [
      "cargo",
      "nextest",
      "archive",
      "@packages@",
      "--archive-file",
      "archive.tar.zst",
    ]),
    ["cargo", "nextest", "archive", "--package", "unrelated", "--archive-file", "archive.tar.zst"],
  );
  assert.deepEqual(rustCommand(plan, ["cargo", "test", "@packages@", "--doc"]), [
    "cargo",
    "test",
    "--package",
    "unrelated",
    "--doc",
  ]);
  assert.throws(
    () => rustCommand({ ...plan, packages: ["$(touch hacked)"] }, ["cargo", "test", "@packages@"]),
    /valid/,
  );
  assert.throws(
    () => rustCommand({ ...plan, cargoArgs: ["--workspace"] }, ["cargo", "test", "@packages@"]),
    /disagree/,
  );
  assert.throws(() => rustCommand(plan, ["cargo", "test"]), /one @packages@/);
  assert.throws(
    () => rustCommand(plan, ["cargo", "test", "@packages@", "@packages@"]),
    /one @packages@/,
  );
  assert.throws(
    () => rustCommand(planAffectedRust(fixture(), ["README.md"]), ["cargo", "test", "@packages@"]),
    /nonempty/,
  );
});

void test("real Cargo metadata supplies the current workspace aliases and reverse dependencies", () => {
  const metadata = readCargoMetadata();
  const current = new Set(metadata.workspace_members);
  const workspace = metadata.packages.filter((pkg) => current.has(pkg.id));
  const all = planAffectedRust(metadata, ["Cargo.lock"]);
  assert.deepEqual(all.packages, workspace.map((pkg) => pkg.name).sort());
  assert.equal(all.packages.includes("vize-zed-extension"), false);
  const leaf = workspace.find((pkg) => pkg.name === "vize_guest");
  assert.ok(leaf);
  const leafPath = leaf.manifest_path.slice(metadata.workspace_root.length + 1);
  const selected = planAffectedRust(metadata, [leafPath]);
  assert.ok(selected.packages.includes("vize_guest"));
  assert.ok(selected.packages.length < all.packages.length);
  const foundation = workspace.find((pkg) => pkg.name === "vize_l0");
  const foundationPath = foundation.manifest_path.slice(metadata.workspace_root.length + 1);
  assert.ok(planAffectedRust(metadata, [foundationPath]).packages.includes("vize_l1"));
});

void test("runner preserves literal arguments and the child process failure status", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-rust-runner-"));
  const script = new URL(
    "../../tools/support/compat/github/run-affected-rust.mjs",
    import.meta.url,
  );
  try {
    const output = join(cwd, "args.json");
    const planPath = join(cwd, "plan.json");
    writeFileSync(
      planPath,
      JSON.stringify(planAffectedRust(fixture(), ["crates/unrelated/src/lib.rs"])),
    );
    writeFileSync(
      join(cwd, "cargo"),
      "#!/usr/bin/env node\nrequire('node:fs').writeFileSync(process.env.ARGS_FILE, JSON.stringify(process.argv.slice(2))); process.exit(37);\n",
      { mode: 0o755 },
    );
    const literal = "archive with spaces $(touch should-not-exist).tar.zst";
    const result = spawnSync(
      process.execPath,
      [
        script.pathname,
        planPath,
        "cargo",
        "nextest",
        "archive",
        "@packages@",
        "--archive-file",
        literal,
      ],
      {
        encoding: "utf8",
        env: {
          ...process.env,
          PATH: `${cwd}:${dirname(process.execPath)}:${process.env.PATH}`,
          ARGS_FILE: output,
        },
      },
    );
    assert.equal(result.status, 37, result.stderr);
    assert.deepEqual(JSON.parse(readFileSync(output, "utf8")), [
      "nextest",
      "archive",
      "--package",
      "unrelated",
      "--archive-file",
      literal,
    ]);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

void test("CLI writes the same JSON plan to disk and Actions outputs and rejects bad identity", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-rust-plan-"));
  const script = new URL(
    "../../tools/support/compat/github/plan-affected-rust.mjs",
    import.meta.url,
  );
  try {
    const output = join(cwd, "plan.json");
    const actions = join(cwd, "actions.txt");
    const result = spawnSync(
      process.execPath,
      [script.pathname, "0".repeat(40), "1".repeat(40), "pull_request", output],
      {
        encoding: "utf8",
        env: {
          ...process.env,
          GITHUB_OUTPUT: actions,
          GITHUB_STEP_SUMMARY: join(cwd, "summary.md"),
        },
      },
    );
    assert.equal(result.status, 0, result.stderr);
    const json = readFileSync(output, "utf8").trim();
    assert.equal(JSON.parse(json).scope, "workspace");
    assert.ok(readFileSync(actions, "utf8").split("\n").includes(`rust-plan=${json}`));
    assert.equal(result.stdout.trim(), json);
    const invalid = spawnSync(
      process.execPath,
      [script.pathname, "invalid", "1".repeat(40), "pull_request", output],
      { encoding: "utf8" },
    );
    assert.notEqual(invalid.status, 0);
    assert.match(invalid.stderr, /expected base SHA/);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

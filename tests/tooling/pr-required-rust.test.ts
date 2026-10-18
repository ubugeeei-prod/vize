import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import type { CargoMetadata } from "../../tools/support/compat/github/plan-affected-rust.ts";
import { planPrChecks, prCheckOutputs } from "../../tools/support/compat/github/plan-pr-checks.ts";

function metadata(root = "/repo"): CargoMetadata {
  const names = ["vize_armature", "vize_curator", "vize_relief", "vize_vitrine", "vize_l4"];
  return {
    workspace_root: root,
    workspace_members: names,
    packages: names.map((name) => ({
      id: name,
      name,
      manifest_path: `${root}/crates/${name}/Cargo.toml`,
      dependencies: ["vize_vitrine", "vize_l4"].includes(name)
        ? [{ path: `${root}/crates/vize_armature` }]
        : [],
    })),
  };
}
const workspace = [
  { name: "@vizejs/native", path: "npm/native", dependencies: [] },
  { name: "@vizejs/cli", path: "npm/cli", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/ui", path: "npm/ui", dependencies: ["@vizejs/native"] },
  { name: "@vizejs/oxlint", path: "npm/oxlint", dependencies: ["@vizejs/native"] },
];
const plan = (paths: string[], eventName = "pull_request") =>
  planPrChecks({ paths, eventName, labels: [], workspace, cargoMetadata: metadata() });
const allPackages = metadata().workspace_members.toSorted();

test("daily source contracts cover the real Rust/docs failures without widening package-only inputs", () => {
  const originalFailures = [
    "crates/vize_atelier_dom/tests/dev_root_fragment.rs",
    "crates/vize_canon/src/batch/virtual_project/content_mapper_config.rs",
    "docs/content/ja/guide/content-mapper.md",
  ];
  for (const event of ["pull_request", "merge_group"]) {
    for (const path of [
      ...originalFailures,
      "crates/vize_canon/src/batch/virtual_project/content_mapper_directive_tests.rs",
      "docs/content/guide/content-mapper.md",
      "docs/scripts/i18n/reviewed.ts",
      "docs/davinci/plan/assertion-allowlist.toml",
      "tools/commands/davinci/assertion-lint.rs",
      "tools/support/common.rs",
      "tools/support/compat/davinci/lib/consumer-migration-scan.mjs",
      "tests/tooling/davinci-assertion-lint.test.ts",
      "tests/tooling/davinci-consumer-migration-surfaces.test.mjs",
      "tests/tooling/docs-reviewed-translations.test.ts",
      "tests/tooling/typecheck-divergence-report-timeout.test.ts",
      "tests/tooling/typecheck-divergence-mutation-oracle.test.ts",
      "tests/tooling/typecheck-fixture-custody.test.ts",
      ".github/workflows/pr-rust-checks.yml",
      "Cargo.lock",
      "new-runtime/lib.rs",
    ]) {
      const selected = plan([path], event);
      assert.equal(selected.sourceContracts, true, `${event}: ${path}`);
      assert.equal(prCheckOutputs(selected)["source-contracts"], "true", path);
      if (originalFailures.includes(path)) assert.equal(selected.source.tooling, false, path);
    }
    const unknownBase = plan([], event);
    assert.equal(unknownBase.sourceContracts, true, "an unknown predecessor cannot skip contracts");
    assert.equal(prCheckOutputs(unknownBase)["source-contracts"], "true");
    for (const path of ["npm/ui/src/Button.ts", "npm/oxlint/src/runtime.ts", "README.md"]) {
      const selected = plan([path], event);
      assert.equal(selected.sourceContracts, false, path);
      assert.equal(prCheckOutputs(selected)["source-contracts"], "false", path);
    }
  }
});

test("every docs, tooling and JS-only PR gets the real Relief compiler baseline", () => {
  for (const path of [
    "docs/guide.md",
    "tools/support/release/preflight.ts",
    "npm/oxlint/src/index.ts",
  ]) {
    const selected = plan([path]);
    const outputs = prCheckOutputs(selected);
    assert.equal(outputs.rust, "true", path);
    assert.deepEqual(selected.rust.packages, ["vize_relief"], path);
    assert.deepEqual(selected.rust.cargoArgs, ["--package", "vize_relief"]);
    assert.deepEqual(
      selected.rust.testMatrix.include.map((entry) => entry.cohort),
      ["compiler"],
    );
    assert.deepEqual(selected.rust.changedPackages, []);
    assert.deepEqual(selected.native.matrix, { lane: ["none"] });
    assert.deepEqual(selected.js.groups, path.startsWith("npm/") ? ["oxlint"] : []);
  }
});

test("Rust PRs validate their actual reverse dependency closure without replacing it with baseline", () => {
  assert.deepEqual(plan(["crates/vize_curator/src/lib.rs"]).rust.packages, ["vize_curator"]);
  const compiler = plan(["crates/vize_armature/src/lib.rs"]);
  assert.deepEqual(compiler.rust.packages, ["vize_armature", "vize_l4", "vize_vitrine"]);
  assert.deepEqual(
    compiler.rust.testMatrix.include.map((entry) => entry.cohort),
    ["compiler", "cli-and-integrations"],
  );
  assert.deepEqual(compiler.js.groups, ["builders", "ui", "oxlint"]);
  assert.equal(compiler.native.matrix.lane.length, 8);
});

test("unknown, shared, deleted and empty PR inputs retain full workspace validation", () => {
  for (const paths of [["Cargo.lock"], ["crates/deleted/src/lib.rs"], ["new-runtime/lib.rs"], []]) {
    const selected = plan(paths);
    assert.equal(selected.rust.scope, "workspace");
    assert.deepEqual(selected.rust.packages, allPackages);
  }
});

test("every merge candidate checks all Rust packages without widening actual JS/native changes", () => {
  for (const path of [
    "docs/guide.md",
    "npm/oxlint/src/index.ts",
    "crates/vize_curator/src/lib.rs",
    "crates/vize_armature/src/lib.rs",
  ]) {
    const pr = plan([path]);
    const merge = plan([path], "merge_group");
    assert.equal(prCheckOutputs(merge).rust, "true");
    assert.equal(merge.rust.scope, "workspace");
    assert.deepEqual(merge.rust.packages, allPackages);
    assert.deepEqual(
      merge.rust.cargoArgs,
      allPackages.flatMap((name) => ["--package", name]),
    );
    assert.deepEqual(merge.js, pr.js, path);
    assert.deepEqual(merge.native, pr.native, path);
  }
});

test("missing baseline or Cargo metadata fails planning instead of silently skipping Rust", () => {
  const withoutRelief = metadata();
  withoutRelief.workspace_members = withoutRelief.workspace_members.filter(
    (name) => name !== "vize_relief",
  );
  withoutRelief.packages = withoutRelief.packages.filter((pkg) => pkg.name !== "vize_relief");
  assert.throws(
    () =>
      planPrChecks({
        paths: ["docs/guide.md"],
        eventName: "pull_request",
        labels: [],
        workspace,
        cargoMetadata: withoutRelief,
      }),
    /missing required vize_relief baseline/,
  );
  assert.throws(
    () =>
      planPrChecks({ paths: ["docs/guide.md"], eventName: "pull_request", labels: [], workspace }),
    /require Cargo workspace metadata/,
  );
});

test("the real CLI reads offline metadata for docs PRs and emits mandatory PR/MQ plans", () => {
  const cwd = mkdtempSync(join(tmpdir(), "vize-required-rust-"));
  const git = (...args: string[]) => execFileSync("git", args, { cwd, encoding: "utf8" }).trim();
  try {
    const bin = join(cwd, "bin");
    const metadataPath = join(cwd, "metadata.json");
    const callsPath = join(cwd, "cargo-calls.jsonl");
    mkdirSync(bin);
    writeFileSync(metadataPath, JSON.stringify(metadata(cwd)));
    writeFileSync(
      join(bin, "cargo"),
      `#!${process.execPath}
const fs = require("node:fs");
fs.appendFileSync(process.env.CARGO_CALLS, JSON.stringify(process.argv.slice(2)) + "\\n");
process.stdout.write(fs.readFileSync(process.env.CARGO_METADATA, "utf8"));
`,
      { mode: 0o755 },
    );
    mkdirSync(join(cwd, "npm/native"), { recursive: true });
    writeFileSync(join(cwd, "npm/native/package.json"), JSON.stringify({ name: "@vizejs/native" }));
    writeFileSync(join(cwd, "pnpm-workspace.yaml"), 'packages:\n  - "npm/*"\n');
    writeFileSync(join(cwd, "README.md"), "before\n");
    git("init", "-q");
    git("config", "user.name", "CI Test");
    git("config", "user.email", "ci@example.invalid");
    git("add", ".");
    git("commit", "-qm", "before");
    const base = git("rev-parse", "HEAD");
    writeFileSync(join(cwd, "README.md"), "after\n");
    git("commit", "-qam", "after");
    for (const event of ["pull_request", "merge_group"]) {
      const output = join(cwd, `${event}-outputs`);
      const run = spawnSync(
        process.execPath,
        [
          fileURLToPath(
            new URL("../../tools/support/compat/github/plan-pr-checks.ts", import.meta.url),
          ),
          base,
          git("rev-parse", "HEAD"),
          event,
        ],
        {
          cwd,
          encoding: "utf8",
          env: {
            ...process.env,
            PATH: `${bin}:${process.env.PATH}`,
            CARGO_CALLS: callsPath,
            CARGO_METADATA: metadataPath,
            PR_LABELS: "[]",
            GITHUB_OUTPUT: output,
            GITHUB_STEP_SUMMARY: "",
          },
        },
      );
      assert.equal(run.status, 0, run.stderr);
      const outputs = Object.fromEntries(
        readFileSync(output, "utf8")
          .trimEnd()
          .split("\n")
          .map((line) => [line.slice(0, line.indexOf("=")), line.slice(line.indexOf("=") + 1)]),
      );
      assert.equal(outputs.rust, "true");
      assert.deepEqual(
        JSON.parse(outputs["rust-plan"]).packages,
        event === "pull_request" ? ["vize_relief"] : allPackages,
      );
      assert.equal(outputs.js, "false");
      assert.deepEqual(JSON.parse(outputs["native-runtime-matrix"]), { lane: ["none"] });
    }
    const calls = readFileSync(callsPath, "utf8")
      .trim()
      .split("\n")
      .map((line) => JSON.parse(line));
    assert.deepEqual(
      calls,
      Array.from({ length: 2 }, () => [
        "metadata",
        "--no-deps",
        "--format-version",
        "1",
        "--locked",
        "--offline",
      ]),
    );
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

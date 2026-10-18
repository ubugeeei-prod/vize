import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  changedPaths,
  isDocsQualityToolingInput,
  planSourceChecks,
} from "../../tools/support/compat/github/plan-source-checks.ts";
import { planAffectedRust } from "../../tools/support/compat/github/plan-affected-rust.ts";
import {
  localImportInputs,
  planToolingTests,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";
import { fixture } from "./_helpers/rust-workspace-fixture.mjs";

const quality = "docs/quality/more-guides-coverage.json";
const root = fileURLToPath(new URL("../../", import.meta.url));
const docsOnly = { rust: false, js: false, tooling: true, playground: false };
const qualityConsumer = /more-guides-coverage|docs[\\/]quality|quality[\\/]more-guides/;

void test("the exact authored More ledger retains Docs tooling without product source suites", () => {
  assert.equal(isDocsQualityToolingInput(quality), true);
  assert.deepEqual(planSourceChecks([quality]), docsOnly);
  assert.deepEqual(
    planSourceChecks([
      "docs/content/credits.md",
      "docs/content/ja/contributing.md",
      "docs/content/ja/credits.md",
      "docs/davinci/decisions/2026-09-27-level-restructure.md",
      "docs/davinci/decisions/2026-10-11-japanese-contribution-credits-review.md",
      quality,
      "tests/tooling/docs-reviewed-translations.test.ts",
    ]),
    docsOnly,
  );
  const rust = planAffectedRust(fixture(), [quality]);
  assert.equal(rust.scope, "none");
  assert.deepEqual(rust.packages, []);
  assert.deepEqual(rust.reasons, []);
  const tooling = planToolingTests([quality]);
  assert.ok(tooling.tests.includes("tests/tooling/docs-more-guides.test.ts"));
  assert.ok(tooling.tests.includes("tests/tooling/docs-quality-source-scope.test.mjs"));
});

void test("the actual reader remains Node-only and Docs checks do not build the product source CLI", () => {
  const reader = "docs/scripts/more-guides-coverage.ts";
  const closure = localImportInputs(reader);
  assert.equal(closure.complete, true);
  assert.deepEqual(closure.files.sort(), [reader, "docs/theme/i18n/sitemap.js"].sort());
  assert.match(
    readFileSync(join(root, reader), "utf8"),
    /readFileSync\(path\.join\(docsRoot, "quality\/more-guides-coverage\.json"\), "utf8"\)/,
  );
  for (const consumer of [
    "docs/scripts/verify-more-guides-render.ts",
    "tests/tooling/docs-more-guides.test.ts",
  ])
    assert.ok(localImportInputs(consumer).files.includes(reader), consumer);
  const rendering = readFileSync(join(root, ".github/workflows/build-docs.yml"), "utf8");
  assert.match(rendering, /vp run --filter '\.\/npm\/native' build:ci/);
  assert.doesNotMatch(rendering, /verify-more-guides-render/);
  const japaneseReading = rendering
    .split("      - name: ")
    .find((step) => step.startsWith("Verify rendered navigation and Japanese reading paths\n"));
  assert.ok(japaneseReading, "the reviewed Japanese browser checks remain enrolled");
  assert.match(japaneseReading, /if: \$\{\{ inputs\.links-only != true \}\}/);
  assert.ok(
    japaneseReading.includes(
      'defaultNavigationRenderRoutes.filter(route => route.startsWith("/ja/"))',
    ),
  );
  assert.ok(
    japaneseReading.includes(
      'node docs/scripts/verify-navigation-render.ts --routes "$ja_reading_routes"',
    ),
  );
  assert.ok(japaneseReading.includes("node docs/scripts/verify-ja-editorial-render.ts"));
  assert.doesNotMatch(rendering, /verify-og-images/);
  assert.doesNotMatch(
    readFileSync(join(root, ".github/workflows/pr-source-checks.yml"), "utf8"),
    /pr-source-cli|test:scripts:pr/,
  );
});

void test("every actual workspace Rust source and manifest remains outside the More quality consumer boundary", () => {
  const manifest = readFileSync(join(root, "Cargo.toml"), "utf8");
  const workspace = manifest.split(/^\[/m).find((section) => /^workspace\]\r?\n/.test(section));
  assert.ok(workspace, "workspace manifest section must be available");
  const members = /^members\s*=\s*\[([^\]]*)\]/m.exec(workspace);
  assert.ok(members, "workspace member list must be available");
  assert.doesNotMatch(manifest, qualityConsumer);
  let sources = 0,
    manifests = 0;
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (["target", "node_modules", ".git"].includes(entry.name)) continue;
      const file = join(directory, entry.name);
      if (entry.isDirectory()) visit(file);
      else if (entry.isFile() && (entry.name.endsWith(".rs") || entry.name === "Cargo.toml")) {
        assert.doesNotMatch(readFileSync(file, "utf8"), qualityConsumer, file);
        if (entry.name.endsWith(".rs")) sources++;
        else manifests++;
      }
    }
  };
  const paths = members[1]
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean);
  assert.ok(paths.length, "workspace members must not be empty");
  for (const path of paths) {
    const member = /^"((?:[A-Za-z0-9_-]+\/)*[A-Za-z0-9_-]+)",?$/.exec(path);
    assert.ok(member, `workspace member syntax requires consumer review: ${path}`);
    visit(join(root, member[1]));
  }
  assert.ok(sources > 0);
  assert.ok(manifests >= paths.length);
});

void test("actual npm product sources, scripts and manifests retain no More quality reader or ledger consumer", () => {
  let sources = 0,
    manifests = 0;
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      if (["node_modules", "target", "dist", ".artifacts", ".git"].includes(entry.name)) continue;
      const file = join(directory, entry.name);
      const source = /\.(?:[cm]?[jt]sx?|json|vue|sh|py|ya?ml|toml)$/.test(entry.name);
      if (entry.isDirectory()) visit(file);
      else if (source) {
        assert.ok(entry.isFile(), `linked product source requires consumer review: ${file}`);
        assert.doesNotMatch(readFileSync(file, "utf8"), qualityConsumer, file);
        sources++;
        if (entry.name === "package.json") manifests++;
      }
    }
  };
  visit(join(root, "npm"));
  assert.ok(sources > 0, "actual product sources must be scanned");
  assert.ok(manifests > 0, "actual product manifests must be scanned");
});

const unknown = [
  "docs/quality/new-ledger.json",
  "docs/quality/more-guides-coverage.schema.json",
  "docs/quality/more-guides-coverage.toml",
  "docs/quality/more-guides-coverage.ts",
  "docs/quality/nested/more-guides-coverage.json",
  "docs/quality/../quality/more-guides-coverage.json",
  "docs/quality/More-guides-coverage.json",
  "docs/scripts/more-guides-coverage.ts",
];
void test("nearby unknown contracts, scripts and changed spelling preserve conservative source planning", () => {
  const all = planAffectedRust(fixture(), ["Cargo.lock"]);
  for (const path of unknown) {
    assert.equal(isDocsQualityToolingInput(path), false, path);
    assert.deepEqual(
      planSourceChecks([quality, path]),
      { rust: true, js: true, tooling: true, playground: true },
      path,
    );
    const rust = planAffectedRust(fixture(), [quality, path]);
    assert.equal(rust.scope, "workspace", path);
    assert.deepEqual(rust.packages, all.packages, path);
  }
});

void test("mixed product, workflow, native and shared contracts retain their actual gates and Rust consumers", () => {
  for (const path of [
    "crates/syntax/src/lib.rs",
    "davinci/vize_l1/src/lib.rs",
    "npm/native/src/index.ts",
    "npm/cli/schemas/vize.config.schema.json",
    "docs/davinci/plan/budgets.toml",
    ".github/workflows/pr-source-checks.yml",
    ".github/actions/test-native-js-setup/action.yml",
    "Cargo.lock",
  ]) {
    assert.deepEqual(
      planSourceChecks([quality, path]),
      { ...planSourceChecks([path]), tooling: true },
      path,
    );
    assert.deepEqual(
      planAffectedRust(fixture(), [quality, path]),
      planAffectedRust(fixture(), [path]),
      path,
    );
  }
});

void test("merge groups keep the exact known Docs ledger outside product source validation", () => {
  assert.deepEqual(planSourceChecks([quality], "merge_group"), planSourceChecks([quality]));
  assert.deepEqual(planAffectedRust(fixture(), [quality], "merge_group").packages, []);
});

void test("actual renamed and deleted quality paths remain visible rather than narrowing a new unknown contract", () => {
  const cwd = mkdtempSync(join(os.tmpdir(), "vize-docs-quality-paths-"));
  try {
    const git = (...args) => execFileSync("git", args, { cwd, encoding: "utf8" });
    git("init", "-q");
    mkdirSync(join(cwd, "docs/quality"), { recursive: true });
    writeFileSync(join(cwd, quality), "complete authored controlled ledger\n");
    git("add", ".");
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "-qm",
      "before",
    );
    const base = git("rev-parse", "HEAD").trim();
    const renamed = "docs/quality/new-ledger.json";
    git("mv", quality, renamed);
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "-qm",
      "rename",
    );
    const moved = changedPaths(base, git("rev-parse", "HEAD").trim(), cwd);
    assert.deepEqual(moved.sort(), [quality, renamed].sort());
    assert.equal(planSourceChecks(moved).rust, true);
    assert.equal(planAffectedRust(fixture(), moved).scope, "workspace");
    const previous = git("rev-parse", "HEAD").trim();
    git("rm", renamed);
    git(
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "-qm",
      "delete",
    );
    const deleted = changedPaths(previous, git("rev-parse", "HEAD").trim(), cwd);
    assert.deepEqual(deleted, [renamed]);
    assert.equal(planSourceChecks(deleted).js, true);
    assert.equal(planAffectedRust(fixture(), deleted).scope, "workspace");
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

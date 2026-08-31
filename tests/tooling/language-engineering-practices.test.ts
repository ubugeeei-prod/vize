import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

// The docs theme has no module loader; each i18n script registers itself on a
// global when loaded. `docs-navigation.test.ts` owns the wiring assertions.
for (const basename of ["sitemap", "locales/en"]) {
  await import(path.join(root, "docs/theme/i18n", `${basename}.js`));
}
const sitemap = (globalThis as { __vizeDocsSitemap?: { navGroups: NavGroup[] } })
  .__vizeDocsSitemap!;
const locales = (globalThis as { __vizeDocsLocales?: Record<string, LocaleStrings> })
  .__vizeDocsLocales!;

type NavGroup = { key: string; paths: string[] };
type LocaleStrings = { labels: Record<string, string> };

test("language engineering practices document upstream-derived compiler workflows", () => {
  const practices = readRepoFile(
    "docs",
    "content",
    "architecture",
    "language-engineering-practices.md",
  );

  for (const source of [
    "rust-lang/rust",
    "rustc-dev-guide",
    "rustc` ecosystem and perf testing",
    "rust-fuzz/cargo-fuzz",
    "Linux kernel testing",
    "Chromium testing and CQ",
    "V8 testing",
    "feature launch",
    "microsoft/TypeScript",
    "TypeScript tests/cases/fourslash",
    "microsoft/typescript-go",
    "facebook/flow",
  ]) {
    assert.match(practices, new RegExp(escapeRegExp(source)), source);
  }

  for (const practice of [
    "compiletest",
    "tests/baselines/reference",
    "baseline-accept",
    "testdata/baselines/local",
    "Crater",
    "rustc-perf",
    "KUnit",
    "kselftest",
    "KCOV",
    "perf stat",
    "Telemetry",
    "ClusterFuzz",
    "Test262",
    "mjsunit",
    "tools/run_perf.py",
    "tests/cases/fourslash",
    ".diff",
    ".exp",
    "newtests",
  ]) {
    assert.match(practices, new RegExp(escapeRegExp(practice)), practice);
  }

  for (const vizeArtifact of [
    "tests/fixtures",
    "tests/expected",
    "tests/snapshots/check",
    "tests/snapshots/lint",
    "tools/benchmarks/scripts/test-inventory.mjs",
    ".github/workflows/benchmark.yml",
    ".github/workflows/check.yml",
    ".github/workflows/fuzz.yml",
    "tests/fuzz/Cargo.toml",
    "tools/commands/ci/fuzz/seed_corpus.rs",
    "tools/benchmarks/scripts/enforce-pr-budget.mjs",
    "security-audit",
    "vp exec pnpm audit --prod --audit-level moderate",
    "cargo audit --deny warnings",
    "docs/release/production-readiness.md",
    "docs/release/vue-parity-matrix.md",
  ]) {
    assert.match(practices, new RegExp(escapeRegExp(vizeArtifact)), vizeArtifact);
  }
});

test("language-facing change classes are present in docs, contribution guide, and PR template", () => {
  const practices = readRepoFile(
    "docs",
    "content",
    "architecture",
    "language-engineering-practices.md",
  );
  const contributing = readRepoFile("CONTRIBUTING.md");
  const pullRequestTemplate = readRepoFile(".github", "PULL_REQUEST_TEMPLATE.md");

  for (const changeClass of [
    "Parser or AST",
    "Compiler and codegen",
    "Semantic analysis, lint, and cross-file analysis",
    "Virtual TypeScript and type checking",
    "Formatter and LSP",
    "Runtime packaging, release, or docs",
  ]) {
    const pattern = new RegExp(escapeRegExp(changeClass));
    assert.match(practices, pattern, changeClass);
    assert.match(contributing, pattern, changeClass);
    assert.match(pullRequestTemplate, pattern, changeClass);
  }

  assert.match(contributing, /Language Engineering Practices/);
  assert.match(pullRequestTemplate, /## Change Class/);
  assert.match(pullRequestTemplate, /## Verification Evidence/);
  assert.match(pullRequestTemplate, /Snapshot\/baseline changes reviewed and explained/);
  assert.match(pullRequestTemplate, /Security\/audit impact considered/);
  assert.match(pullRequestTemplate, /Performance status or PR benchmark impact considered/);
  assert.match(pullRequestTemplate, /Fuzzing target or crash-reproducer coverage considered/);
  assert.match(pullRequestTemplate, /Editor\/LSP scenario coverage considered/);
  assert.match(pullRequestTemplate, /Ecosystem or broad app compatibility impact considered/);
});

test("docs navigation exposes the language engineering practices page", () => {
  const architectureOverview = readRepoFile("docs", "content", "architecture", "overview.md");
  const productionReadiness = readRepoFile("docs", "release", "production-readiness.md");

  // The sidebar's structure and its labels are data, not file text: the groups
  // live in `docs/theme/i18n/sitemap.js` and the labels in
  // `docs/theme/i18n/locales/`. Assert the whole Architecture group so a page
  // dropped from it fails here rather than reading as an unrelated reorder.
  assert.deepEqual(sitemap.navGroups.find((group) => group.key === "architecture")?.paths, [
    "/architecture/overview",
    "/architecture/crates",
    "/architecture/source-guide",
    "/architecture/language-engineering-practices",
    "/architecture/performance",
    "/philosophy",
  ]);
  assert.equal(
    locales.en.labels["/architecture/language-engineering-practices"],
    "Language Engineering",
  );

  assert.match(architectureOverview, /language-engineering-practices\.md/);
  assert.match(productionReadiness, /language-engineering change-class evidence/);
  assert.match(productionReadiness, /security-audit` workflow status/);
  assert.match(productionReadiness, /pr-benchmark-budget` status/);
  assert.match(productionReadiness, /\.github\/workflows\/fuzz\.yml/);
});

function readRepoFile(...segments: string[]): string {
  return fs.readFileSync(path.join(root, ...segments), "utf8");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

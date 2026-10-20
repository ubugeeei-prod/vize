// Keeps the canonical corpus selection honest: every input the PR/queue
// canonical corpus run provably reads must select the corpus, derived here
// from the workflows, local actions, script imports, Cargo path dependencies
// and literal fixture paths instead of being restated by hand.
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, posix } from "node:path";
import { test } from "node:test";
import { parse as parseToml } from "@iarna/toml";
import { parse as parseYaml } from "yaml";
import {
  canonicalCorpusRequired,
  changedLinks,
} from "../../tools/support/compat/github/canonical-corpus-selection.mjs";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = { uses?: string; run?: string };
type Job = { uses?: string; steps?: Step[] };
type Workflow = { jobs: Record<string, Job> };

const corpusWorkflow = ".github/workflows/davinci-canonical-corpus.yml";
const selects = (path: string) => canonicalCorpusRequired([path], "pull_request");
const exists = (path: string) => existsSync(join(root, path));
const index = execFileSync("git", ["ls-files", "-z"], {
  cwd: root,
  encoding: "utf8",
  maxBuffer: 256 * 1024 * 1024,
})
  .split("\0")
  .filter(Boolean);
const tracked = (path: string) =>
  index.filter((file) => file === path || file.startsWith(`${path}/`));
const yaml = (path: string) => parseYaml(readRepoFile(path)) as Workflow;
const steps = (workflow: Workflow) =>
  Object.values(workflow.jobs).flatMap((job) => job.steps ?? []);

function assertSelected(path: string, reason: string) {
  const files = tracked(path);
  assert(files.length > 0, `${reason}: ${path} is not tracked`);
  for (const file of files) assert(selects(file), `${reason}: ${file} must select the corpus`);
}

await test("the canonical call path and every local action it uses select the corpus", () => {
  const check = yaml(".github/workflows/check.yml");
  const source = yaml(".github/workflows/pr-source-checks.yml");
  assert.equal(check.jobs["pr-source-checks"]?.uses, "./.github/workflows/pr-source-checks.yml");
  assert.equal(source.jobs["canonical-corpus"]?.uses, `./${corpusWorkflow}`);
  for (const workflow of [
    ".github/workflows/check.yml",
    ".github/workflows/pr-source-checks.yml",
    corpusWorkflow,
  ])
    assertSelected(workflow, "canonical workflow");
  const pending = steps(yaml(corpusWorkflow)).map((step) => step.uses ?? "");
  const actions = new Set<string>();
  while (pending.length) {
    const uses = pending.pop() as string;
    if (!uses.startsWith("./")) continue;
    const directory = posix.normalize(uses.slice(2));
    if (actions.has(directory)) continue;
    actions.add(directory);
    const action = parseYaml(readRepoFile(directory, "action.yml")) as {
      runs?: { steps?: Step[] };
    };
    pending.push(...(action.runs?.steps ?? []).map((step) => step.uses ?? ""));
  }
  assert(actions.size >= 3, "Missing canonical local actions");
  for (const directory of actions) assertSelected(directory, "canonical local action");
  // setup-node reads the pinned runtime version.
  assert(steps(yaml(corpusWorkflow)).some((step) => step.uses?.startsWith("actions/setup-node@")));
  assertSelected("package.json", "setup-node runtime");
  for (const unrelated of [
    ".github/workflows/e2e.yml",
    ".github/workflows/real-project-matrix.yml",
    ".github/actions/install-playwright-browsers/action.yml",
  ]) {
    assert(exists(unrelated), unrelated);
    assert(!selects(unrelated), `${unrelated} is not read by the canonical corpus run`);
  }
});

const importPattern =
  /(?:\bfrom\s*|\bimport\s*\(\s*|\bnew URL\(\s*)["'](\.{1,2}\/[^"']+\.(?:mjs|cjs|js|ts))["']/g;
// Repository-root paths, also when reached through `../` from a crate.
const literalPattern = /(?:^|[^\w.-])((?:tests\/_fixtures|docs)\/[\w@.+-]+(?:\/[\w@.+-]+)*)/g;

function literalReads(text: string): string[] {
  return [...text.matchAll(literalPattern)]
    .map((match) => (match[1] as string).replace(/\/+$/, ""))
    .filter((path) => tracked(path).length > 0);
}

// Path constants of other tools that share a module with a corpus helper;
// the canonical jobs never read them.
const unreadConstants = new Set([
  "docs/davinci/plan/croquis-consumption.md",
  "docs/davinci/plan/croquis-consumption",
]);

function scriptClosure(roots: string[]) {
  const pending = [...roots];
  const scripts = new Set<string>();
  const literals = new Set<string>();
  while (pending.length) {
    const script = posix.normalize(pending.pop() as string);
    if (scripts.has(script)) continue;
    scripts.add(script);
    const text = readRepoFile(script);
    for (const literal of literalReads(text))
      if (!unreadConstants.has(literal)) literals.add(literal);
    const imports = script.endsWith(".rs")
      ? [...text.matchAll(/#\[path\s*=\s*"([^"]+)"\]/g)]
      : [...text.matchAll(importPattern)];
    for (const match of imports)
      pending.push(posix.join(posix.dirname(script), match[1] as string));
  }
  for (const script of scripts) {
    assert(exists(script), `${script} is imported but missing`);
    assertSelected(script, "canonical script");
  }
  return { scripts, literals };
}

await test("every script, import and literal fixture read of the canonical jobs selects the corpus", () => {
  const scriptPattern = /\b(?:node|rust-script)\s+((?:tools|tests)\/[\w./-]+\.(?:mjs|ts|rs))/g;
  const roots = steps(yaml(corpusWorkflow)).flatMap((step) =>
    [...(step.run ?? "").matchAll(scriptPattern)].map((match) => match[1] as string),
  );
  assert(roots.includes("tools/support/compat/github/canonical-corpus-shards.ts"));
  const { scripts, literals } = scriptClosure(roots);
  for (const literal of literals) assertSelected(literal, "canonical script read");
  // The source plan and source report run the selection itself; its path
  // predicates are not reads, but its code must select the corpus.
  const selection = [
    ...readRepoFile(".github/workflows/pr-source-checks.yml").matchAll(scriptPattern),
  ]
    .map((match) => match[1] as string)
    .filter((path) => path.includes("canonical-corpus"));
  assert(selection.includes("tools/support/compat/github/canonical-corpus-selection.mjs"));
  scriptClosure(selection);
  for (const expected of [
    "tools/support/compat/github/canonical-corpus-batch.ts",
    "tools/support/compat/github/release-preflight-github.mjs",
    "tools/support/compat/davinci/lib/corpus-baseline-contract.ts",
    "tools/support/common.rs",
    ".github/actions/setup-rust-sticky-cache/cache-policy.ts",
  ])
    assert(scripts.has(expected), `${expected} left the derived script closure`);
  assert(literals.has("tests/_fixtures/vue-ecosystem-fixtures.json"));
});

const observerTests = [
  "crates/vize_atelier_sfc/tests/davinci_production_reach",
  "crates/vize_atelier_ssr/tests/davinci_ssr_corpus",
  "davinci/vize_l1_to_l2/tests/davinci_dom_corpus",
  "davinci/vize_l1_to_l2/tests/davinci_pug_corpus",
];

await test("every fixture the observer test binaries read selects the corpus", () => {
  const sources = observerTests.flatMap((entry) => {
    const directory = posix.dirname(entry);
    const name = posix.basename(entry);
    return [
      `${entry}.rs`,
      ...tracked(`${directory}/${name}`),
      ...tracked(`${directory}/${name}_support`),
      ...(name === "davinci_pug_corpus" ? tracked(`${directory}/support`) : []),
    ].filter((path) => path.endsWith(".rs"));
  });
  sources.push(...tracked("tests/davinci_test_support/src").filter((path) => path.endsWith(".rs")));
  const literals = new Set(sources.flatMap((source) => literalReads(readRepoFile(source))));
  for (const expected of [
    "tests/_fixtures/davinci-pug",
    "tests/_fixtures/differential/compiler/ssr-builtins-7891",
    "tests/_fixtures/differential/compiler/typed-slot-handler-scope-8142/cases.json",
    "tests/_fixtures/differential/compiler/n8n-if-key-regression/diagnostics.json",
    "docs/davinci/plan/reach-budgets.toml",
  ])
    assert(literals.has(expected), `${expected} left the derived observer reads`);
  for (const literal of literals) assertSelected(literal, "observer test read");
  // Production reach sweeps every committed fixture SFC outside the gitlinks.
  const vue = tracked("tests/_fixtures").filter(
    (path) => path.endsWith(".vue") && !path.startsWith("tests/_fixtures/_git/"),
  );
  assert(vue.length > 400, "Committed fixture SFC census shrank");
  for (const path of vue) assert(selects(path), `${path} must select the corpus`);
  for (const gitlink of tracked("tests/_fixtures/_git"))
    assert(selects(gitlink), `${gitlink} must select the corpus`);
  for (const unread of [
    "tests/_fixtures/fixture-compatibility-ledger.json",
    "tests/_fixtures/vue-benchmarks-upstream.json",
    "tests/_fixtures/n8n-adoption.json",
  ]) {
    assert(exists(unread), unread);
    assert(!selects(unread), `${unread} is not read by the canonical corpus run`);
  }
  assert(!selects("tests/_fixtures/differential/lint/unread/case.json"));
});

await test("every Cargo path dependency of the observer binaries selects the corpus", () => {
  type Manifest = Record<string, unknown>;
  const toml = (path: string) => parseToml(readRepoFile(path)) as Manifest;
  const workspace = (toml("Cargo.toml").workspace as Manifest).dependencies as Record<
    string,
    { path?: string }
  >;
  const tables = (manifest: Manifest, kinds: string[]) => {
    const found: Record<string, { path?: string; workspace?: boolean }>[] = [];
    for (const kind of kinds) found.push((manifest[kind] ?? {}) as never);
    for (const target of Object.values((manifest.target ?? {}) as Record<string, Manifest>))
      for (const kind of kinds) found.push((target[kind] ?? {}) as never);
    return found;
  };
  const packages = [...new Set(observerTests.map((entry) => posix.dirname(posix.dirname(entry))))];
  const pending = packages.map((directory) => ({ directory, root: true }));
  const seen = new Set<string>();
  while (pending.length) {
    const { directory, root: isRoot } = pending.pop() as { directory: string; root: boolean };
    if (seen.has(directory)) continue;
    seen.add(directory);
    const kinds = ["dependencies", "build-dependencies", ...(isRoot ? ["dev-dependencies"] : [])];
    for (const table of tables(toml(`${directory}/Cargo.toml`), kinds))
      for (const [name, spec] of Object.entries(table)) {
        const path =
          typeof spec === "object" && spec.path
            ? posix.normalize(posix.join(directory, spec.path))
            : typeof spec === "object" && spec.workspace
              ? workspace[name]?.path
              : undefined;
        if (path) pending.push({ directory: path, root: false });
      }
  }
  assert(seen.has("davinci/vize_l0") && seen.has("crates/vize_croquis"), "Missing core crates");
  for (const directory of seen) assertSelected(`${directory}/Cargo.toml`, "observer Cargo crate");
});

await test("changed fixture links select the corpus because the committed sweep follows them", () => {
  const link = "tests/_fixtures/differential/lint/linked-components";
  assert(!selects(link));
  assert(canonicalCorpusRequired([link], "pull_request", [link]));
  assert(!canonicalCorpusRequired(["docs/linked"], "merge_group", ["docs/linked"]));
  assert.throws(() => canonicalCorpusRequired([link], "pull_request", ["foreign"]));
  const directory = mkdtempSync(join(tmpdir(), "vize-canonical-links-"));
  const git = (...args: string[]) =>
    execFileSync("git", args, { cwd: directory, encoding: "utf8" }).trim();
  try {
    git("init", "--quiet");
    git("config", "user.email", "fixture@example.invalid");
    git("config", "user.name", "Fixture");
    writeFileSync(join(directory, "plain.json"), "{}\n");
    git("add", ".");
    git("commit", "--quiet", "-m", "base");
    const base = git("rev-parse", "HEAD");
    symlinkSync("plain.json", join(directory, "alias"));
    writeFileSync(join(directory, "plain.json"), "{ }\n");
    git("add", ".");
    git("commit", "--quiet", "-m", "link");
    assert.deepEqual(changedLinks(base, git("rev-parse", "HEAD"), directory), ["alias"]);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
});

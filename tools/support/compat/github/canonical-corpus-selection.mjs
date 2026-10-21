import { execFileSync } from "node:child_process";
import { appendFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { changedPaths } from "./plan-source-checks.ts";
import { aggregateCheckNeedsResults } from "./require-needs-success.ts";

// Only paths the canonical corpus jobs provably read. Keep this list honest
// with tests/tooling/davinci-canonical-corpus-selection-inputs.test.ts, which
// derives the read set from the workflows, script imports, Cargo path
// dependencies and literal fixture paths.
const inputs = [
  /^\.gitmodules$/,
  // The workflows on the PR/queue call path and the local actions they use.
  /^\.github\/workflows\/(?:check|pr-source-checks|davinci-canonical-corpus)\.yml$/,
  /^\.github\/actions\/(?:setup-rust-script|setup-wild-http-recovery|setup-rust-sticky-cache|reuse-rust-target)\//,
  // setup-node reads the pinned runtime from package.json.
  /^package\.json$/,
  /^(?:Cargo\.(?:toml|lock)|rust-toolchain\.toml)$/,
  /^\.cargo\//,
  /^davinci\//,
  /^crates\/vize_(?:atelier_|armature\/|carton\/|croquis\/|croquis_cf\/|curator\/|relief\/)/,
  /^vendor\//,
  // Pinned gitlinks, the committed fixture SFCs swept by production reach,
  // and the fixture folders the observer binaries and recorder read.
  /^tests\/_fixtures\/_git\//,
  /^tests\/_fixtures\/(?:.*\/)?[^/]*\.vue$/,
  /^tests\/_fixtures\/davinci-pug\//,
  /^tests\/_fixtures\/differential\/compiler\/(?:ssr-builtins-7891|typed-slot-handler-scope-8142|n8n-if-key-regression)\//,
  /^tests\/_fixtures\/(?:vue-ecosystem-fixtures|davinci-baseline)\.json$/,
  /^docs\/davinci\/plan\/(?:reach-budgets\.toml|corpus-baseline-notes\.md|corpus-baseline-unstable\.json)$/,
  /^tests\/davinci_test_support\//,
  /^tests\/tooling\/(?:davinci-dom-corpus|davinci-canonical-corpus|fixture-)/,
  /^tests\/tooling\/fixtures\/canonical-observer-logs\//,
  /^tests\/tooling\/support\/canonical-corpus-worker-inventory\.mjs$/,
  /^tools\/(?:commands\/fixtures|support\/compat\/fixtures|benchmarks)\//,
  /^tools\/support\/common\.rs$/,
  /^tools\/support\/(?:command-runtime|json-value)\.ts$/,
  /^tools\/support\/compat\/davinci\/lib\/(?:corpus-baseline-contract|corpus-baseline-artifact|ordering|paths)\.ts$/,
  /^tools\/support\/compat\/github\/canonical-corpus-[a-z-]+\.(?:mjs|ts|rs)$/,
  /^tools\/support\/compat\/github\/(?:comparison-base|plan-source-checks|require-needs-success|prepare-vue-benchmarks|release-preflight-github)\.mjs$/,
  /^tools\/support\/compat\/github\/(?:comparison-base|plan-source-checks)\.ts$/,
  /^tools\/support\/compat\/github\/real-project-matrix-plan\.ts$/,
  /^tools\/support\/compat\/github\/(?:require-needs-success|require-rust-tier)\.ts$/,
];
// A symlink can make the committed fixture sweep read any directory, so a
// changed link anywhere under tests/_fixtures selects the corpus.
const fixtureLink = /^tests\/_fixtures\//;

export function canonicalCorpusRequired(paths, eventName, links = []) {
  if (!["pull_request", "merge_group"].includes(eventName))
    throw new Error("expected PR or merge-group canonical corpus context");
  if (!Array.isArray(paths)) throw new Error("expected changed paths");
  if (paths.length === 0) return true;
  for (const path of paths) {
    if (
      typeof path !== "string" ||
      path.length === 0 ||
      path.startsWith("/") ||
      path.split("/").some((part) => part === "." || part === "..") ||
      path.includes("\0")
    )
      throw new Error("expected repository-relative changed path");
  }
  if (!Array.isArray(links) || links.some((link) => !paths.includes(link)))
    throw new Error("expected changed link paths");
  return paths.some(
    (path) =>
      inputs.some((pattern) => pattern.test(path)) ||
      (links.includes(path) && fixtureLink.test(path)),
  );
}

// Changed paths whose old or new Git mode is a symlink or gitlink.
export function changedLinks(base, head, cwd = process.cwd()) {
  const fields = execFileSync(
    "git",
    ["diff", "--no-renames", "--raw", "-z", "--diff-filter=ACDMRT", base, head],
    { cwd, encoding: "utf8", maxBuffer: 256 * 1024 * 1024 },
  ).split("\0");
  const links = [];
  for (let index = 0; index + 1 < fields.length; index += 2) {
    const modes = /^:(\d{6}) (\d{6}) /.exec(fields[index]);
    if (!modes) throw new Error("unexpected raw diff record");
    if ([modes[1], modes[2]].some((mode) => mode === "120000" || mode === "160000"))
      links.push(fields[index + 1]);
  }
  return links;
}

export function requireCanonicalCorpusSuccess(needs) {
  const plan = needs?.["pr-source-plan"];
  const result = needs?.["canonical-corpus"]?.result;
  if (plan?.result !== "success") throw new Error("canonical corpus plan did not succeed");
  const required = plan.outputs?.["canonical-corpus"];
  if (required !== "true" && required !== "false")
    throw new Error("missing canonical corpus selection");
  if (required === "true" && result !== "success")
    throw new Error(`required canonical corpus did not succeed: ${result}`);
  if (required === "false" && result !== "success" && result !== "skipped")
    throw new Error(`optional canonical corpus did not succeed: ${result}`);
}

export function canonicalSourceReport(needs, eventName) {
  requireCanonicalCorpusSuccess(needs);
  const original = Object.fromEntries(
    Object.entries(needs).filter(([job]) => job !== "canonical-corpus"),
  );
  return aggregateCheckNeedsResults(original, eventName);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const args = process.argv.slice(2);
  if (args.length === 1 && args[0] === "--check") {
    const needs = JSON.parse(process.env.NEEDS_JSON ?? "null");
    const report = canonicalSourceReport(needs, process.env.GITHUB_EVENT_NAME);
    process.stdout.write(`${report.message}\n`);
    process.exitCode = report.exitCode;
  } else {
    const [base, head, eventName] = args;
    if (
      args.length !== 3 ||
      !/^[0-9a-f]{40}$/.test(base ?? "") ||
      !/^[0-9a-f]{40}$/.test(head ?? "") ||
      /^0+$/.test(head)
    )
      throw new Error("expected full comparison and candidate identities");
    const paths = /^0+$/.test(base) ? [] : changedPaths(base, head);
    const links = /^0+$/.test(base) ? [] : changedLinks(base, head);
    const required = canonicalCorpusRequired(paths, eventName, links);
    if (process.env.GITHUB_OUTPUT)
      appendFileSync(process.env.GITHUB_OUTPUT, `canonical-corpus=${required}\n`);
    process.stdout.write(`Canonical corpus: ${required}; changed paths: ${paths.length}\n`);
  }
}

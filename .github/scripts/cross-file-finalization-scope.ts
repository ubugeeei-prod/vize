/** This planner selects the frozen historical experiment; it never qualifies performance. */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { appendFileSync, mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

export const HISTORICAL_FINALIZATION_PATHS = [
  "crates/vize/src/commands/lint/cross_file.rs",
  "crates/vize_vitrine/src/wasm/cross_file.rs",
  "crates/vize_croquis_cf/benches/cross_file.rs",
] as const;

export function planFinalizationComparison(
  changedPaths: readonly string[],
  event: string,
  expectedPaths: readonly string[] = HISTORICAL_FINALIZATION_PATHS,
) {
  assert.ok(
    event === "pull_request" || event === "workflow_dispatch",
    "unknown comparison trigger",
  );
  assert.ok(
    changedPaths.every(
      (name) => typeof name === "string" && name.length > 0 && !name.includes("\0"),
    ),
  );
  assert.ok(expectedPaths.length > 0 && new Set(expectedPaths).size === expectedPaths.length);
  assert.ok(
    expectedPaths.every(
      (name) =>
        /^(?:crates|davinci)\/[A-Za-z0-9_./-]+\.rs$/u.test(name) &&
        !name.split("/").some((part) => part === "." || part === ".." || part.length === 0),
    ),
    "historical source scope must contain unique relative Rust paths",
  );
  const expected = [...expectedPaths].sort();
  const actual = [...changedPaths].sort();
  const scopeMatches = JSON.stringify(actual) === JSON.stringify(expected);
  return {
    scopeMatches,
    runComparison: event === "workflow_dispatch" || scopeMatches,
    manualRequest: event === "workflow_dispatch",
    expectedPaths: expected,
    actualPaths: actual,
    reason: scopeMatches
      ? "historical changed-source scope matches"
      : expectedPaths === HISTORICAL_FINALIZATION_PATHS
        ? "outside the historical three-file changed-source certificate"
        : "outside the separately reviewed historical ordering source certificate",
    metricsProduced: false,
    measurementQualification:
      "not evaluated by the planner; the original comparator remains strict",
  };
}

export function main(
  options: {
    expectedPaths?: readonly string[];
    outputDirectory?: string;
    summaryTitle?: string;
  } = {},
) {
  const base = process.env.BASE_SOURCE_SHA,
    head = process.env.EXPECTED_SOURCE_SHA;
  assert.match(base ?? "", /^[a-f0-9]{40}$/u);
  assert.match(head ?? "", /^[a-f0-9]{40}$/u);
  const actualHead = execFileSync("git", ["rev-parse", "HEAD"], { encoding: "utf8" }).trim();
  assert.equal(actualHead, head, "planner must inspect the exact authored source");
  const changed = execFileSync(
    "git",
    ["diff", "--name-only", "-z", base!, head!, "--", "crates", "davinci"],
    { encoding: "utf8" },
  )
    .split("\0")
    .filter(Boolean);
  const plan = {
    schemaVersion: 1,
    baseSource: base,
    headSource: head,
    ...planFinalizationComparison(
      changed,
      process.env.GITHUB_EVENT_NAME ?? "",
      options.expectedPaths,
    ),
  };
  const directory = resolve(
    options.outputDirectory ?? "target/vize-tests/cross-file-finalization-scope",
  );
  mkdirSync(directory, { recursive: true });
  writeFileSync(resolve(directory, "scope.json"), JSON.stringify(plan, null, 2) + "\n");
  assert.ok(process.env.GITHUB_OUTPUT);
  appendFileSync(process.env.GITHUB_OUTPUT, `run_comparison=${plan.runComparison}\n`);
  if (process.env.GITHUB_STEP_SUMMARY)
    appendFileSync(
      process.env.GITHUB_STEP_SUMMARY,
      `### ${options.summaryTitle ?? "Historical cross-file finalization comparison"}\n\nBase: ${base}\nHead: ${head}\n\n` +
        (plan.runComparison
          ? "The strict historical comparator will evaluate its original prerequisites and counters.\n"
          : "Not applicable to this changed-source scope. No performance measurements or performance pass are claimed.\n") +
        `\nScope evidence: ${JSON.stringify(plan.actualPaths)}\n`,
    );
  console.log(JSON.stringify(plan));
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) main();

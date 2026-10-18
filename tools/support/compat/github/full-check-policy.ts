import assert from "node:assert/strict";
import fs from "node:fs";
import { isEntrypoint } from "../../command-runtime.ts";

type Row = Record<string, unknown>;
export type FullCheckProfile = "legacy" | "full-qualification-v2";

const legacyJobs = [
  "test-scripts",
  "test-js-packages",
  "build-js-packages",
  "full-js-report",
  "Full Rust workspace and doctests",
  "clippy-and-test",
  "full-rust-report",
] as const;
const workflow = ".github/workflows/full-qualification.yml";

/**
 * GitHub's own referenced-workflow snapshot binds this profile to the selected
 * run's frozen source. A static reference does not prove execution: all eight
 * actual successful jobs, plus the whole run, remain independently mandatory.
 * Historical runs without this callee keep their exact original job contract.
 */
export function fullCheckProfile(run: Row | undefined): FullCheckProfile {
  const references = run?.referenced_workflows;
  if (references === undefined) return "legacy";
  assert.ok(Array.isArray(references), "Check referenced workflow inventory required");
  const matches = references.filter((value: unknown) => {
    assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
    const row = value as Row;
    assert.equal(typeof row.path, "string", "referenced workflow path required");
    return (row.path as string).includes(workflow);
  });
  if (matches.length === 0) return "legacy";
  assert.equal(matches.length, 1, "unique full qualification workflow required");
  assert.ok(run !== undefined);
  assert.equal(run.path, ".github/workflows/check.yml", "full Check workflow changed");
  const head = run.head_sha;
  if (typeof head !== "string") throw new Error("frozen Check source required");
  assert.match(head, /^[a-f0-9]{40}$/u, "frozen Check source required");
  assert.ok(
    run.event === "workflow_dispatch" || run.event === "schedule",
    "full Check event changed",
  );
  const title = `Check ${run.event === "workflow_dispatch" ? "full" : "schedule"} @ ${head}`;
  assert.equal(run.name, title, "full Check name changed");
  assert.equal(run.display_title, title, "full Check title changed");
  assert.ok(
    run.repository !== null && typeof run.repository === "object" && !Array.isArray(run.repository),
    "Check repository required",
  );
  const repository = (run.repository as Row).full_name;
  if (typeof repository !== "string") throw new Error("Check repository required");
  assert.match(repository, /^[\w.-]+\/[\w.-]+$/u);
  const selected = matches[0] as Row;
  assert.equal(selected.sha, run.head_sha, "full qualification source changed");
  assert.equal(
    selected.path,
    `${repository}/${workflow}@${head}`,
    "full qualification repository/path/source changed",
  );
  assert.equal(typeof run.head_branch, "string", "Check source ref required");
  assert.ok(typeof selected.ref === "string", "full qualification ref required");
  assert.equal(
    selected.ref,
    `refs/${/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u.test(run.head_branch as string) ? "tags" : "heads"}/${run.head_branch as string}`,
    "full qualification ref changed",
  );
  return "full-qualification-v2";
}

export function fullCheckJobNames(run?: Row): string[] {
  if (fullCheckProfile(run) === "legacy") return [...legacyJobs];
  return [
    ...legacyJobs.map((name) =>
      name === "full-js-report" ? name : `Full qualification / ${name}`,
    ),
    "Full qualification / Nuxt module qualification / Nuxt 3 build and SSR",
  ];
}

if (isEntrypoint(import.meta.url)) {
  assert.deepEqual(
    process.argv.slice(2),
    ["job-names"],
    "expected job-names with run JSON on stdin",
  );
  const value: unknown = JSON.parse(fs.readFileSync(0, "utf8"));
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
  console.log(JSON.stringify(fullCheckJobNames(value as Row)));
}

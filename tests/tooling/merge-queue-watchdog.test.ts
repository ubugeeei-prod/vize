import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

type Workflow = {
  on: { merge_group: { types: string[]; branches: string[] } };
  permissions: Record<string, string>;
  jobs: Record<string, { permissions: Record<string, string>; steps: { run: string }[] }>;
};

void test("the watchdog cancels only a candidate whose queue ref is confirmed gone", () => {
  const workflow = parse(
    readRepoFile(".github", "workflows", "merge-queue-watchdog.yml"),
  ) as Workflow;
  assert.deepEqual(workflow.on, {
    merge_group: { types: ["checks_requested"], branches: ["main"] },
  });
  assert.deepEqual(workflow.permissions, {});
  const job = workflow.jobs.watch;
  assert.deepEqual(job.permissions, { actions: "write", contents: "read" });
  const script = job.steps[0].run;
  // Two consecutive 404s, never a transient error, decide that the ref is gone.
  assert.match(script, /== 404 \]\]; then missing=\$\(\(missing \+ 1\)\); else missing=0; fi/);
  assert.match(script, /if \(\( missing >= 2 \)\); then/);
  // Its own run is excluded, and it ends when the candidate's other runs finish.
  assert.match(script, /select\(\.id != \$GITHUB_RUN_ID and \.status != \\"completed\\"\)/);
  assert.match(script, /\[\[ -n "\$\(others \|\| echo unknown\)" \]\] \|\| exit 0/);
});

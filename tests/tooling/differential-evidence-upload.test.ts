import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  uses?: string;
  run?: string;
  with?: Record<string, string>;
};
type Workflow = { jobs: Record<string, { steps: Step[] }> };
const actionPath = "./.github/actions/upload-formatter-api-corpus-evidence";
const action = parse(readRepoFile(actionPath.slice(2), "action.yml")) as {
  inputs: { shard: { default: string } };
  runs: { using: string; steps: Step[] };
};

test("shared evidence uploader retains every product raw tree and the selected CLI receipt", () => {
  assert.equal(action.runs.using, "composite");
  assert.equal(action.runs.steps.length, 1, "retention must not introduce another build");
  const upload = action.runs.steps[0];
  assert.match(upload.uses ?? "", /^actions\/upload-artifact@[0-9a-f]{40}$/);
  assert.deepEqual(upload.with?.path.trim().split("\n"), [
    "target/differential/",
    "target/ci/vize.differential-build.json",
  ]);
  assert.equal(upload.with?.["if-no-files-found"], "warn");
  assert.equal(upload.run, undefined);
});

test("evidence artifact identity preserves its prefix and separates jobs, shards and attempts", () => {
  assert.equal(action.inputs.shard.default, "full");
  const template = action.runs.steps[0].with?.name ?? "";
  assert.equal(
    template,
    "formatter-api-corpus-${{ github.run_id }}-${{ github.run_attempt }}-${{ github.job }}-${{ inputs.shard }}",
  );
  const identities = new Set<string>();
  for (const run of ["101", "102"]) {
    for (const attempt of ["1", "2"]) {
      for (const job of ["test-scripts", "pr-tooling-scripts"]) {
        for (const shard of ["full", "1", "2"]) {
          const identity = template
            .replace("${{ github.run_id }}", run)
            .replace("${{ github.run_attempt }}", attempt)
            .replace("${{ github.job }}", job)
            .replace("${{ inputs.shard }}", shard);
          assert.ok(!identities.has(identity), `colliding evidence upload ${identity}`);
          identities.add(identity);
        }
      }
    }
  }
  assert.equal(identities.size, 24);
});

test("PR and full tooling gates retain failed observations after their tests", () => {
  const full = parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
  const pr = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as Workflow;
  for (const [job, condition, shard] of [
    [full.jobs["test-scripts"], "${{ always() }}", "full"],
    [
      pr.jobs["pr-tooling-scripts"],
      "${{ always() && needs.pr-source-plan.outputs.tooling == 'true' }}",
      "${{ matrix.index }}",
    ],
  ] as const) {
    const uploads = job.steps.filter((step) => step.uses === actionPath);
    assert.equal(uploads.length, 1);
    assert.equal(uploads[0].if, condition);
    assert.equal(uploads[0].with?.shard ?? action.inputs.shard.default, shard);
    assert.deepEqual(Object.keys(uploads[0].with ?? {}), shard === "full" ? [] : ["shard"]);
    const uploadIndex = job.steps.indexOf(uploads[0]);
    const tests = job.steps.filter((step) => /test:scripts(?::pr)?\b/.test(step.run ?? ""));
    assert.ok(tests.length > 0);
    for (const step of tests) assert.ok(job.steps.indexOf(step) < uploadIndex);
    const build = job.steps.find((step) =>
      /tests\/differential\/build-receipt\.mjs/.test(step.run ?? ""),
    );
    assert.ok(build, "uploaded CLI receipt must be made by the actual selected source build");
    assert.ok(job.steps.indexOf(build) < uploadIndex);
  }
});

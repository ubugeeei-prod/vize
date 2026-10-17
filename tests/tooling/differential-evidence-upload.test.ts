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
type Workflow = {
  jobs: Record<string, { steps: Step[]; needs?: string | string[]; uses?: string; if?: string }>;
};
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
      for (const job of ["test-scripts", "another-worker"]) {
        for (const shard of ["full", "1", "2", "3", "4"]) {
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
  assert.equal(identities.size, 40);
});

test("scheduled and dispatched full tooling retains failed observations after its tests", () => {
  const full = parse(readRepoFile(".github", "workflows", "check.yml")) as Workflow;
  const fullProducer = parse(
    readRepoFile(".github", "workflows", "full-source-cli.yml"),
  ) as Workflow;
  const steps = (
    parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml")) as {
      runs: { steps: Step[] };
    }
  ).runs.steps;
  assert.deepEqual(full.jobs["test-scripts"].needs, ["full-source-cli", "tooling-plan"]);
  assert.equal(
    full.jobs["test-scripts"].if,
    "${{ always() && (github.event_name == 'schedule' || github.event_name == 'workflow_dispatch') }}",
  );
  assert.equal(full.jobs["full-source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  const call = full.jobs["test-scripts"].steps.find(
    (step) => step.uses === "./.github/actions/test-full-tooling",
  );
  assert.deepEqual(call?.with, {
    producer: "${{ toJSON(needs.full-source-cli.outputs) }}",
    shard: "${{ matrix.index }}",
    total: "${{ matrix.total }}",
  });
  const restore = parse(readRepoFile(".github", "actions", "restore-source-cli", "action.yml")) as {
    runs: { steps: Step[] };
  };
  const uploads = steps.filter((step) => step.uses === actionPath);
  assert.equal(uploads.length, 1);
  assert.equal(uploads[0].if, "${{ always() }}");
  assert.deepEqual(uploads[0].with, { shard: "${{ inputs.shard }}" });
  const uploadIndex = steps.indexOf(uploads[0]);
  const tests = steps.filter((step) => /\btest:scripts:planned\b/.test(step.run ?? ""));
  assert.ok(tests.length > 0);
  const build = steps.find((step) => step.uses === "./.github/actions/restore-source-cli");
  assert.ok(build, "uploaded CLI receipt must be made by the actual selected source build");
  assert.deepEqual(build.with, { producer: "${{ inputs.producer }}" });
  for (const step of tests) {
    assert.ok(steps.indexOf(build) < steps.indexOf(step));
    assert.ok(steps.indexOf(step) < uploadIndex);
  }
  const originalBuild = fullProducer.jobs.cli.steps.find((step) =>
    /tests\/differential\/build-receipt\.ts/.test(step.run ?? ""),
  );
  assert.ok(originalBuild, "same-run producer makes the original uploaded receipt");
  assert.match(
    originalBuild.run ?? "",
    /cargo build --profile ci -p vize && vp exec node tests\/differential\/build-receipt\.ts/,
  );
  assert.equal(originalBuild.if, undefined);
  const downloads = restore.runs.steps.filter((step) =>
    step.uses?.startsWith("actions/download-artifact@"),
  );
  assert.equal(downloads.length, 1);
  assert.equal(downloads[0].with?.["digest-mismatch"], "error");
  const restoreRun = restore.runs.steps.at(-1)?.run ?? "";
  assert.match(restoreRun, /source-cli-artifact\.ts restore/);
  assert.match(restoreRun, /cat target\/ci\/vize\.differential-build\.json/);
  assert.doesNotMatch(restoreRun, /build-receipt\.ts/);
});

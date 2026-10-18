import assert from "node:assert/strict";
import { test } from "node:test";
import { parse } from "yaml";
import { checkWorkflowJobSources, readRepoFile } from "./support/github-workflows.ts";

type Step = {
  name?: string;
  if?: string;
  uses?: string;
  run?: string;
  with?: Record<string, string>;
};
type Workflow = {
  jobs: Record<string, { steps: Step[]; needs?: string | string[]; uses?: string }>;
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
      for (const job of ["test-scripts", "pr-native-runtime"]) {
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

test("the full tooling gate retains failed observations after its extracted tests", () => {
  const full = parse(checkWorkflowJobSources()) as Workflow;
  const fullProducer = parse(
    readRepoFile(".github", "workflows", "full-source-cli.yml"),
  ) as Workflow;
  const fullWorker = {
    ...full.jobs["test-scripts"],
    steps: (
      parse(readRepoFile(".github", "actions", "test-full-tooling", "action.yml")) as {
        runs: { steps: Step[] };
      }
    ).runs.steps,
  };
  assert.deepEqual(fullWorker.needs, ["full-source-cli", "tooling-plan"]);
  assert.equal(full.jobs["full-source-cli"].uses, "./.github/workflows/full-source-cli.yml");
  const call = full.jobs["test-scripts"].steps.find(
    (step) => step.uses === "./.github/actions/test-full-tooling",
  );
  assert.ok(call, "Check must invoke the extracted full tooling worker");
  assert.deepEqual(call.with, {
    producer: "${{ toJSON(needs.full-source-cli.outputs) }}",
    shard: "${{ matrix.index }}",
    total: "${{ matrix.total }}",
  });
  const restore = parse(readRepoFile(".github", "actions", "restore-source-cli", "action.yml")) as {
    runs: { steps: Step[] };
  };
  const job = fullWorker;
  const condition = "${{ always() }}";
  const shard = "${{ inputs.shard }}";
  const uploads = job.steps.filter((step) => step.uses === actionPath);
  assert.equal(uploads.length, 1);
  assert.equal(uploads[0].if, condition);
  assert.equal(uploads[0].with?.shard ?? action.inputs.shard.default, shard);
  assert.deepEqual(Object.keys(uploads[0].with ?? {}), ["shard"]);
  const uploadIndex = job.steps.indexOf(uploads[0]);
  const tests = job.steps.filter((step) => /test:scripts(?::pr)?\b/.test(step.run ?? ""));
  assert.ok(tests.length > 0);
  for (const step of tests) assert.ok(job.steps.indexOf(step) < uploadIndex);
  const build = job.steps.find((step) => step.uses === "./.github/actions/restore-source-cli");
  assert.ok(build, "uploaded CLI receipt must be made by the actual selected source build");
  assert.ok(job.steps.indexOf(build) < uploadIndex);
  assert.deepEqual(build.with, { producer: "${{ inputs.producer }}" });
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

// #9198 retired the broad PR tooling shard. The live PR path still executes
// selected native product actions and remains required by both report levels.
test("the current PR native runtime lane remains connected to required source and Check reports", () => {
  const full = parse(checkWorkflowJobSources()) as Workflow;
  const pr = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as Workflow;
  assert.equal(full.jobs["pr-source-checks"].uses, "./.github/workflows/pr-source-checks.yml");
  const outerNeeds = full.jobs["test-report"].needs;
  assert.ok(Array.isArray(outerNeeds));
  assert.ok(outerNeeds.includes("pr-source-checks"));
  const sourceNeeds = pr.jobs["source-report"].needs;
  assert.ok(Array.isArray(sourceNeeds));
  assert.ok(sourceNeeds.includes("pr-native-runtime"));
  const runtime = pr.jobs["pr-native-runtime"];
  assert.ok(runtime);
  assert.equal(runtime.needs, "pr-source-plan");
  assert.deepEqual(
    runtime.steps
      .filter((step) => step.uses?.startsWith("./.github/actions/test-native-"))
      .map((step) => step.uses),
    [
      "./.github/actions/test-native-canon",
      "./.github/actions/test-native-lsp",
      "./.github/actions/test-native-ssr",
      "./.github/actions/test-native-scoped-css",
      "./.github/actions/test-native-js-setup",
      "./.github/actions/test-native-vapor",
      "./.github/actions/test-native-attribute-values-7502",
      "./.github/actions/test-native-selected-template",
    ],
  );
});

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

import { hostVersions, parseHostOptions } from "../../npm/oxlint/scripts/ci-host-concurrency.mjs";

const action = parse(
  readFileSync(
    new URL("../../.github/actions/test-js-packages-with-history/action.yml", import.meta.url),
    "utf8",
  ),
);
const workflow = parse(
  readFileSync(
    new URL("../../.github/workflows/ci-walltime-benchmark.yml", import.meta.url),
    "utf8",
  ),
);
const pullRequest = parse(
  readFileSync(new URL("../../.github/workflows/pr-js-checks.yml", import.meta.url), "utf8"),
);
const controller = "node npm/oxlint/scripts/check-project-transport.mjs";
const selector = new URL("../../npm/oxlint/scripts/n8n-baseline-partitions.mjs", import.meta.url)
  .href;

function observedMode(...overlays) {
  const environment = { ...process.env };
  delete environment.VIZE_N8N_BASELINE_MODE;
  for (const overlay of overlays) Object.assign(environment, overlay ?? {});
  const child = spawnSync(
    process.execPath,
    [
      "--input-type=module",
      "-e",
      `import { baselineMode } from ${JSON.stringify(selector)}; process.stdout.write(baselineMode());`,
    ],
    { env: environment, encoding: "utf8", timeout: 5000 },
  );
  assert.equal(child.error, undefined);
  assert.equal(child.signal, null);
  assert.equal(child.status, 0, child.stderr);
  assert.equal(child.stderr, "");
  return child.stdout;
}

void test("only the CI qualification steps opt into fixed baseline partitions", () => {
  const selected = action.runs.steps.filter((step) => step.id === "original-project-qualification");
  assert.equal(selected.length, 1);
  const qualification = selected[0];
  assert.deepEqual(qualification.env, { VIZE_N8N_BASELINE_MODE: "partitioned" });
  assert.equal(qualification.run, controller);
  assert.equal(qualification.shell, "bash");
  assert.equal(
    qualification.if,
    "${{ success() && job.status == 'success' && (inputs.partition == 'all' || inputs.partition == 'original-project') }}",
  );
  assert.equal(qualification["continue-on-error"], undefined);
  assert.equal(action.env?.VIZE_N8N_BASELINE_MODE, undefined);
  for (const step of action.runs.steps) {
    if (step !== qualification) assert.equal(step.env?.VIZE_N8N_BASELINE_MODE, undefined);
  }
  assert.equal(observedMode(qualification.env), "partitioned");
  // PR CI runs the same qualification step in its own job.
  assert.equal(pullRequest.env?.VIZE_N8N_BASELINE_MODE, undefined);
  const pullRequestSteps = Object.values(pullRequest.jobs).flatMap((job) => {
    assert.equal(job.env?.VIZE_N8N_BASELINE_MODE, undefined);
    return job.steps;
  });
  const pullRequestQualification = pullRequestSteps.filter(
    (step) => step.id === "original-project-qualification",
  );
  assert.equal(pullRequestQualification.length, 1);
  assert.deepEqual(pullRequestQualification[0].env, qualification.env);
  assert.equal(pullRequestQualification[0].run, controller);
  assert.equal(pullRequestQualification[0]["continue-on-error"], undefined);
  for (const step of pullRequestSteps) {
    if (step !== pullRequestQualification[0])
      assert.equal(step.env?.VIZE_N8N_BASELINE_MODE, undefined);
  }
  assert.equal(parseHostOptions([], {}).mode, "concurrent");
  assert.equal(parseHostOptions([], {}).workers, 2);
  assert.equal(hostVersions.length, 2);
});

void test("local and original manual comparison environments retain the serial default", () => {
  assert.equal(observedMode(), "serial");
  assert.equal(workflow.env?.VIZE_N8N_BASELINE_MODE, undefined);
  const manual = workflow.jobs.js;
  const arms = manual.steps.filter((step) => step.run?.startsWith(controller));
  assert.equal(arms.length, 2);
  assert.deepEqual(
    arms.map((step) => step.run),
    [
      `${controller} --host-mode serial --phase-walltime target/ci-walltime/js-serial.json`,
      `${controller} --host-mode concurrent --phase-walltime target/ci-walltime/js-concurrent.json`,
    ],
  );
  for (const step of arms) {
    assert.equal(observedMode(workflow.env, manual.env, step.env), "serial");
  }
});

void test("the explicit manual baseline pair still selects its two original modes", () => {
  const manual = workflow.jobs["n8n-baseline"];
  const arms = manual.steps.filter((step) => step.run?.startsWith(controller));
  assert.equal(arms.length, 2);
  assert.deepEqual(
    arms.map((step) => step.env),
    [{ VIZE_N8N_BASELINE_MODE: "serial" }, { VIZE_N8N_BASELINE_MODE: "partitioned" }],
  );
  assert.deepEqual(
    arms.map((step) => observedMode(workflow.env, manual.env, step.env)),
    ["serial", "partitioned"],
  );
  assert.deepEqual(
    arms.map((step) => step.run),
    [
      `${controller} --host-mode concurrent --phase-walltime target/ci-walltime/n8n-baseline-serial.json`,
      `${controller} --host-mode concurrent --phase-walltime target/ci-walltime/n8n-baseline-partitioned.json`,
    ],
  );
});

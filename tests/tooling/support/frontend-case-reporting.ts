import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";

type Step = { name?: string; run?: string; [key: string]: unknown };
type Job = { steps?: Step[]; [key: string]: unknown };

function selectedStep(job: Job, name: string): { index: number; step: Step } {
  const matches = (job.steps ?? []).flatMap((step, index) =>
    step.name === name ? [{ index, step }] : [],
  );
  assert.equal(matches.length, 1, name);
  return matches[0];
}

export function restoreHistoryToolchainSelection(job: Job, original: Job) {
  assert.equal(
    createHash("sha256")
      .update(fs.readFileSync(new URL("../native-history-toolchain.test.ts", import.meta.url)))
      .digest("hex"),
    "da230fd0bc5a8d51005578d8c2d1c437fc3b0bd85216c6c6cdc952928bc8c3c0",
  );
  const previous = selectedStep(original, "Test CI selection when its inputs change").step;
  const previousRun = previous.run;
  assert(typeof previousRun === "string");
  const marker = "tests/tooling/native-history-profile.test.ts";
  assert.equal(previousRun.split(marker).length, 2);
  const reviewed = previousRun.replace(
    marker,
    `${marker} tests/tooling/native-history-toolchain.test.ts`,
  );
  const current = selectedStep(job, "Test CI selection when its inputs change").step;
  assert.equal(current.id, "cases");
  assert.deepEqual(current.env, { TEST_CASES_JOB: "CI selection tests" });
  const reporter =
    'node --test --test-reporter=spec --test-reporter-destination=stdout --test-reporter=./tools/support/compat/github/test-case-reporter.ts --test-reporter-destination="$RUNNER_TEMP/test-cases-ci-selection.json" ';
  const currentRun = current.run;
  assert(typeof currentRun === "string");
  assert.equal(currentRun.replace(reporter, "node --test "), reviewed);
  current.run = previousRun;
  delete current.id;
  delete current.env;
  assert.deepEqual(current, previous);
  const steps = job.steps;
  assert(steps);
  const upload = selectedStep(job, "Upload CI selection test cases");
  assert.equal(upload.index, steps.indexOf(current) + 1);
  assert.deepEqual(upload.step, {
    name: "Upload CI selection test cases",
    if: "${{ !cancelled() && steps.cases.outcome != 'skipped' }}",
    uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
    with: {
      name: "test-cases-ci-selection-${{ github.run_attempt }}",
      path: "${{ runner.temp }}/test-cases-ci-selection.json",
      "if-no-files-found": "error",
      "retention-days": 7,
    },
  });
  steps.splice(upload.index, 1);
}

function restoreSmallNodeAction(
  job: Job,
  original: Job,
  name: string,
  withInputs: Record<string, string>,
) {
  const current = selectedStep(job, name);
  const previous = selectedStep(original, name).step;
  const previousRun = previous.run;
  assert(typeof previousRun === "string");
  const { run: _run, ...unchanged } = previous;
  assert.deepEqual(current.step, {
    ...unchanged,
    uses: "./.github/actions/test-small-node-cases",
    with: withInputs,
  });
  assert(job.steps);
  job.steps[current.index] = structuredClone(previous);
}

export function restoreCheckJsCasePacket(job: Job, original: Job) {
  restoreSmallNodeAction(job, original, "Check fast JS/TS", {
    suite: "check-js",
    job: "check-js",
    packet: "check-js",
  });
}

export function restoreNodeEngineCasePacket(job: Job, original: Job) {
  restoreSmallNodeAction(job, original, "Check package engine matrix", {
    suite: "node-engine",
    job: "node-engine-compat (${{ matrix.node-version }})",
    packet: "node-engine-${{ matrix.node-version }}",
  });
}

export function restoreNativeRuntimeCasePacket(job: Job, original: Job) {
  assert.deepEqual(job.env, {
    ...(original.env as Record<string, string>),
    TEST_CASES_JOB: "Native runtime (${{ matrix.lane }})",
    VIZE_TEST_CASES: "test-cases-native-${{ matrix.lane }}.json",
  });
  job.env = structuredClone(original.env);
  const steps = job.steps;
  assert(steps);
  const additions = [
    {
      name: "Prepare native runtime test case capture",
      if: "${{ matrix.lane != 'none' }}",
      run: 'node tools/support/compat/github/job-test-cases.ts setup "$RUNNER_TEMP/native-test-cases-${{ matrix.lane }}"',
    },
    {
      name: "Collect and upload native runtime test cases",
      if: "${{ !cancelled() && matrix.lane != 'none' }}",
      uses: "./.github/actions/collect-job-test-cases",
      with: {
        artifact: "test-cases-native-${{ matrix.lane }}-${{ github.run_attempt }}",
      },
    },
  ];
  for (const expected of additions) {
    const selected = selectedStep(job, expected.name);
    assert.deepEqual(selected.step, expected);
    steps.splice(selected.index, 1);
  }
  assert.deepEqual(job, original);
}

import assert from "node:assert/strict";

type Step = { name?: string; id?: string; if?: string; run?: string; [key: string]: unknown };
type Job = { steps?: Step[]; [key: string]: unknown };
type Workflow = { jobs: Record<string, Job>; [key: string]: unknown };

// #11690 records the existing browser command and executes VRT outside the
// task cache so its case receipt remains available to the following step.
export function restorePlaygroundCaseReporting(workflow: Workflow, original: Workflow) {
  const target = workflow.jobs["pr-playground-test"];
  const before = original.jobs["pr-playground-test"];
  const steps = target.steps ?? [];
  const previous = before.steps ?? [];
  const browserIndex = steps.findIndex((step) => step.name === "Run playground tests");
  const previousBrowser = previous.find((step) => step.name === "Run playground tests");
  assert(browserIndex >= 0 && previousBrowser);
  assert.deepEqual(steps[browserIndex], {
    ...previousBrowser,
    id: "browser-cases",
    env: {
      TEST_CASES_JOB: "PR playground browser tests",
      VIZE_TEST_CASES: "${{ runner.temp }}/test-cases-playground-browser.json",
    },
    run: "node tools/support/compat/github/js-test-cases.ts capture -- vp run --filter './playground' test:browser",
  });
  steps[browserIndex] = structuredClone(previousBrowser);

  const vrtIndex = steps.findIndex((step) => step.name === "Run VRT");
  const previousVrt = previous.find((step) => step.name === "Run VRT");
  assert(vrtIndex >= 0 && previousVrt);
  const currentVrt = { ...previousVrt };
  assert.equal(currentVrt["continue-on-error"], true);
  delete currentVrt["continue-on-error"];
  assert.deepEqual(steps[vrtIndex], {
    ...currentVrt,
    env: { VIZE_PLAYWRIGHT_CASES: "e2e/vrt/test-results/case-report.json" },
    run: "vp exec --filter './playground' -- playwright test",
  });
  steps[vrtIndex] = structuredClone(previousVrt);

  const collect = steps[vrtIndex + 1];
  const upload = steps[vrtIndex + 2];
  assert.equal(collect?.name, "Collect playground test cases");
  assert.equal(collect?.id, "cases");
  assert.equal(collect?.if, "${{ !cancelled() && env.RUN_PLAYGROUND == 'true' }}");
  assert.deepEqual(collect?.env, {
    TEST_CASES_JOB: "PR playground browser tests",
    RUN_VRT: "${{ github.event_name == 'merge_group' }}",
    VRT_OUTCOME: "${{ steps.vrt.outcome }}",
  });
  assert.match(String(collect?.run), /playground\/e2e\/vrt\/test-results\/case-report\.json/u);
  assert.match(String(collect?.run), /successful VRT produced no Playwright case receipt/u);
  assert.match(String(collect?.run), /test-cases\.ts playwright/);
  assert.match(String(collect?.run), /test-cases\.ts merge/);
  assert.deepEqual(upload, {
    name: "Upload playground test cases",
    if: "${{ !cancelled() && steps.cases.outcome != 'skipped' }}",
    uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
    with: {
      name: "test-cases-playground-${{ github.run_attempt }}",
      path: "${{ runner.temp }}/test-cases-playground.json",
      "if-no-files-found": "error",
      "retention-days": 7,
    },
  });
  steps.splice(vrtIndex + 1, 2);
  // Reconstruct the original reporting sequence after verifying each status transition.
  for (const name of ["Upload VRT report", "Upload VRT diff artifacts"]) {
    const index = steps.findIndex((step) => step.name === name);
    const old = previous.find((step) => step.name === name);
    assert.ok(index >= 0 && old);
    assert.deepEqual(steps[index], { ...old, if: "failure() && steps.vrt.outcome == 'failure'" });
    steps[index] = structuredClone(old);
  }
  const originalGate = previous.at(-1);
  assert.equal(originalGate?.name, "Fail if VRT failed");
  assert.equal(originalGate.run, "exit 1");
  assert.equal(
    originalGate.if,
    "${{ github.event_name == 'merge_group' && env.RUN_PLAYGROUND == 'true' && steps.vrt.outcome == 'failure' }}",
  );
  steps.push(structuredClone(originalGate));
  assert.deepEqual(target, before);
  workflow.jobs["pr-playground-test"] = structuredClone(before);
}

export function restoreFullPlaygroundVrtGate(job: Job) {
  const steps = job.steps;
  assert(steps);
  const index = steps.length - 3;
  const [vrt, report, diff] = steps.slice(index);
  assert.deepEqual(vrt, {
    name: "Run VRT",
    run: "vp run --filter './playground' test:vrt",
    id: "vrt",
  });
  steps[index] = { name: vrt.name, run: vrt.run, "continue-on-error": true, id: vrt.id };
  assert.deepEqual([report.name, diff.name], ["Upload VRT report", "Upload VRT diff artifacts"]);
  const previousCondition = "steps.vrt.outcome == 'failure'";
  for (const upload of [report, diff]) {
    assert.equal(upload.if, "failure() && " + previousCondition);
    upload.if = previousCondition;
  }
  steps.push({ name: "Fail if VRT failed", if: previousCondition, run: "exit 1" });
}

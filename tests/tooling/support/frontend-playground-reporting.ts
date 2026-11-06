import assert from "node:assert/strict";

type Step = { name?: string; id?: string; if?: string; run?: string; [key: string]: unknown };
type Job = { steps?: Step[]; [key: string]: unknown };
type Workflow = { jobs: Record<string, Job>; [key: string]: unknown };

// #11690 records the existing browser and VRT commands without changing their
// runner contract. Validate and invert only those reporting additions.
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
  assert.deepEqual(steps[vrtIndex], {
    ...previousVrt,
    env: { VIZE_PLAYWRIGHT_CASES: "${{ runner.temp }}/test-cases-playground-vrt-report.json" },
  });
  steps[vrtIndex] = structuredClone(previousVrt);

  const collect = steps[vrtIndex + 1];
  const upload = steps[vrtIndex + 2];
  assert.equal(collect?.name, "Collect playground test cases");
  assert.equal(collect?.id, "cases");
  assert.equal(collect?.if, "${{ !cancelled() && env.RUN_PLAYGROUND == 'true' }}");
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
  assert.deepEqual(target, before);
  workflow.jobs["pr-playground-test"] = structuredClone(before);
}

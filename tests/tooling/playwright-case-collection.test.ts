import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { parse } from "yaml";
import { mergePackets, playwrightCases } from "../../tools/support/compat/github/test-cases.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

type Step = {
  id?: string;
  name?: string;
  env?: Record<string, string>;
  with?: Record<string, string>;
  run?: string;
};

test("the playground job uploads browser and visual test cases", () => {
  const workflow = parse(readRepoFile(".github", "workflows", "pr-source-checks.yml")) as {
    jobs: Record<string, { name?: string; steps: Step[] }>;
  };
  const playground = workflow.jobs["pr-playground-test"];
  const browser = playground.steps.find((step) => step.name === "Run playground tests");
  assert.equal(browser?.env?.TEST_CASES_JOB, playground.name);
  assert.ok(browser?.run?.includes("js-test-cases.ts capture"));
  const collect = playground.steps.find((step) => step.name === "Collect playground test cases");
  assert.equal(collect?.id, "cases");
  assert.equal(collect?.env?.TEST_CASES_JOB, playground.name);
  assert.equal(collect?.env?.VRT_OUTCOME, "${{ steps.vrt.outcome }}");
  assert.match(collect?.run ?? "", /VRT_OUTCOME.*success.*-s/u);
  assert.ok(collect?.run?.includes("test-cases.ts playwright"));
  assert.ok(collect?.run?.includes("test-cases.ts merge"));
  const upload = playground.steps[playground.steps.indexOf(collect ?? {}) + 1];
  assert.equal(upload.with?.name, "test-cases-playground-${{ github.run_attempt }}");
  assert.equal(upload.with?.["if-no-files-found"], "error");
});

test("Playwright retries and projects become mergeable playground cases", () => {
  const report = {
    config: { rootDir: path.join(root, "playground", "e2e", "vrt") },
    suites: [
      {
        title: "editor",
        specs: [
          {
            file: "editor.spec.ts",
            title: "renders",
            tests: [
              {
                projectName: "chromium",
                expectedStatus: "passed",
                results: [{ status: "failed" }, { status: "passed" }],
              },
              { projectName: "webkit", expectedStatus: "skipped", results: [] },
              { projectName: "firefox", expectedStatus: "passed", results: [] },
            ],
          },
        ],
        suites: [],
      },
    ],
  };
  const cases = playwrightCases(report, root);
  assert.deepEqual(cases, [
    {
      path: "playground/e2e/vrt/editor.spec.ts",
      name: "playground/e2e/vrt/editor.spec.ts › editor › renders › chromium",
      outcome: "passed",
    },
    {
      path: "playground/e2e/vrt/editor.spec.ts",
      name: "playground/e2e/vrt/editor.spec.ts › editor › renders › webkit",
      outcome: "skipped",
    },
  ]);
  assert.deepEqual(
    mergePackets([
      { version: 1, job: "PR playground browser tests", attempt: 3, cases: [cases[0]] },
      { version: 1, job: "PR playground browser tests", attempt: 3, cases: [cases[1]] },
    ]),
    { version: 1, job: "PR playground browser tests", attempt: 3, cases },
  );
  assert.throws(
    () =>
      mergePackets([
        { version: 1, job: "one", attempt: 1, cases: [] },
        { version: 1, job: "two", attempt: 1, cases: [] },
      ]),
    /same job and attempt/,
  );
});

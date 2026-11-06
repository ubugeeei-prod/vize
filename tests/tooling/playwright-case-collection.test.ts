import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";
import { mergePackets, playwrightCases } from "../../tools/support/compat/github/test-cases.ts";
import { root } from "./support/github-workflows.ts";

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

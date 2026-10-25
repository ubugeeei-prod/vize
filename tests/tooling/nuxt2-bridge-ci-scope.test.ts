import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { matchesGlob } from "node:path";
import { test } from "node:test";
import { parse } from "yaml";

import { localImportInputs } from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const file = ".github/workflows/nuxt2-bridge-module-build.yml";
const workflow = parse(readFileSync(new URL(`../../${file}`, import.meta.url), "utf8")) as {
  on: Record<string, { paths: string[] }>;
  jobs: Record<string, { steps: Array<{ uses?: string }> }>;
};

void test("both Bridge event filters reach real host helper imports and referenced local actions", () => {
  const helpers = localImportInputs("tools/support/compat/nuxt/nuxt2-module-build.ts").files;
  const actions = Object.values(workflow.jobs).flatMap(({ steps }) =>
    steps.flatMap(({ uses }) => (uses?.startsWith("./") ? [`${uses.slice(2)}/action.yml`] : [])),
  );
  assert.ok(helpers.includes("tools/support/compat/nuxt/host-types.ts"));
  assert.ok(helpers.includes("tools/support/json-value.ts"));
  assert.ok(actions.length > 0, "the actual workflow retains local setup actions");
  for (const event of ["pull_request", "push"])
    for (const input of [file, ...helpers, ...actions])
      assert.ok(
        workflow.on[event].paths.some((pattern) => matchesGlob(input, pattern)),
        `${event}: ${input}`,
      );
});

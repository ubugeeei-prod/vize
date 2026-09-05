import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  cleanup,
  divergenceClassification,
  readJson,
  root,
  run,
  setup,
  unusableFailure,
} from "./_helpers/typecheck-divergence-report-fixture.ts";

/**
 * Run 31979524200, end to end: the negative control for the ambient gate.
 *
 * Everything about the run below is what run 31979524200 had and what the
 * previous checks pass on — one diagnostic per side at the same span, the same
 * single Vue file on both sides, a clean configuration and a passing mutation
 * oracle. The only difference between the two tests is one extra line in the
 * `--listFilesOnly` program listing: a second copy of `vue`, resolved above the
 * fixture where Vize keeps its own `node_modules`.
 *
 * That one line is the whole elk failure. It is what turns the fixture's
 * `declare module 'vue'` augmentations into augmentations of a module its
 * components are not typed against, and before this gate it produced a green
 * "Vize divergence, the vue-tsc baseline loaded cleanly over the same N Vue
 * files" verdict on the way to scoring 894 phantom false negatives.
 */

/** Where a fixture's own package manager puts it: inside the fixture. */
const fixtureVue = "node_modules/.pnpm/vue@3.5.30/node_modules/vue/dist/vue.d.mts";
/**
 * Where the resolver goes when a `/// <reference types="..." />` escapes the
 * fixture. `setup` roots the fixture at `tests/_fixtures/<tmp>`, so this is the
 * repository's own `node_modules`, three levels up — the same relationship elk's
 * fixture at `tests/_fixtures/_git/elk` has to Vize's install.
 */
const vizeVue = path.join(
  root,
  "node_modules/.pnpm/vue@3.6.0-beta.10/node_modules/vue/dist/vue.d.mts",
);

test("a fixture typed against its own Vue runtime is measured, not rejected", () => {
  const fixture = setup({ baselineProgramFiles: [fixtureVue] });
  try {
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);
    const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
    assert.deepEqual(artifact.baseline.ambient, {
      externalFileCount: 0,
      externalPackages: [],
      unusableReason: null,
      verdict: "isolated",
      vueRuntime: [
        {
          name: "vue",
          copies: [{ path: "node_modules/.pnpm/vue@3.5.30/node_modules/vue", insideFixture: true }],
          insideFixtureCount: 1,
          outsideFixtureCount: 0,
        },
      ],
    });
    assert.equal(artifact.budget.verdict, "passed");
    assert.equal(artifact.budget.passed, true);
    assert.equal(artifact.budget.unusableReason, null);
  } finally {
    cleanup(fixture);
  }
});

test("fixture-local Vue runtimes are pinned into the baseline config", () => {
  const fixture = setup();
  try {
    for (const name of ["vue", "@vue/runtime-core", "@vue/runtime-dom"]) {
      const packageRoot = path.join(fixture.fixtureRoot, "node_modules", ...name.split("/"));
      fs.mkdirSync(packageRoot, { recursive: true });
      fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
    }
    fs.writeFileSync(
      path.join(fixture.fixtureRoot, "tsconfig.json"),
      `${JSON.stringify({
        compilerOptions: {
          baseUrl: ".",
          paths: {
            "#app": ["src/app"],
          },
        },
      })}\n`,
    );
    const result = run(fixture);
    assert.equal(result.status, 0, result.stderr);

    const config = readJson(path.join(fixture.reportDir, "fixture-vue-tsc.tsconfig.json"));
    assert.equal(config.compilerOptions.baseUrl, ".");
    assert.deepEqual(config.compilerOptions.paths, {
      "#app": ["../src/app"],
      "@vue/runtime-core": ["../node_modules/@vue/runtime-core"],
      "@vue/runtime-dom": ["../node_modules/@vue/runtime-dom"],
      vue: ["../node_modules/vue"],
    });
  } finally {
    cleanup(fixture);
  }
});

test("a second Vue runtime resolved above the fixture sinks the run", () => {
  const fixture = setup({ baselineProgramFiles: [fixtureVue, vizeVue] });
  try {
    const result = run(fixture);
    const reason =
      "vue-tsc resolved 2 copies of 'vue' into the baseline program " +
      "(../../../node_modules/.pnpm/vue@3.6.0-beta.10/node_modules/vue, " +
      "node_modules/.pnpm/vue@3.5.30/node_modules/vue), so the fixture's own module " +
      "augmentations merged into a different module identity than its components";
    assert.equal(result.status, 1);
    assert.equal(result.stderr, `${unusableFailure(reason)}\n`);

    const artifact = readJson(path.join(fixture.reportDir, "fixture-typecheck-divergence.json"));
    assert.deepEqual(artifact.baseline.ambient, {
      externalFileCount: 1,
      externalPackages: ["vue"],
      unusableReason: reason,
      verdict: "contaminated",
      vueRuntime: [
        {
          name: "vue",
          copies: [
            {
              path: "../../../node_modules/.pnpm/vue@3.6.0-beta.10/node_modules/vue",
              insideFixture: false,
            },
            { path: "node_modules/.pnpm/vue@3.5.30/node_modules/vue", insideFixture: true },
          ],
          insideFixtureCount: 1,
          outsideFixtureCount: 1,
        },
      ],
    });
    // The corpus checks stay green — that is the point. They are what let the
    // elk run call a broken instrument a Vize divergence.
    assert.equal(artifact.baseline.configuration.errorCount, 0);
    assert.equal(artifact.baseline.coverage.verdict, "usable");
    assert.equal(artifact.baseline.coverage.sharedVueFileCount, 1);
    assert.equal(artifact.budget.verdict, "unusable");
    assert.equal(artifact.budget.passed, false);
    assert.equal(artifact.budget.unusableReason, reason);

    const markdown = readMarkdown(fixture);
    assert.ok(markdown.includes(`vue-tsc ambient environment: contaminated (${reason})`));
    assert.ok(markdown.includes("Classification: instrument failure"));
    assert.ok(!markdown.includes(`Classification: ${divergenceClassification(1)}`));
  } finally {
    cleanup(fixture);
  }
});

function readMarkdown(fixture: ReturnType<typeof setup>) {
  return fs.readFileSync(path.join(fixture.reportDir, "fixture-typecheck-divergence.md"), "utf8");
}

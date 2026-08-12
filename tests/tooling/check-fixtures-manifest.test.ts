import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  CHECK_FIXTURE_ENV,
  CHECK_FIXTURE_NODE_ARGS,
  checkFixturePhases,
} from "./support/check-fixtures/manifest.ts";
import { readRepoFile, root } from "./support/github-workflows.ts";

/**
 * The exact file list, in the exact order, that `test:check:fixtures` ran as a
 * single `node --test` shell string before the supervisor owned it (#4126).
 *
 * Pinned here rather than derived, so moving the lane to a manifest can never
 * silently drop, reorder, or quietly re-enable a fixture: the order is
 * load-bearing (`zz-intentional-errors-fixtures` repairs state the earlier
 * fixtures plant, and `compat-ratchet` reads the ledger they leave).
 *
 * A fixture added to the lane on `main` after this branch was cut belongs here
 * in the position the shell string gave it, so the pin keeps meaning "every
 * fixture the lane runs" rather than drifting into a stale historical list:
 * `nuxt-template-globals` arrived that way with #4248.
 */
const PHASE_FILES_BEFORE_SUPERVISION = [
  "snapshots/check/typecheck-errors.ts",
  "snapshots/check/typecheck-vue-imports.ts",
  "snapshots/check/compiler-macros.ts",
  "snapshots/check/style-preprocessors.ts",
  "snapshots/check/ecosystem-products.ts",
  "snapshots/check/typescript-go-module-resolution-determinism.ts",
  "snapshots/check/generic-build.ts",
  "snapshots/check/nuxt-parity.ts",
  "snapshots/check/toolchain-parity.ts",
  "snapshots/check/options-api.ts",
  "snapshots/check/class-component.ts",
  "snapshots/check/class-component-lsp-oracle.ts",
  "snapshots/check/create-vue-patch-oracle.ts",
  "snapshots/check/create-vue-generated-template-oracle.ts",
  "snapshots/check/create-vue-editor-range-oracle.ts",
  "snapshots/check/vue-benchmarks-correctness-plants.ts",
  "snapshots/check/javascript-sfc-checkjs-oracle.ts",
  "snapshots/check/vue-benchmarks-lsp-ref-unwrap-oracle.ts",
  "snapshots/check/vue-benchmarks-scaled-corpus-plants.ts",
  "snapshots/check/vue-router-patch-oracle.ts",
  "snapshots/check/vue-router-formatter-oracle.ts",
  "snapshots/check/pinia-generic-store-oracle.ts",
  "snapshots/check/typescript-project-references-oracle.ts",
  "snapshots/check/vue-router-dmts-oracle.ts",
  "snapshots/check/element-plus-slot-oracle.ts",
  "snapshots/check/nuxt-ui-ambient-oracle.ts",
  "snapshots/check/nuxt-no-tsconfig-oracle.ts",
  "snapshots/check/nuxt-template-globals.ts",
  "snapshots/check/vitepress-theme-oracle.ts",
  "snapshots/check/vue-element-admin-legacy-oracle.ts",
  "snapshots/check/vue-element-admin-legacy-lsp-oracle.ts",
  "snapshots/check/vue-element-admin-unmapped-diagnostic-oracle.ts",
  "snapshots/check/vue2-elm.ts",
  "snapshots/check/vue2-class-component-oracle.ts",
  "snapshots/check/zz-intentional-errors-fixtures.ts",
  "tooling/compat-ratchet.test.ts",
];

test("the phase manifest carries every fixture the shell string ran, in order", () => {
  assert.deepEqual(
    checkFixturePhases.map((phase) => phase.file),
    PHASE_FILES_BEFORE_SUPERVISION,
  );
  assert.equal(checkFixturePhases.length, 36);
});

test("every phase has a unique id and a file that exists", () => {
  const ids = checkFixturePhases.map((phase) => phase.id);
  assert.equal(new Set(ids).size, ids.length, "phase ids identify phases in telemetry");
  assert.deepEqual(ids.slice(0, 3), [
    "typecheck-errors",
    "typecheck-vue-imports",
    "compiler-macros",
  ]);
  assert.equal(ids.at(-1), "compat-ratchet");
  for (const phase of checkFixturePhases) {
    assert.ok(
      fs.existsSync(path.join(root, "tests", phase.file)),
      `${phase.file} must exist under tests/`,
    );
  }
});

// The runner options were part of the shell string and are just as load-bearing
// as the file list: `--test-concurrency=1` keeps fixtures off each other's
// materialized projects, and `VIZE_TEST_REQUIRE_TSGO=1` makes the typecheck
// oracles fail closed rather than skip when the Corsa runtime is missing.
test("the manifest keeps the runner options the shell string passed", () => {
  assert.deepEqual([...CHECK_FIXTURE_NODE_ARGS], ["--test", "--test-concurrency=1"]);
  assert.deepEqual({ ...CHECK_FIXTURE_ENV }, { VIZE_TEST_REQUIRE_TSGO: "1" });
});

test("the fixture scripts delegate to the supervisor and the cycle harness", () => {
  const scripts = (
    JSON.parse(readRepoFile("tests", "package.json")) as { scripts: Record<string, string> }
  ).scripts;

  assert.equal(scripts["test:check:fixtures"], "node tooling/support/check-fixtures/supervisor.ts");
  assert.equal(
    scripts["test:check:fixtures:cycles"],
    "node tooling/support/check-fixtures/cycles.ts",
  );
  // A second enumeration of the fixtures would be a second source of truth, and
  // the two would drift; the manifest is the only list.
  assert.doesNotMatch(scripts["test:check:fixtures"]!, /snapshots\/check\//);
});

import assert from "node:assert/strict";
import { test } from "node:test";

import { CONFIG_FREE_PROJECT_SHAPES } from "../../../tools/support/compat/npm/smoke-release-init-config-free.mjs";
import {
  FRESH_INIT_MATRIX,
  PROJECT_SHAPES,
} from "../../../tools/support/compat/npm/smoke-release-init-shapes.mjs";
import { PACKAGE_MANAGERS } from "../../../tools/support/compat/npm/smoke-release-init-managers.mjs";
import { readRepoFile } from "../support/github-workflows.ts";
import { expectedInitOutput } from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";

const CHANGED_FIELDS = new Set([
  "features",
  "reconfiguredDetection",
  "reconfiguredFeatures",
  "createdFiles",
  "expectedFiles",
]);
const PEERS = { typescript: "6.0.3", vite: "8.0.0", "vite-plus": "0.1.0", vue: "3.5.0" };

test("current initializer changes only the declared generated-plan capability", () => {
  assert.deepEqual(Object.keys(CONFIG_FREE_PROJECT_SHAPES), Object.keys(PROJECT_SHAPES));
  for (const [id, historical] of Object.entries(PROJECT_SHAPES)) {
    const current = CONFIG_FREE_PROJECT_SHAPES[id];
    assert.deepEqual(Object.keys(current), Object.keys(historical), id);
    for (const [key, value] of Object.entries(historical)) {
      if (!CHANGED_FIELDS.has(key)) assert.deepEqual(current[key], value, `${id}.${key}`);
    }
    assert.deepEqual(current.files(PEERS), historical.files(PEERS), `${id}: whole authored inputs`);
    assert.deepEqual(current.check, historical.check, `${id}: whole diagnostic triples`);
    assert.ok(historical.createdFiles.includes("vize.config.ts"), `${id}: archive retained`);
    assert.ok(historical.expectedFiles["vize.config.ts"], `${id}: whole archive retained`);
    const { "vize.config.ts": originalConfig, ...retainedFiles } = historical.expectedFiles;
    assert.equal(typeof originalConfig, "string");
    assert.deepEqual(current.expectedFiles, retainedFiles, `${id}: all other complete files`);
    assert.deepEqual(
      current.createdFiles,
      id === "vite-vue-js-checkjs"
        ? ["tsconfig.json", ".vscode/extensions.json"]
        : [".vscode/extensions.json"],
    );
    assert.deepEqual(current.features, [
      "  lint      skipped    not selected",
      `  bundler   configured adds vize() to vite.config.${id === "vite-vue-js-checkjs" ? "js" : "ts"}`,
      "  fmt       configured uses project settings and formatter defaults",
      id === "vite-vue-js-checkjs"
        ? "  typecheck configured writes tsconfig.json"
        : "  typecheck configured uses tsconfig.json and project settings",
      "  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize",
    ]);
    assert.deepEqual(current.reconfiguredFeatures, [
      "  lint      skipped    not selected",
      `  bundler   unchanged  vite.config.${id === "vite-vue-js-checkjs" ? "js" : "ts"} already uses @vizejs/vite-plugin`,
      "  fmt       unchanged  uses project settings and formatter defaults",
      "  typecheck unchanged  uses tsconfig.json and project settings",
      "  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize",
    ]);
  }
});

test("every original manager cell retains its entire current discovery packet", () => {
  for (const cell of FRESH_INIT_MATRIX) {
    const current = CONFIG_FREE_PROJECT_SHAPES[cell.shape];
    const manager = PACKAGE_MANAGERS[cell.packageManager];
    assert.deepEqual(current.reconfiguredDetection(manager), [
      cell.shape === "vite-plus-vue-ts"
        ? "  framework:       Vite+ (vite.config.ts)"
        : `  framework:       Vite (vite.config.${cell.shape === "vite-vue-js-checkjs" ? "js" : "ts"})`,
      `  package manager: ${manager.detectedPackageManager}`,
      "  language:        TypeScript (tsconfig.json)",
      cell.shape === "vite-plus-vue-ts"
        ? "  lint command:    vp lint"
        : "  lint command:    oxlint",
      "  vize config:     none",
      "  oxlint config:   none",
    ]);
  }
});

test("the genuine packed driver selects the named successor and checks both init passes", () => {
  const source = readRepoFile("tools/support/compat/npm/smoke-release-init-fresh.mjs");
  assert.ok(source.includes("CONFIG_FREE_PROJECT_SHAPES as PROJECT_SHAPES"));
  assert.equal(source.match(/assertNoDedicatedConfig\(projectRoot\);/gu)?.length, 2);
  assert.ok(source.includes('name.startsWith("vize.config.")'));
  assert.ok(source.includes('"a second init run changed the project"'));
  assert.ok(source.includes("shape.check.brokenDiagnostics"));
});

test("the delivered Yarn cell retains its whole archive and declares three current stdout packets", () => {
  const historical = JSON.parse(
    readRepoFile("tests/tooling/fixtures/release-smoke-init-yarn-checkjs.json"),
  );
  const current = JSON.parse(
    readRepoFile("tests/tooling/fixtures/release-smoke-init-yarn-checkjs-config-free.json"),
  );
  const shape = CONFIG_FREE_PROJECT_SHAPES["vite-vue-js-checkjs"];
  const manager = PACKAGE_MANAGERS.yarn;
  for (const [key, value] of Object.entries(historical)) {
    if (key !== "expected") assert.deepEqual(current[key], value, key);
  }
  assert.deepEqual(Object.keys(current), Object.keys(historical));
  assert.deepEqual(Object.keys(current.expected), Object.keys(historical.expected));
  for (const [key, value] of Object.entries(historical.expected)) {
    if (!["files", "dryStdout", "applyStdout", "rerunStdout"].includes(key))
      assert.deepEqual(current.expected[key], value, key);
  }
  assert.deepEqual(FRESH_INIT_MATRIX, [
    ...historical.originalSevenCells,
    historical.proposedEighthCell,
  ]);
  assert.deepEqual(
    { ...shape.files(current.peers), ...manager.projectFiles },
    current.authoredFiles,
  );
  const { "vize.config.ts": archivedConfig, ...retainedFiles } = historical.expected.files;
  assert.equal(typeof archivedConfig, "string");
  assert.deepEqual(current.expected.files, retainedFiles);
  assert.deepEqual(shape.expectedFiles, current.expected.files);
  for (const mode of ["dry", "apply", "rerun"])
    assert.equal(
      expectedInitOutput(shape, current.projectRootToken, manager, mode),
      current.expected[`${mode}Stdout`],
    );
  assert.deepEqual(shape.check.broken, current.expected.broken.files);
  assert.deepEqual(shape.check.brokenDiagnostics, current.expected.broken.diagnosticFiles);
  assert.deepEqual(current.expected.clean, historical.expected.clean);
  assert.deepEqual(current.expected.broken, historical.expected.broken);
  assert.deepEqual(current.expected.repair, historical.expected.repair);
});

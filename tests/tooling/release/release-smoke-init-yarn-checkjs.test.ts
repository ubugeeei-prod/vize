import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

import {
  expectedInitOutput,
  managerCommand,
} from "../../../tools/support/compat/npm/smoke-release-init-project.mjs";
import { PACKAGE_MANAGERS } from "../../../tools/support/compat/npm/smoke-release-init-managers.mjs";
import {
  FRESH_INIT_MATRIX,
  PROJECT_SHAPES,
} from "../../../tools/support/compat/npm/smoke-release-init-shapes.mjs";

// Authored before the eighth matrix cell. This is a full source expectation,
// never an assertion that a package install, CLI or editor host has executed.
const authored = JSON.parse(
  fs.readFileSync(
    new URL("../fixtures/release-smoke-init-yarn-checkjs.json", import.meta.url),
    "utf8",
  ),
);
const manager = PACKAGE_MANAGERS.yarn;
const shape = PROJECT_SHAPES["vite-vue-js-checkjs"];

function withYarnBinary(binary: string | undefined, check: () => void) {
  const original = process.env[manager.binaryEnv];
  try {
    if (binary === undefined) delete process.env[manager.binaryEnv];
    else process.env[manager.binaryEnv] = binary;
    check();
  } finally {
    if (original === undefined) delete process.env[manager.binaryEnv];
    else process.env[manager.binaryEnv] = original;
  }
}

void test("Yarn checkJs is cell eight after the unchanged complete original seven", () => {
  assert.deepEqual(FRESH_INIT_MATRIX, [
    ...authored.originalSevenCells,
    authored.proposedEighthCell,
  ]);
  assert.deepEqual(authored.proposedEighthCell, {
    packageManager: "yarn",
    shape: "vite-vue-js-checkjs",
  });
});

void test("Yarn checkJs keeps the pinned existing manager, linker and complete command plans", () => {
  assert.equal(authored.manager, "yarn@4.9.2");
  assert.equal(manager.corepackSpec, authored.manager);
  assert.equal(manager.lockfile, "yarn.lock");
  assert.deepEqual(manager.projectFiles, { ".yarnrc.yml": "nodeLinker: node-modules\n" });
  assert.equal(authored.nodeLinker, "node-modules");
  assert.deepEqual(manager.environment, { YARN_ENABLE_IMMUTABLE_INSTALLS: "false" });
  withYarnBinary(undefined, () => {
    for (const [args, expected] of [
      [manager.bootstrapArgs, authored.commands.bootstrap],
      [manager.runScriptArgs("vize:check", []), authored.commands.generatedCheck],
      [
        [
          ...manager.installArgs,
          ...shape.plannedDependencies.map(
            (name) => `${name}@file:<authenticated-packed-artifact>`,
          ),
        ],
        authored.commands.plannedInstall,
      ],
    ]) {
      const runner = managerCommand(manager, args, "linux");
      assert.deepEqual([runner.command, ...runner.args], expected);
    }
  });
  assert.deepEqual(authored.commands.localCheck, [
    "<authenticated-node>",
    "<fresh-project>/node_modules/vize/bin/vize",
    "check",
    "--format",
    "json",
    "--quiet",
  ]);
});

void test("Yarn checkJs keeps the supported binary override and complete authored arguments", () => {
  withYarnBinary("/authored-yarn/bin/yarn", () => {
    for (const expected of [
      authored.commands.bootstrap,
      authored.commands.generatedCheck,
      authored.commands.plannedInstall,
    ]) {
      assert.deepEqual(managerCommand(manager, expected.slice(2), "linux"), {
        command: "/authored-yarn/bin/yarn",
        args: expected.slice(2),
      });
    }
  });
});

void test("Yarn checkJs preserves all authored project and generated config bytes and full init stdout", () => {
  assert.deepEqual(
    { ...shape.files(authored.peers), ...manager.projectFiles },
    authored.authoredFiles,
  );
  assert.deepEqual(shape.initialAbsentFiles, authored.initialAbsent);
  assert.deepEqual(shape.initFlags, authored.initFlags);
  assert.deepEqual(shape.expectedFiles, authored.expected.files);
  assert.deepEqual(shape.expectedScripts, authored.expected.manifestScripts);
  assert.deepEqual(shape.expectedDevDependencies, authored.expected.devDependencyNames);
  assert.deepEqual(shape.plannedDependencies, authored.expected.plannedDependencies);
  for (const [mode, expected] of [
    ["dry", authored.expected.dryStdout],
    ["apply", authored.expected.applyStdout],
    ["rerun", authored.expected.rerunStdout],
  ]) {
    assert.equal(expectedInitOutput(shape, authored.projectRootToken, manager, mode), expected);
  }
});

void test("Yarn checkJs uses the unchanged whole JS broken diagnostic and authored original repair", () => {
  assert.deepEqual(shape.check.broken, authored.expected.broken.files);
  assert.deepEqual(shape.check.brokenDiagnostics, authored.expected.broken.diagnosticFiles);
  assert.equal(authored.expected.broken.errorCount, 1);
  assert.equal(authored.expected.broken.exit, 1);
  assert.equal(authored.expected.broken.signal, null);
  assert.deepEqual(authored.expected.repair, {
    "src/main.js": shape.files(authored.peers)["src/main.js"],
  });
  assert.deepEqual(authored.expected.clean, {
    exit: 0,
    signal: null,
    errorCount: 0,
    diagnosticFiles: [],
  });
});

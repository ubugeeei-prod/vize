import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { materializeBaselineProject } from "../../tools/fixtures/typecheck-baseline-project.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const dependencyRoot =
  process.env.VIZE_TEST_WORKSPACE_NODE_MODULES ?? path.join(root, "tests/node_modules");
const vueTsc = path.join(dependencyRoot, ".bin/vue-tsc");

test(
  "materialized baseline checks Vue files omitted by a solution-style config",
  { skip: !fs.existsSync(vueTsc) },
  () => {
    const temp = fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-project-"));
    const fixtureRoot = path.join(temp, "fixture");
    const reportDir = path.join(temp, "report");
    const app = path.join(fixtureRoot, "src/App.vue");
    fs.mkdirSync(path.dirname(app), { recursive: true });
    fs.mkdirSync(reportDir);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.json"),
      `${JSON.stringify({ files: [], references: [{ path: "./tsconfig.app.json" }] })}\n`,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({ compilerOptions: { composite: true }, include: ["src/**/*.vue"] })}\n`,
    );
    fs.writeFileSync(app, '<script setup lang="ts">const value: string = 1</script>\n');

    try {
      const direct = runVueTsc(path.join(fixtureRoot, "tsconfig.json"), fixtureRoot);
      assert.doesNotMatch(direct.stdout, /App\.vue/u);

      const project = materializeBaselineProject(
        fixtureRoot,
        reportDir,
        { id: "fixture", tsconfig: "tsconfig.json" },
        { fileCount: 1, files: [{ file: "src/App.vue" }] },
      );
      const materialized = runVueTsc(project.path, fixtureRoot);
      assert.match(materialized.stdout, new RegExp(escapeRegExp(app)));
      assert.equal(materialized.status, 2, materialized.stderr);
    } finally {
      fs.rmSync(temp, { recursive: true, force: true });
    }
  },
);

function runVueTsc(project: string, cwd: string) {
  return spawnSync(vueTsc, ["--noEmit", "--pretty", "false", "--listFiles", "-p", project], {
    cwd,
    encoding: "utf8",
  });
}

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { rewriteOutsidePackagePaths } from "../../tools/fixtures/typecheck-baseline-outside-paths.mjs";
import { materializeBaselineProject } from "../../tools/fixtures/typecheck-baseline-project.mjs";

/**
 * Unique isolation can link a fixture-local copy, but vue-tsc still honors
 * `compilerOptions.paths`. When those mappings point above the fixture, the
 * baseline loads Vize's Vue beside the fixture's (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-outside-paths-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideRouter = path.join(outer, "node_modules", "vue-router");
  fs.mkdirSync(outsideRouter, { recursive: true });
  fs.writeFileSync(path.join(outsideRouter, "package.json"), `{"name":"vue-router"}\n`);
  return { outer, fixtureRoot, outsideRouter };
}

function writeLocalRouter(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "vue-router");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue-router"}\n`);
  return local;
}

test("an outside package path is retargeted to the fixture-local copy", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#imports": ["./src/imports.d.ts"],
            "vue-router": [path.relative(fixtureRoot, outsideRouter)],
          },
        },
      })}\n`,
    );
    const configDir = path.join(fixtureRoot, ".vize-baseline");
    assert.deepEqual(rewriteOutsidePackagePaths(fixtureRoot, sourcePath, configDir), {
      "#imports": ["../src/imports.d.ts"],
      "vue-router": ["../node_modules/vue-router"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside mapping reached only through extends is still retargeted", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        compilerOptions: {
          paths: { "vue-router": [path.relative(fixtureRoot, outsideRouter)] },
        },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(sourcePath, `{ "extends": "./tsconfig.app.json" }\n`);
    const configDir = path.join(fixtureRoot, ".vize-baseline");
    assert.deepEqual(rewriteOutsidePackagePaths(fixtureRoot, sourcePath, configDir), {
      "vue-router": ["../node_modules/vue-router"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside path with no fixture-local copy is left inherited", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "vue-router": [path.relative(fixtureRoot, outsideRouter)] },
        },
      })}\n`,
    );
    assert.equal(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("materialized baseline writes the retargeted paths onto the generated config", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.json"),
      `${JSON.stringify({
        compilerOptions: {
          paths: { "vue-router": [path.relative(fixtureRoot, outsideRouter)] },
        },
      })}\n`,
    );
    const reportDir = path.join(outer, "report");
    fs.mkdirSync(reportDir);
    const project = materializeBaselineProject(
      fixtureRoot,
      reportDir,
      { id: "fixture", tsconfig: "tsconfig.json" },
      { fileCount: 1, files: [{ file: "src/App.vue" }] },
    );
    assert.deepEqual(JSON.parse(project.source).compilerOptions.paths, {
      "vue-router": ["../node_modules/vue-router"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

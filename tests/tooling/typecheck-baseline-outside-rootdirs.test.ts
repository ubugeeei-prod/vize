import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { materializeBaselineProject } from "../../legacy-tools/fixtures/typecheck-baseline-project.mjs";
import {
  rewriteOutsideRootDirs,
  writeIsolatedTsconfigOverlay,
} from "../../legacy-tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * Unique isolation cannot retarget `compilerOptions.rootDirs`. An outside
 * `node_modules/vue` root lets TypeScript load Vize's Vue beside the fixture
 * (#4461). Overlay rewrite is the repair, and the vue-tsc baseline must apply
 * the same rewrite so the two tools still measure one program.
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-rootdirs-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideVue = path.join(outer, "node_modules", "vue");
  fs.mkdirSync(outsideVue, { recursive: true });
  fs.writeFileSync(path.join(outsideVue, "package.json"), `{"name":"vue"}\n`);
  return { fixtureRoot, outer, outsideVue };
}

function writeLocalVue(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "vue");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue"}\n`);
  return local;
}

test("an outside node_modules/vue rootDir is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { rootDirs: [path.relative(fixtureRoot, outsideVue)] },
      })}\n`,
    );
    const configDir = path.join(fixtureRoot, ".vize-baseline");
    assert.deepEqual(rewriteOutsideRootDirs(fixtureRoot, sourcePath, configDir), [
      "../node_modules/vue",
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("rootDirs reached only through relative extends are still retargeted", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        compilerOptions: { rootDirs: [path.relative(fixtureRoot, outsideVue)] },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(sourcePath, `{ "extends": "./tsconfig.app.json" }\n`);
    const overlay = writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath);
    assert.equal(overlay.path, path.join(fixtureRoot, ".vize-isolated-tsconfig.check.json"));
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.check.json",
      compilerOptions: { rootDirs: ["./node_modules/vue"] },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside rootDir is left for inheritance", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const generated = path.join(fixtureRoot, "generated");
    fs.mkdirSync(generated);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({ compilerOptions: { rootDirs: ["./generated"] } })}\n`,
    );
    assert.equal(rewriteOutsideRootDirs(fixtureRoot, sourcePath, fixtureRoot), null);
    assert.equal(writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside rootDir is left alone when the fixture has no local package", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { rootDirs: [path.relative(fixtureRoot, outsideVue)] },
      })}\n`,
    );
    assert.equal(rewriteOutsideRootDirs(fixtureRoot, sourcePath, fixtureRoot), null);
    assert.equal(writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a custom outside rootDir that is not a node_modules package is not guessed", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const outsideGenerated = path.join(outer, "generated");
    fs.mkdirSync(outsideGenerated);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { rootDirs: [path.relative(fixtureRoot, outsideGenerated)] },
      })}\n`,
    );
    assert.equal(rewriteOutsideRootDirs(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("materialized baseline retargets outside rootDirs with the overlay", () => {
  const { outer, fixtureRoot, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { rootDirs: [path.relative(fixtureRoot, outsideVue)] },
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
    const options = JSON.parse(project.source).compilerOptions;
    assert.deepEqual(options.rootDirs, ["../node_modules/vue"]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

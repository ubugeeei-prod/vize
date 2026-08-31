import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { applyIsolatedAliasOverlay } from "../../legacy-tools/fixtures/typecheck-baseline-outside-aliases.mjs";
import { rewriteOutsideCompilerPlugins } from "../../legacy-tools/fixtures/typecheck-baseline-outside-compiler-plugins.mjs";
import { writeIsolatedTsconfigOverlay } from "../../legacy-tools/fixtures/typecheck-baseline-outside-paths.mjs";
import { materializeBaselineProject } from "../../legacy-tools/fixtures/typecheck-baseline-project.mjs";

/**
 * Unique isolation cannot retarget `require("../../node_modules/<pkg>")` from
 * `compilerOptions.plugins` (#4461). Package-name plugins stay with unique-link.
 * Overlay rewrite is the repair for `{ name }` path specifiers, and the vue-tsc
 * baseline must apply the same rewrite so both tools measure one program.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-compiler-plugins-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsidePlugin = path.join(outer, "node_modules", "typescript-plugin-css-modules");
  fs.mkdirSync(outsidePlugin, { recursive: true });
  fs.writeFileSync(
    path.join(outsidePlugin, "package.json"),
    `{"name":"typescript-plugin-css-modules"}\n`,
  );
  return { fixtureRoot, outer, outsidePlugin };
}

function writeLocalPlugin(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "typescript-plugin-css-modules");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"typescript-plugin-css-modules"}\n`);
  return local;
}

test("an outside compiler plugin name is retargeted to the fixture copy", () => {
  const { fixtureRoot, outer, outsidePlugin } = scaffold();
  try {
    writeLocalPlugin(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          plugins: [{ pretty: true, name: path.relative(fixtureRoot, outsidePlugin) }],
        },
      })}\n`,
    );
    assert.deepEqual(rewriteOutsideCompilerPlugins(fixtureRoot, sourcePath, fixtureRoot), [
      { pretty: true, name: "./node_modules/typescript-plugin-css-modules" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a package-name compiler plugin is left for unique isolation", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalPlugin(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] },
      })}\n`,
    );
    assert.equal(rewriteOutsideCompilerPlugins(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an inside compiler plugin path is left for inheritance", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalPlugin(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          plugins: [{ name: "./node_modules/typescript-plugin-css-modules" }],
        },
      })}\n`,
    );
    assert.equal(rewriteOutsideCompilerPlugins(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside compiler plugin is left alone when the fixture has no local package", () => {
  const { fixtureRoot, outer, outsidePlugin } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { plugins: [{ name: path.relative(fixtureRoot, outsidePlugin) }] },
      })}\n`,
    );
    assert.equal(rewriteOutsideCompilerPlugins(fixtureRoot, sourcePath, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("relative extends concatenates parent compiler plugin paths onto the overlay", () => {
  const { fixtureRoot, outer, outsidePlugin } = scaffold();
  try {
    writeLocalPlugin(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        compilerOptions: { plugins: [{ name: path.relative(fixtureRoot, outsidePlugin) }] },
      })}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        extends: "./tsconfig.app.json",
        compilerOptions: { strict: true },
      })}\n`,
    );
    const overlay = applyIsolatedAliasOverlay(
      fixtureRoot,
      sourcePath,
      writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath),
    );
    assert.equal(overlay.path, path.join(fixtureRoot, ".vize-isolated-tsconfig.check.json"));
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.check.json",
      compilerOptions: {
        plugins: [{ name: "./node_modules/typescript-plugin-css-modules" }],
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("materialized baseline retargets outside compiler plugin paths with the overlay", () => {
  const { outer, fixtureRoot, outsidePlugin } = scaffold();
  try {
    writeLocalPlugin(fixtureRoot);
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: { plugins: [{ name: path.relative(fixtureRoot, outsidePlugin) }] },
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
    const parsed = JSON.parse(project.source);
    assert.deepEqual(parsed.compilerOptions.plugins, [
      { name: "../node_modules/typescript-plugin-css-modules" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

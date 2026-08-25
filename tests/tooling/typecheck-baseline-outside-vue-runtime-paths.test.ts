import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { applyIsolatedAliasOverlay } from "../../tools/fixtures/typecheck-baseline-outside-aliases.mjs";
import { rewriteLocalVueRuntimePaths } from "../../tools/fixtures/typecheck-baseline-outside-vue-runtime-paths.mjs";
import { materializeBaselineProject } from "../../tools/fixtures/typecheck-baseline-project.mjs";

/**
 * vue-tsc language-core still resolves `@vue/runtime-dom` from Vize's store
 * beside the fixture Vue (#4461). Overlay `paths` pin the program onto the
 * fixture copy when unique isolation has already placed that package.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-vue-runtime-paths-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  return { fixtureRoot, outer };
}

function writeLocalRuntime(fixtureRoot: string, name: string) {
  const local = path.join(fixtureRoot, "node_modules", ...name.split("/"));
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"${name}"}\n`);
  return local;
}

test("local Vue runtime packages are pinned onto overlay paths", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalRuntime(fixtureRoot, "vue");
    writeLocalRuntime(fixtureRoot, "@vue/runtime-dom");
    writeLocalRuntime(fixtureRoot, "@vue/runtime-core");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{}\n`);
    assert.deepEqual(rewriteLocalVueRuntimePaths(fixtureRoot, fixtureRoot), {
      "@vue/runtime-core": ["./node_modules/@vue/runtime-core"],
      "@vue/runtime-dom": ["./node_modules/@vue/runtime-dom"],
      vue: ["./node_modules/vue"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a missing local Vue runtime package is not guessed", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{}\n`);
    assert.equal(rewriteLocalVueRuntimePaths(fixtureRoot, fixtureRoot), null);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("alias overlay writes Vue runtime paths and keeps hash aliases", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocalRuntime(fixtureRoot, "@vue/runtime-dom");
    const outsideNuxt = path.join(outer, "node_modules", "nuxt");
    fs.mkdirSync(path.join(outsideNuxt, "dist", "app"), { recursive: true });
    fs.writeFileSync(path.join(outsideNuxt, "package.json"), `{"name":"nuxt"}\n`);
    const localNuxt = path.join(fixtureRoot, "node_modules", "nuxt");
    fs.mkdirSync(path.join(localNuxt, "dist", "app"), { recursive: true });
    fs.writeFileSync(path.join(localNuxt, "package.json"), `{"name":"nuxt"}\n`);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "#app": [path.relative(fixtureRoot, path.join(outsideNuxt, "dist", "app"))] },
        },
      })}\n`,
    );
    const overlay = applyIsolatedAliasOverlay(fixtureRoot, sourcePath, null);
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.json",
      compilerOptions: {
        paths: {
          "#app": ["./node_modules/nuxt/dist/app"],
          "@vue/runtime-dom": ["./node_modules/@vue/runtime-dom"],
        },
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("materialized baseline pins Vue runtime paths relative to the generated config", () => {
  const { outer, fixtureRoot } = scaffold();
  try {
    writeLocalRuntime(fixtureRoot, "@vue/runtime-dom");
    fs.writeFileSync(path.join(fixtureRoot, "src/App.vue"), "<template />\n");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{}\n`);
    const reportDir = path.join(outer, "report");
    fs.mkdirSync(reportDir);
    const project = materializeBaselineProject(
      fixtureRoot,
      reportDir,
      { id: "fixture", tsconfig: "tsconfig.json" },
      { fileCount: 1, files: [{ file: "src/App.vue" }] },
    );
    const parsed = JSON.parse(project.source);
    assert.deepEqual(parsed.compilerOptions.paths, {
      "@vue/runtime-dom": ["../node_modules/@vue/runtime-dom"],
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

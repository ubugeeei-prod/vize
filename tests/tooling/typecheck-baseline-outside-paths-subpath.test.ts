import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { rewriteOutsidePackagePaths } from "../../tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * Package overlay used to retarget `vue` onto the fixture package root and
 * drop `dist/vue`. Nuxt still points `paths` at that JSX entry above the
 * fixture, so vue-tsc would load Vize's `vue/jsx-runtime` (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-outside-paths-subpath-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideVue = path.join(outer, "node_modules", "vue");
  fs.mkdirSync(path.join(outsideVue, "dist"), { recursive: true });
  fs.writeFileSync(path.join(outsideVue, "package.json"), `{"name":"vue"}\n`);
  fs.writeFileSync(path.join(outsideVue, "dist", "vue.d.ts"), "export {}\n");
  const outsideRuntime = path.join(outer, "node_modules", "@vue", "runtime-dom");
  fs.mkdirSync(path.join(outsideRuntime, "dist"), { recursive: true });
  fs.writeFileSync(path.join(outsideRuntime, "package.json"), `{"name":"@vue/runtime-dom"}\n`);
  fs.writeFileSync(path.join(outsideRuntime, "dist", "runtime-dom.d.ts"), "export {}\n");
  return { fixtureRoot, outer, outsideRuntime, outsideVue };
}

function writeLocalVue(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "vue");
  fs.mkdirSync(path.join(local, "dist"), { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue"}\n`);
  fs.writeFileSync(path.join(local, "dist", "vue.d.ts"), "export {}\n");
  return local;
}

function writeLocalRuntimeDom(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "@vue", "runtime-dom");
  fs.mkdirSync(path.join(local, "dist"), { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"@vue/runtime-dom"}\n`);
  fs.writeFileSync(path.join(local, "dist", "runtime-dom.d.ts"), "export {}\n");
  return local;
}

test("an outside vue mapping that points at dist/vue keeps that subpath", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { vue: [path.relative(fixtureRoot, path.join(outsideVue, "dist", "vue"))] },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { vue: ["../node_modules/vue/dist/vue"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside scoped mapping keeps its dist subpath", () => {
  const { fixtureRoot, outer, outsideRuntime } = scaffold();
  try {
    writeLocalRuntimeDom(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "@vue/runtime-dom": [
              path.relative(fixtureRoot, path.join(outsideRuntime, "dist", "runtime-dom")),
            ],
          },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "@vue/runtime-dom": ["../node_modules/@vue/runtime-dom/dist/runtime-dom"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a trailing-star mapping under dist keeps that directory", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocalVue(fixtureRoot);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { "vue/*": [`${path.relative(fixtureRoot, path.join(outsideVue, "dist"))}/*`] },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "vue/*": ["../node_modules/vue/dist/*"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a missing fixture subpath is left inherited", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    const local = path.join(fixtureRoot, "node_modules", "vue");
    fs.mkdirSync(local, { recursive: true });
    fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue"}\n`);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { vue: [path.relative(fixtureRoot, path.join(outsideVue, "dist", "vue"))] },
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

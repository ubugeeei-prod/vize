import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import {
  applyIsolatedAliasOverlay,
  mergePathRewrites,
  rewriteOutsideAliasPaths,
} from "../../legacy-tools/fixtures/typecheck-baseline-outside-aliases.mjs";
import { writeIsolatedTsconfigOverlay } from "../../legacy-tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * Non-package `paths` keys still load outside trees (#4461). Overlay retargets
 * `#app` onto the fixture copy of the owning package. Package-name mappings
 * stay with the existing package rewrite.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-outside-aliases-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideNuxt = path.join(outer, "node_modules", "nuxt");
  fs.mkdirSync(path.join(outsideNuxt, "dist", "app"), { recursive: true });
  fs.writeFileSync(path.join(outsideNuxt, "package.json"), `{"name":"nuxt"}\n`);
  fs.writeFileSync(path.join(outsideNuxt, "dist", "app", "index.d.ts"), "export {}\n");
  const outsideRouter = path.join(outer, "node_modules", "vue-router");
  fs.mkdirSync(outsideRouter, { recursive: true });
  fs.writeFileSync(path.join(outsideRouter, "package.json"), `{"name":"vue-router"}\n`);
  return { outer, fixtureRoot, outsideNuxt, outsideRouter };
}

function writeLocalPackage(fixtureRoot: string, name: string) {
  const local = path.join(fixtureRoot, "node_modules", name);
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"${name}"}\n`);
  return local;
}

test("an outside hash alias is retargeted to the fixture copy of its package", () => {
  const { outer, fixtureRoot, outsideNuxt } = scaffold();
  try {
    writeLocalPackage(fixtureRoot, "nuxt");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#imports": ["./src/imports.d.ts"],
            "#app": [path.relative(fixtureRoot, path.join(outsideNuxt, "dist", "app"))],
          },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      {
        "#app": ["../node_modules/nuxt/dist/app"],
        "#imports": ["../src/imports.d.ts"],
      },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside hash alias with a trailing star is retargeted", () => {
  const { outer, fixtureRoot, outsideNuxt } = scaffold();
  try {
    writeLocalPackage(fixtureRoot, "nuxt");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#app/*": [`${path.relative(fixtureRoot, path.join(outsideNuxt, "dist", "app"))}/*`],
          },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "#app/*": ["../node_modules/nuxt/dist/app/*"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("package-name mappings are left for the package rewrite", () => {
  const { outer, fixtureRoot, outsideRouter } = scaffold();
  try {
    writeLocalPackage(fixtureRoot, "vue-router");
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
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an interior star in a hash alias is not guessed", () => {
  const { outer, fixtureRoot, outsideNuxt } = scaffold();
  try {
    writeLocalPackage(fixtureRoot, "nuxt");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#app": [`${path.relative(fixtureRoot, path.dirname(outsideNuxt))}/*/nuxt/dist/app`],
          },
        },
      })}\n`,
    );
    assert.equal(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("an outside hash alias with no fixture-local package is left inherited", () => {
  const { outer, fixtureRoot, outsideNuxt } = scaffold();
  try {
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#app": [path.relative(fixtureRoot, path.join(outsideNuxt, "dist", "app"))],
          },
        },
      })}\n`,
    );
    assert.equal(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("package rewrites win over alias rewrites for the same key", () => {
  assert.deepEqual(
    mergePathRewrites(
      { "vue-router": ["./node_modules/vue-router"], "#app": ["./escaped"] },
      { "vue-router": ["./wrong"], "#app": ["./node_modules/nuxt/dist/app"] },
    ),
    {
      "#app": ["./node_modules/nuxt/dist/app"],
      "vue-router": ["./node_modules/vue-router"],
    },
  );
});

test("alias overlay keeps package-path overlay typeRoots and merges paths", () => {
  const { outer, fixtureRoot, outsideNuxt, outsideRouter } = scaffold();
  try {
    writeLocalPackage(fixtureRoot, "nuxt");
    writeLocalPackage(fixtureRoot, "vue-router");
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#app": [path.relative(fixtureRoot, path.join(outsideNuxt, "dist", "app"))],
            "vue-router": [path.relative(fixtureRoot, outsideRouter)],
          },
        },
      })}\n`,
    );
    const overlay = applyIsolatedAliasOverlay(
      fixtureRoot,
      sourcePath,
      writeIsolatedTsconfigOverlay(fixtureRoot, sourcePath),
    );
    assert.deepEqual(overlay?.paths, {
      "#app": ["./node_modules/nuxt/dist/app"],
      "vue-router": ["./node_modules/vue-router"],
    });
    assert.deepEqual(JSON.parse(fs.readFileSync(overlay.path, "utf8")), {
      extends: "./tsconfig.json",
      compilerOptions: {
        paths: {
          "#app": ["./node_modules/nuxt/dist/app"],
          "vue-router": ["./node_modules/vue-router"],
        },
      },
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

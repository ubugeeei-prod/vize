import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { expandConfigDir } from "../../tools/fixtures/typecheck-baseline-config-dir.mjs";
import { rewriteOutsideAliasPaths } from "../../tools/fixtures/typecheck-baseline-outside-aliases.mjs";
import {
  pathMappingRoot,
  rewriteOutsidePackagePaths,
  rewriteOutsideTypeRoots,
} from "../../tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * TypeScript 5.5 expands `${configDir}` to the declaring tsconfig directory.
 * Overlay rewrite used to treat the token as a literal segment, so Nuxt-style
 * mappings never resolved above the fixture and never retargeted (#4461).
 */

const configDirToken = "${configDir}";

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-baseline-outside-config-dir-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, ".nuxt"), { recursive: true });
  const outsideVue = path.join(outer, "node_modules", "vue");
  fs.mkdirSync(outsideVue, { recursive: true });
  fs.writeFileSync(path.join(outsideVue, "package.json"), `{"name":"vue"}\n`);
  const outsideNuxt = path.join(outer, "node_modules", "nuxt");
  fs.mkdirSync(path.join(outsideNuxt, "dist", "app"), { recursive: true });
  fs.writeFileSync(path.join(outsideNuxt, "package.json"), `{"name":"nuxt"}\n`);
  const outsideTypes = path.join(outer, "node_modules", "@types");
  fs.mkdirSync(outsideTypes, { recursive: true });
  return { fixtureRoot, outer, outsideNuxt, outsideTypes, outsideVue };
}

function writeLocal(fixtureRoot: string, name: string) {
  const local = path.join(fixtureRoot, "node_modules", name);
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"${name}"}\n`);
  return local;
}

test("expandConfigDir substitutes the declaring tsconfig directory", () => {
  assert.equal(expandConfigDir("vue", "/abs/.nuxt"), "vue");
  assert.equal(
    expandConfigDir(`${configDirToken}/../node_modules/vue`, "/abs/.nuxt"),
    "/abs/.nuxt/../node_modules/vue",
  );
});

test("a ${configDir} package mapping that resolves outside is retargeted", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocal(fixtureRoot, "vue");
    const sourcePath = path.join(fixtureRoot, ".nuxt", "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: { vue: [`${configDirToken}/../../node_modules/vue`] },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { vue: ["../node_modules/vue"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a ${configDir} baseUrl still resolves outside package mappings from the fixture root", () => {
  const { fixtureRoot, outer, outsideVue } = scaffold();
  try {
    writeLocal(fixtureRoot, "vue");
    const nuxtDir = path.join(fixtureRoot, ".nuxt");
    const sourcePath = path.join(nuxtDir, "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          baseUrl: `${configDirToken}/..`,
          paths: { vue: [path.relative(fixtureRoot, outsideVue)] },
        },
      })}\n`,
    );
    assert.equal(pathMappingRoot(sourcePath, fixtureRoot, nuxtDir), fixtureRoot);
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { vue: ["../node_modules/vue"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a ${configDir} hash alias that resolves outside is retargeted", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeLocal(fixtureRoot, "nuxt");
    const sourcePath = path.join(fixtureRoot, ".nuxt", "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          paths: {
            "#app": [`${configDirToken}/../../node_modules/nuxt/dist/app`],
          },
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "#app": ["../node_modules/nuxt/dist/app"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a ${configDir} typeRoots mapping that resolves outside is retargeted", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    fs.mkdirSync(path.join(fixtureRoot, "node_modules", "@types"), { recursive: true });
    const sourcePath = path.join(fixtureRoot, ".nuxt", "tsconfig.json");
    fs.writeFileSync(
      sourcePath,
      `${JSON.stringify({
        compilerOptions: {
          typeRoots: [`${configDirToken}/../../node_modules/@types`],
        },
      })}\n`,
    );
    assert.deepEqual(
      rewriteOutsideTypeRoots(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      ["../node_modules/@types"],
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

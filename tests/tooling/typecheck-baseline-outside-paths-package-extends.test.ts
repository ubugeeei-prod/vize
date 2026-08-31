import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { rewriteOutsidePackagePaths } from "../../legacy-tools/fixtures/typecheck-baseline-outside-paths.mjs";

/**
 * Overlay rewrite used to follow only relative `extends`. A package-name
 * specifier is a different walk: TypeScript climbs `node_modules` and can load
 * Vize's `@vue/tsconfig`, whose `paths` still point above the fixture (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-overlay-package-extends-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideRouter = path.join(outer, "node_modules", "vue-router");
  fs.mkdirSync(outsideRouter, { recursive: true });
  fs.writeFileSync(path.join(outsideRouter, "package.json"), `{"name":"vue-router"}\n`);
  return { fixtureRoot, outer, outsideRouter };
}

function writeLocalRouter(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "vue-router");
  fs.mkdirSync(local, { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue-router"}\n`);
  return local;
}

function writeVueTsconfig(packageRoot: string, outsideRouter: string, fileName = "tsconfig.json") {
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"@vue/tsconfig"}\n`);
  fs.writeFileSync(
    path.join(packageRoot, fileName),
    `${JSON.stringify({
      compilerOptions: {
        paths: { "vue-router": [path.relative(packageRoot, outsideRouter)] },
      },
    })}\n`,
  );
}

test("an outside mapping reached only through a fixture package-name extends is retargeted", () => {
  const { fixtureRoot, outer, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    writeVueTsconfig(path.join(fixtureRoot, "node_modules", "@vue", "tsconfig"), outsideRouter);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{ "extends": "@vue/tsconfig" }\n`);
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "vue-router": ["../node_modules/vue-router"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a subpath package-name extends still contributes its outside mapping", () => {
  const { fixtureRoot, outer, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    writeVueTsconfig(
      path.join(fixtureRoot, "node_modules", "@vue", "tsconfig"),
      outsideRouter,
      "tsconfig.dom.json",
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{ "extends": "@vue/tsconfig/tsconfig.dom.json" }\n`);
    assert.deepEqual(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "vue-router": ["../node_modules/vue-router"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a package-name extends that lives only above the fixture is not followed", () => {
  const { fixtureRoot, outer, outsideRouter } = scaffold();
  try {
    writeLocalRouter(fixtureRoot);
    writeVueTsconfig(path.join(outer, "node_modules", "@vue", "tsconfig"), outsideRouter);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{ "extends": "@vue/tsconfig" }\n`);
    assert.equal(
      rewriteOutsidePackagePaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

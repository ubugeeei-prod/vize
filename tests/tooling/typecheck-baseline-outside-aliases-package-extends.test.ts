import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { rewriteOutsideAliasPaths } from "../../legacy-tools/fixtures/typecheck-baseline-outside-aliases.mjs";

/**
 * Hash-alias overlay used to follow only relative `extends`. A package-name
 * specifier is a different walk: TypeScript climbs `node_modules` and can load
 * Vize's generated Nuxt config, whose `#app` still points above the fixture
 * (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-overlay-alias-package-extends-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(path.join(fixtureRoot, "src"), { recursive: true });
  const outsideNuxt = path.join(outer, "node_modules", "nuxt");
  fs.mkdirSync(path.join(outsideNuxt, "dist", "app"), { recursive: true });
  fs.writeFileSync(path.join(outsideNuxt, "package.json"), `{"name":"nuxt"}\n`);
  fs.writeFileSync(path.join(outsideNuxt, "dist", "app", "index.d.ts"), "export {}\n");
  return { fixtureRoot, outer, outsideNuxt };
}

function writeLocalNuxt(fixtureRoot: string) {
  const local = path.join(fixtureRoot, "node_modules", "nuxt");
  fs.mkdirSync(path.join(local, "dist", "app"), { recursive: true });
  fs.writeFileSync(path.join(local, "package.json"), `{"name":"nuxt"}\n`);
  fs.writeFileSync(path.join(local, "dist", "app", "index.d.ts"), "export {}\n");
  return local;
}

function writeNuxtTsconfig(packageRoot: string, outsideNuxt: string, fileName = "tsconfig.json") {
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"nuxt"}\n`);
  fs.writeFileSync(
    path.join(packageRoot, fileName),
    `${JSON.stringify({
      compilerOptions: {
        paths: { "#app": [path.relative(packageRoot, path.join(outsideNuxt, "dist", "app"))] },
      },
    })}\n`,
  );
}

test("an outside hash alias reached only through a fixture package-name extends is retargeted", () => {
  const { fixtureRoot, outer, outsideNuxt } = scaffold();
  try {
    writeLocalNuxt(fixtureRoot);
    writeNuxtTsconfig(
      path.join(fixtureRoot, "node_modules", "@nuxt", "typescript-config"),
      outsideNuxt,
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "node_modules", "@nuxt", "typescript-config", "package.json"),
      `{"name":"@nuxt/typescript-config"}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{ "extends": "@nuxt/typescript-config" }\n`);
    assert.deepEqual(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "#app": ["../node_modules/nuxt/dist/app"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a subpath package-name extends still contributes its outside hash alias", () => {
  const { fixtureRoot, outer, outsideNuxt } = scaffold();
  try {
    writeLocalNuxt(fixtureRoot);
    writeNuxtTsconfig(
      path.join(fixtureRoot, "node_modules", "@nuxt", "typescript-config"),
      outsideNuxt,
      "tsconfig.app.json",
    );
    fs.writeFileSync(
      path.join(fixtureRoot, "node_modules", "@nuxt", "typescript-config", "package.json"),
      `{"name":"@nuxt/typescript-config"}\n`,
    );
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{ "extends": "@nuxt/typescript-config/tsconfig.app.json" }\n`);
    assert.deepEqual(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      { "#app": ["../node_modules/nuxt/dist/app"] },
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a package-name extends that lives only above the fixture is not followed", () => {
  const { fixtureRoot, outer, outsideNuxt } = scaffold();
  try {
    writeLocalNuxt(fixtureRoot);
    writeNuxtTsconfig(path.join(outer, "node_modules", "@nuxt", "typescript-config"), outsideNuxt);
    const sourcePath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(sourcePath, `{ "extends": "@nuxt/typescript-config" }\n`);
    assert.equal(
      rewriteOutsideAliasPaths(fixtureRoot, sourcePath, path.join(fixtureRoot, ".vize-baseline")),
      null,
    );
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

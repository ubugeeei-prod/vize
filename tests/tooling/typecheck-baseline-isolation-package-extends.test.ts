import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { readDeclaredPackagePaths } from "../../legacy-tools/fixtures/typecheck-baseline-isolation.mjs";
import {
  ancestorPackagePath,
  packageNameFromExtendsSpecifier,
} from "../../legacy-tools/fixtures/typecheck-baseline-isolation-package-extends.mjs";
import { isolateUniqueLocalTypePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";

/**
 * Relative `extends` is already followed for `paths`. A package-name specifier
 * is a different walk: TypeScript climbs `node_modules` and can load Vize's
 * `@vue/tsconfig` (or `nuxt`) instead of the fixture's copy (#4461).
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-package-extends-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  const ancestor = path.join(outer, "node_modules", "@vue", "tsconfig");
  fs.mkdirSync(ancestor, { recursive: true });
  fs.writeFileSync(path.join(ancestor, "package.json"), `{"name":"@vue/tsconfig"}\n`);
  return { ancestor, fixtureRoot, outer };
}

function writeStoreCopy(fixtureRoot: string) {
  const packageRoot = path.join(
    fixtureRoot,
    "node_modules",
    ".pnpm",
    "@vue+tsconfig@0.5.0",
    "node_modules",
    "@vue",
    "tsconfig",
  );
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"@vue/tsconfig"}\n`);
  return packageRoot;
}

test("package-name extends specifiers resolve to the package, not a subpath", () => {
  assert.equal(packageNameFromExtendsSpecifier("@vue/tsconfig"), "@vue/tsconfig");
  assert.equal(packageNameFromExtendsSpecifier("@vue/tsconfig/tsconfig.dom.json"), "@vue/tsconfig");
  assert.equal(packageNameFromExtendsSpecifier("nuxt"), "nuxt");
  assert.equal(packageNameFromExtendsSpecifier("nuxt/tsconfig"), "nuxt");
  assert.equal(packageNameFromExtendsSpecifier("./tsconfig.app.json"), null);
  assert.equal(packageNameFromExtendsSpecifier("../tsconfig.json"), null);
  assert.equal(
    packageNameFromExtendsSpecifier("../node_modules/@vue/tsconfig/tsconfig.json"),
    "@vue/tsconfig",
  );
  assert.equal(packageNameFromExtendsSpecifier("../../node_modules/nuxt/tsconfig"), "nuxt");
});

test("an ancestor package-name extends is recorded as the ancestor directory", () => {
  const { ancestor, fixtureRoot, outer } = scaffold();
  try {
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(configPath, `{ "extends": "@vue/tsconfig" }\n`);
    assert.equal(ancestorPackagePath(fixtureRoot, "@vue/tsconfig"), ancestor);
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@vue/tsconfig": ancestor,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a subpath package-name extends still names the package", () => {
  const { ancestor, fixtureRoot, outer } = scaffold();
  try {
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(configPath, `{ "extends": "@vue/tsconfig/tsconfig.dom.json" }\n`);
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@vue/tsconfig": ancestor,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation links a relative node_modules package-name extends", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot);
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.json");
    fs.mkdirSync(path.dirname(configPath), { recursive: true });
    fs.writeFileSync(configPath, `{ "extends": "../node_modules/@vue/tsconfig/tsconfig.json" }\n`);
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      {
        name: "@vue/tsconfig",
        target: "node_modules/.pnpm/@vue+tsconfig@0.5.0/node_modules/@vue/tsconfig",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation links the fixture copy of a package-name extends", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot);
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(configPath, `{ "extends": "@vue/tsconfig" }\n`);
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      {
        name: "@vue/tsconfig",
        target: "node_modules/.pnpm/@vue+tsconfig@0.5.0/node_modules/@vue/tsconfig",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation records package-name extends from a relative parent", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot);
    fs.writeFileSync(
      path.join(fixtureRoot, "tsconfig.app.json"),
      `${JSON.stringify({
        extends: "@vue/tsconfig",
        compilerOptions: { paths: { vue: ["./node_modules/vue"] } },
      })}\n`,
    );
    const configPath = path.join(fixtureRoot, "tsconfig.check.json");
    fs.writeFileSync(configPath, `// check-only\n{ "extends": "./tsconfig.app.json", }\n`);
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      {
        name: "@vue/tsconfig",
        target: "node_modules/.pnpm/@vue+tsconfig@0.5.0/node_modules/@vue/tsconfig",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("paths still win when the same name is also a package-name extends", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const local = path.join(fixtureRoot, "packages", "tsconfig");
    fs.mkdirSync(local, { recursive: true });
    fs.writeFileSync(path.join(local, "package.json"), `{"name":"@vue/tsconfig"}\n`);
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({
        extends: "@vue/tsconfig",
        compilerOptions: { paths: { "@vue/tsconfig": ["./packages/tsconfig"] } },
      })}\n`,
    );
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@vue/tsconfig": local,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a package-name extends with no fixture copy is left unlinked", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const configPath = path.join(fixtureRoot, "tsconfig.json");
    fs.writeFileSync(configPath, `{ "extends": "@vue/tsconfig" }\n`);
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

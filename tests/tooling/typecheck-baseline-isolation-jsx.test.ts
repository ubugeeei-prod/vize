import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { readDeclaredPackagePaths } from "../../legacy-tools/fixtures/typecheck-baseline-isolation.mjs";
import { isolateUniqueLocalTypePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";
import { jsxImportSourcePackageName } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-jsx.mjs";

/**
 * `compilerOptions.jsxImportSource` resolves `<name>/jsx-runtime` by climbing
 * `node_modules` (#4461). Unique isolation links the fixture copy. Child
 * replaces parent. Package-name `extends` configs are not read.
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-jsx-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  const ancestorVue = path.join(outer, "node_modules", "vue");
  fs.mkdirSync(ancestorVue, { recursive: true });
  fs.writeFileSync(path.join(ancestorVue, "package.json"), `{"name":"vue"}\n`);
  const ancestorPreact = path.join(outer, "node_modules", "preact");
  fs.mkdirSync(ancestorPreact, { recursive: true });
  fs.writeFileSync(path.join(ancestorPreact, "package.json"), `{"name":"preact"}\n`);
  const ancestorVueTsconfig = path.join(outer, "node_modules", "@vue", "tsconfig");
  fs.mkdirSync(ancestorVueTsconfig, { recursive: true });
  fs.writeFileSync(path.join(ancestorVueTsconfig, "package.json"), `{"name":"@vue/tsconfig"}\n`);
  fs.writeFileSync(
    path.join(ancestorVueTsconfig, "tsconfig.json"),
    `${JSON.stringify({ compilerOptions: { jsxImportSource: "vue" } })}\n`,
  );
  return { ancestorPreact, ancestorVue, ancestorVueTsconfig, fixtureRoot, outer };
}

function writeStoreCopy(fixtureRoot: string, id: string, name: string) {
  const packageRoot = path.join(fixtureRoot, "node_modules", ".pnpm", id, "node_modules", name);
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"${name}"}\n`);
  return packageRoot;
}

function writeConfig(fixtureRoot: string, config: unknown, fileName = "tsconfig.json") {
  const configPath = path.join(fixtureRoot, fileName);
  fs.writeFileSync(configPath, `${JSON.stringify(config)}\n`);
  return configPath;
}

test("jsxImportSource names the package, not a relative path", () => {
  assert.equal(jsxImportSourcePackageName("vue"), "vue");
  assert.equal(jsxImportSourcePackageName("preact"), "preact");
  assert.equal(jsxImportSourcePackageName("./jsx"), null);
  assert.equal(jsxImportSourcePackageName("../node_modules/vue"), "vue");
  assert.equal(jsxImportSourcePackageName(undefined), null);
});

test("compilerOptions.jsxImportSource records ancestor packages for unique isolation", () => {
  const { ancestorVue, fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { jsxImportSource: "vue" },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      vue: ancestorVue,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation links a relative node_modules jsxImportSource specifier", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue@3.5.13", "vue");
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.json");
    fs.mkdirSync(path.dirname(configPath), { recursive: true });
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({ compilerOptions: { jsxImportSource: "../node_modules/vue" } })}\n`,
    );
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      { name: "vue", target: "node_modules/.pnpm/vue@3.5.13/node_modules/vue" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation links the fixture copy of a jsxImportSource package", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vue@3.5.13", "vue");
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { jsxImportSource: "vue" },
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      { name: "vue", target: "node_modules/.pnpm/vue@3.5.13/node_modules/vue" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("jsxImportSource on a relative extends parent is recorded", () => {
  const { ancestorVue, fixtureRoot, outer } = scaffold();
  try {
    writeConfig(fixtureRoot, { compilerOptions: { jsxImportSource: "vue" } }, "tsconfig.app.json");
    const configPath = writeConfig(fixtureRoot, { extends: "./tsconfig.app.json" });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      vue: ancestorVue,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("child jsxImportSource replaces parent jsxImportSource", () => {
  const { ancestorPreact, fixtureRoot, outer } = scaffold();
  try {
    writeConfig(fixtureRoot, { compilerOptions: { jsxImportSource: "vue" } }, "tsconfig.app.json");
    const configPath = writeConfig(fixtureRoot, {
      extends: "./tsconfig.app.json",
      compilerOptions: { jsxImportSource: "preact" },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      preact: ancestorPreact,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("package-name extends is not walked for jsxImportSource", () => {
  const { ancestorVueTsconfig, fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, { extends: "@vue/tsconfig" });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@vue/tsconfig": ancestorVueTsconfig,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("paths still win when the same name is also jsxImportSource", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const local = path.join(fixtureRoot, "packages", "vue");
    fs.mkdirSync(local, { recursive: true });
    fs.writeFileSync(path.join(local, "package.json"), `{"name":"vue"}\n`);
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: {
        jsxImportSource: "vue",
        paths: { vue: ["./packages/vue"] },
      },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      vue: local,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a jsxImportSource package with no fixture copy is left unlinked", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { jsxImportSource: "vue" },
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

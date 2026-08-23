import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { readDeclaredPackagePaths } from "../../tools/fixtures/typecheck-baseline-isolation.mjs";
import { isolateUniqueLocalTypePackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";
import { pluginPackageNamesFromConfigs } from "../../tools/fixtures/typecheck-baseline-isolation-plugins.mjs";

/**
 * Language-service plugins resolve by climbing `node_modules` (#4461). Unique
 * isolation links the fixture copy. Package-name `extends` configs are not
 * read for plugins; relative `extends` chains union plugin lists.
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-plugins-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  const ancestorTsPlugin = path.join(outer, "node_modules", "typescript-plugin-css-modules");
  fs.mkdirSync(ancestorTsPlugin, { recursive: true });
  fs.writeFileSync(
    path.join(ancestorTsPlugin, "package.json"),
    `{"name":"typescript-plugin-css-modules"}\n`,
  );
  const ancestorVuePlugin = path.join(outer, "node_modules", "@vue", "language-plugin-pug");
  fs.mkdirSync(ancestorVuePlugin, { recursive: true });
  fs.writeFileSync(
    path.join(ancestorVuePlugin, "package.json"),
    `{"name":"@vue/language-plugin-pug"}\n`,
  );
  const ancestorVueTsconfig = path.join(outer, "node_modules", "@vue", "tsconfig");
  fs.mkdirSync(ancestorVueTsconfig, { recursive: true });
  fs.writeFileSync(path.join(ancestorVueTsconfig, "package.json"), `{"name":"@vue/tsconfig"}\n`);
  fs.writeFileSync(
    path.join(ancestorVueTsconfig, "tsconfig.json"),
    `${JSON.stringify({
      compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] },
    })}\n`,
  );
  return { ancestorTsPlugin, ancestorVuePlugin, ancestorVueTsconfig, fixtureRoot, outer };
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

test("plugin names come from compilerOptions and vueCompilerOptions", () => {
  assert.deepEqual(
    pluginPackageNamesFromConfigs([
      {
        compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] },
        vueCompilerOptions: {
          plugins: ["@vue/language-plugin-pug", ["./local-plugin.js", { pretty: true }]],
        },
      },
    ]),
    ["typescript-plugin-css-modules", "@vue/language-plugin-pug"],
  );
  assert.deepEqual(pluginPackageNamesFromConfigs([]), []);
  assert.deepEqual(pluginPackageNamesFromConfigs(undefined), []);
});

test("compilerOptions.plugins records ancestor packages for unique isolation", () => {
  const { ancestorTsPlugin, fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "typescript-plugin-css-modules": ancestorTsPlugin,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("vueCompilerOptions.plugins records ancestor packages", () => {
  const { ancestorVuePlugin, fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      vueCompilerOptions: { plugins: ["@vue/language-plugin-pug"] },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@vue/language-plugin-pug": ancestorVuePlugin,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation links the fixture copy of a plugin package", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(
      fixtureRoot,
      "typescript-plugin-css-modules@5.0.0",
      "typescript-plugin-css-modules",
    );
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] },
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      {
        name: "typescript-plugin-css-modules",
        target:
          "node_modules/.pnpm/typescript-plugin-css-modules@5.0.0/node_modules/typescript-plugin-css-modules",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("plugins on a relative extends parent are recorded", () => {
  const { ancestorTsPlugin, fixtureRoot, outer } = scaffold();
  try {
    writeConfig(
      fixtureRoot,
      { compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] } },
      "tsconfig.app.json",
    );
    const configPath = writeConfig(fixtureRoot, { extends: "./tsconfig.app.json" });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "typescript-plugin-css-modules": ancestorTsPlugin,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("child and parent plugins are unioned", () => {
  const { ancestorTsPlugin, ancestorVuePlugin, fixtureRoot, outer } = scaffold();
  try {
    writeConfig(
      fixtureRoot,
      { compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] } },
      "tsconfig.app.json",
    );
    const configPath = writeConfig(fixtureRoot, {
      extends: "./tsconfig.app.json",
      vueCompilerOptions: { plugins: [["@vue/language-plugin-pug", { pretty: true }]] },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@vue/language-plugin-pug": ancestorVuePlugin,
      "typescript-plugin-css-modules": ancestorTsPlugin,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("package-name extends is not walked for plugins", () => {
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

test("paths still win when the same name is also a plugin", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const local = path.join(fixtureRoot, "packages", "typescript-plugin-css-modules");
    fs.mkdirSync(local, { recursive: true });
    fs.writeFileSync(
      path.join(local, "package.json"),
      `{"name":"typescript-plugin-css-modules"}\n`,
    );
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: {
        plugins: [{ name: "typescript-plugin-css-modules" }],
        paths: { "typescript-plugin-css-modules": ["./packages/typescript-plugin-css-modules"] },
      },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "typescript-plugin-css-modules": local,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a plugin package with no fixture copy is left unlinked", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { plugins: [{ name: "typescript-plugin-css-modules" }] },
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

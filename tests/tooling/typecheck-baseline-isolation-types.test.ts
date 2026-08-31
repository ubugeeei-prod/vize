import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { readDeclaredPackagePaths } from "../../legacy-tools/fixtures/typecheck-baseline-isolation.mjs";
import { isolateUniqueLocalTypePackages } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-unique.mjs";
import { typePackageNamesFromTypes } from "../../legacy-tools/fixtures/typecheck-baseline-isolation-types.mjs";

/**
 * `compilerOptions.types` is a type-reference walk out of the fixture (#4461).
 * Overlay cannot retarget it; unique isolation links the fixture copy of each
 * named package (and `@types/<name>` for unscoped entries). Package-name
 * `extends` configs are not read for `types`.
 */

function scaffold() {
  const outer = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-types-")));
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  const ancestorVite = path.join(outer, "node_modules", "vite");
  fs.mkdirSync(ancestorVite, { recursive: true });
  fs.writeFileSync(path.join(ancestorVite, "package.json"), `{"name":"vite"}\n`);
  const ancestorNode = path.join(outer, "node_modules", "@types", "node");
  fs.mkdirSync(ancestorNode, { recursive: true });
  fs.writeFileSync(path.join(ancestorNode, "package.json"), `{"name":"@types/node"}\n`);
  const ancestorVueTsconfig = path.join(outer, "node_modules", "@vue", "tsconfig");
  fs.mkdirSync(ancestorVueTsconfig, { recursive: true });
  fs.writeFileSync(path.join(ancestorVueTsconfig, "package.json"), `{"name":"@vue/tsconfig"}\n`);
  fs.writeFileSync(
    path.join(ancestorVueTsconfig, "tsconfig.json"),
    `${JSON.stringify({ compilerOptions: { types: ["vite/client"] } })}\n`,
  );
  return { ancestorNode, ancestorVite, ancestorVueTsconfig, fixtureRoot, outer };
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

test("types entries resolve to the package and @types package for unscoped names", () => {
  assert.deepEqual(typePackageNamesFromTypes(["vite/client"]), ["vite", "@types/vite"]);
  assert.deepEqual(typePackageNamesFromTypes(["node"]), ["node", "@types/node"]);
  assert.deepEqual(typePackageNamesFromTypes(["@vue/runtime-dom"]), ["@vue/runtime-dom"]);
  assert.deepEqual(typePackageNamesFromTypes(["../node_modules/@types/node"]), ["@types/node"]);
  assert.deepEqual(typePackageNamesFromTypes(["../node_modules/vite/client"]), [
    "vite",
    "@types/vite",
  ]);
  assert.deepEqual(typePackageNamesFromTypes([]), []);
  assert.deepEqual(typePackageNamesFromTypes(undefined), []);
});

test("compilerOptions.types records ancestor packages for unique isolation", () => {
  const { ancestorVite, fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { types: ["vite/client"] },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      vite: ancestorVite,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("compilerOptions.types node records the ancestor @types/node package", () => {
  const { ancestorNode, fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { types: ["node"] },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@types/node": ancestorNode,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation links the fixture copy of a types package", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "vite@6.0.0", "vite");
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { types: ["vite/client"] },
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      { name: "vite", target: "node_modules/.pnpm/vite@6.0.0/node_modules/vite" },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("unique isolation links a relative node_modules types specifier", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot, "@types+node@22.0.0", "@types/node");
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.json");
    fs.mkdirSync(path.dirname(configPath), { recursive: true });
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({ compilerOptions: { types: ["../node_modules/@types/node"] } })}\n`,
    );
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      {
        name: "@types/node",
        target: "node_modules/.pnpm/@types+node@22.0.0/node_modules/@types/node",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("types on a relative extends parent are recorded", () => {
  const { ancestorVite, fixtureRoot, outer } = scaffold();
  try {
    writeConfig(fixtureRoot, { compilerOptions: { types: ["vite/client"] } }, "tsconfig.app.json");
    const configPath = writeConfig(fixtureRoot, { extends: "./tsconfig.app.json" });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      vite: ancestorVite,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("child types replace parent types", () => {
  const { ancestorNode, fixtureRoot, outer } = scaffold();
  try {
    writeConfig(fixtureRoot, { compilerOptions: { types: ["vite/client"] } }, "tsconfig.app.json");
    const configPath = writeConfig(fixtureRoot, {
      extends: "./tsconfig.app.json",
      compilerOptions: { types: ["node"] },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      "@types/node": ancestorNode,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("package-name extends is not walked for types", () => {
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

test("paths still win when the same name is also a types entry", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const local = path.join(fixtureRoot, "packages", "vite");
    fs.mkdirSync(local, { recursive: true });
    fs.writeFileSync(path.join(local, "package.json"), `{"name":"vite"}\n`);
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: {
        types: ["vite/client"],
        paths: { vite: ["./packages/vite"] },
      },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {
      vite: local,
    });
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("typeRoots directories are not recorded as packages", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { typeRoots: ["./typings"] },
    });
    assert.deepEqual(Object.fromEntries(readDeclaredPackagePaths(fixtureRoot, configPath)), {});
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

test("a types package with no fixture copy is left unlinked", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    const configPath = writeConfig(fixtureRoot, {
      compilerOptions: { types: ["vite/client"] },
    });
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), []);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});

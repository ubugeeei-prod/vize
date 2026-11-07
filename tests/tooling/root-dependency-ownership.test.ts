import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const readRepoFile = (filePath: string): string =>
  fs.readFileSync(path.join(root, filePath), "utf8");
const readManifest = (filePath: string): Record<string, string> => {
  const manifest = JSON.parse(readRepoFile(filePath)) as {
    devDependencies?: Record<string, string>;
  };
  return manifest.devDependencies ?? {};
};

test("root development dependencies stay limited to root execution entry points", () => {
  assert.deepEqual(Object.keys(readManifest("package.json")).sort(), [
    "@types/node",
    "oxfmt",
    "oxlint",
    "pkg-pr-new",
    "vite",
    "vite-plus",
  ]);

  const ownership: Record<string, readonly string[]> = {
    "docs/package.json": ["yaml"],
    "editors/package.json": ["@types/vscode", "vscode-languageclient"],
    "npm/builder/unplugin/package.json": ["@tsdown/css", "tsdown"],
    "npm/builder/vite/package.json": ["oxfmt"],
    "npm/cli/package.json": ["json-schema-to-typescript", "oxfmt"],
    "npm/framework/nuxt-lint-config/package.json": ["@nuxt/eslint", "@nuxt/eslint-config"],
    "npm/framework/nuxt/package.json": ["oxlint"],
    "npm/linter/oxlint/package.json": [
      "@nuxt/eslint",
      "@nuxt/eslint-config",
      "@typescript-eslint/parser",
      "ajv",
      "eslint",
      "eslint-plugin-vue",
      "oxlint",
      "vue-eslint-parser",
    ],
    "npm/ui/package.json": ["@babel/core"],
    "tests/package.json": [
      "@babel/core",
      "@babel/plugin-syntax-typescript",
      "@types/vscode",
      "@vue/babel-plugin-jsx",
      "semver",
      "vscode-languageclient",
      "vscode-oniguruma",
      "vscode-textmate",
      "zigpty",
    ],
    "tools/benchmarks/scripts/package.json": ["@iarna/toml", "ajv", "oxfmt"],
    "tools/package.json": ["@iarna/toml", "chokidar-braces-oracle", "oxlint", "semver", "yaml"],
  };

  for (const [manifestPath, dependencies] of Object.entries(ownership)) {
    const devDependencies = readManifest(manifestPath);
    for (const dependency of dependencies) {
      assert.ok(devDependencies[dependency], `${dependency} must be owned by ${manifestPath}`);
    }
  }

  const workspace = parse(readRepoFile("pnpm-workspace.yaml")) as { packages?: string[] };
  assert.ok(workspace.packages?.includes("editors"), "editor tooling must be a workspace package");
  assert.ok(workspace.packages?.includes("tools"), "tools must be a workspace package");
  assert.equal(fs.existsSync(path.join(root, "docs/pnpm-lock.yaml")), false);
});

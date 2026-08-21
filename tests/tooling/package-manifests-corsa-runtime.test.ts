import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

test("Corsa runtime is declared for vize check users", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "npm/cli/package.json"), "utf-8"),
  ) as {
    dependencies?: Record<string, string>;
    optionalDependencies?: Record<string, string>;
    peerDependencies?: Record<string, string>;
    peerDependenciesMeta?: Record<string, { optional?: boolean }>;
  };

  for (const name of ["@typescript/native-preview", "typescript"]) {
    for (const section of [
      "dependencies",
      "optionalDependencies",
      "peerDependencies",
      "peerDependenciesMeta",
    ] as const) {
      assert.equal(packageJson[section]?.[name], undefined);
    }
  }
  for (const name of [
    "@typescript/typescript-darwin-arm64",
    "@typescript/typescript-darwin-x64",
    "@typescript/typescript-linux-arm64",
    "@typescript/typescript-linux-x64",
    "@typescript/typescript-win32-arm64",
    "@typescript/typescript-win32-x64",
  ]) {
    assert.equal(packageJson.optionalDependencies?.[name], "catalog:corsa-runtime");
  }
});

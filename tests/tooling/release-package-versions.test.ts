import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const releasePackageJsonPaths = [
  "npm/builder/rspack/package.json",
  "npm/builder/unplugin/package.json",
  "npm/builder/vite/package.json",
  "npm/builder/vite-musea/package.json",
  "npm/cli/package.json",
  "npm/fresco/package.json",
  "npm/fresco-native/package.json",
  "npm/framework/musea-nuxt/package.json",
  "npm/framework/nuxt-lint-config/package.json",
  "npm/framework/nuxt/package.json",
  "npm/marquette/package.json",
  "npm/mcp-musea/package.json",
  "npm/native/package.json",
  "npm/oxlint/package.json",
  "npm/wasm/package.json",
];

function readRepoFile(filePath: string): string {
  return fs.readFileSync(path.join(root, filePath), "utf-8");
}

function workspaceVersion(): string {
  const version = readRepoFile("Cargo.toml").match(/^version = "(.+)"$/m)?.[1];
  assert.ok(version, "workspace version");
  return version;
}

test("release package manifests stay aligned with the workspace version", () => {
  const version = workspaceVersion();
  const failures: string[] = [];

  for (const packageJsonPath of releasePackageJsonPaths) {
    const packageJson = JSON.parse(readRepoFile(packageJsonPath)) as {
      name?: string;
      version?: string;
    };
    if (packageJson.version !== version) {
      failures.push(
        `${packageJson.name ?? packageJsonPath}: version ${packageJson.version ?? "<missing>"} does not match ${version}`,
      );
    }
  }

  assert.deepEqual(failures, []);
});

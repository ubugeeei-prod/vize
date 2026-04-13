import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const npmDir = path.join(root, "npm");

function collectStrings(value: unknown, out: string[]): void {
  if (typeof value === "string") {
    out.push(value);
    return;
  }

  if (Array.isArray(value)) {
    for (const item of value) collectStrings(item, out);
    return;
  }

  if (value != null && typeof value === "object") {
    for (const item of Object.values(value)) collectStrings(item, out);
  }
}

function isEsmPackPackage(packageDir: string): boolean {
  const configPath = path.join(packageDir, "vite.config.ts");
  if (!fs.existsSync(configPath)) return false;

  const config = fs.readFileSync(configPath, "utf-8");
  return config.includes('format: "esm"') && config.includes("pack:");
}

test("esm packed npm manifests point at mjs and d.mts outputs", () => {
  const failures: string[] = [];

  for (const entry of fs.readdirSync(npmDir, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;

    const packageDir = path.join(npmDir, entry.name);
    if (!isEsmPackPackage(packageDir)) continue;

    const packageJsonPath = path.join(packageDir, "package.json");
    if (!fs.existsSync(packageJsonPath)) continue;

    const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf-8")) as {
      bin?: unknown;
      exports?: unknown;
      main?: unknown;
      name?: string;
      types?: unknown;
    };

    const publishedPaths: string[] = [];
    collectStrings(packageJson.main, publishedPaths);
    collectStrings(packageJson.types, publishedPaths);
    collectStrings(packageJson.bin, publishedPaths);
    collectStrings(packageJson.exports, publishedPaths);

    for (const publishedPath of publishedPaths) {
      if (!publishedPath.includes("dist/")) continue;

      if (publishedPath.endsWith(".js") || publishedPath.endsWith(".d.ts")) {
        failures.push(`${packageJson.name ?? entry.name}: ${publishedPath}`);
      }
    }
  }

  assert.deepEqual(failures, []);
});

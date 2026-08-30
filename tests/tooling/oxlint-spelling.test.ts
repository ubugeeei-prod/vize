import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { normalizeRepoText, root } from "./support/github-workflows.ts";

const USER_VISIBLE_LINT_SURFACES = [
  ".github",
  "CONTRIBUTING.md",
  "README.md",
  "docs",
  "examples",
  "npm/cli/package.json",
  "npm/cli/src",
  "npm/oxlint/package.json",
  "npm/oxlint/src",
  "npm/ui/core/package.json",
  "npm/ui/core/scripts",
  "npm/ui/tooling",
  "package.json",
  "pnpm-workspace.yaml",
] as const;

const TEXT_EXTENSIONS = new Set([
  ".cjs",
  ".cts",
  ".js",
  ".json",
  ".jsonc",
  ".md",
  ".mjs",
  ".mts",
  ".svg",
  ".ts",
  ".tsx",
  ".vue",
  ".yaml",
  ".yml",
]);

const IGNORED_DIRECTORIES = new Set([".git", ".vitepress", "dist", "node_modules"]);

test("user-visible oxlint docs and scripts do not spell oxlint as oxint", () => {
  const offenders: string[] = [];

  for (const relativePath of collectLintSurfaceFiles()) {
    const source = normalizeRepoText(fs.readFileSync(path.join(root, relativePath), "utf8"));
    const lines = source.split("\n");
    for (const [index, line] of lines.entries()) {
      if (/\boxint\b/iu.test(line)) {
        offenders.push(`${relativePath}:${index + 1}: ${line.trim()}`);
      }
    }
  }

  assert.deepEqual(offenders, []);
});

function collectLintSurfaceFiles(): string[] {
  return USER_VISIBLE_LINT_SURFACES.flatMap((surface) => collectFiles(surface)).sort();
}

function collectFiles(relativePath: string): string[] {
  const fullPath = path.join(root, relativePath);
  const stat = fs.statSync(fullPath);

  if (stat.isFile()) {
    return isTextFile(fullPath) ? [relativePath] : [];
  }

  const files: string[] = [];
  for (const entry of fs.readdirSync(fullPath, { withFileTypes: true })) {
    if (entry.isDirectory() && IGNORED_DIRECTORIES.has(entry.name)) continue;
    files.push(...collectFiles(path.join(relativePath, entry.name)));
  }
  return files;
}

function isTextFile(filename: string): boolean {
  return TEXT_EXTENSIONS.has(path.extname(filename));
}

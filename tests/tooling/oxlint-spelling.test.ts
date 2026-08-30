import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { normalizeRepoText, readRepoFile, root } from "./support/github-workflows.ts";

const CANONICAL_OXLINT_PACKAGE_DIR = "npm/oxlint";
const MISSPELLED_OXLINT_PACKAGE_DIR = path.posix.join("npm", "oxint");

const USER_VISIBLE_LINT_SURFACES = [
  ".github",
  "CONTRIBUTING.md",
  "README.md",
  "docs",
  "examples",
  "npm/cli/package.json",
  "npm/cli/src",
  CANONICAL_OXLINT_PACKAGE_DIR,
  "npm/ui/core/package.json",
  "npm/ui/core/scripts",
  "npm/ui/tooling",
  "package.json",
  "pnpm-workspace.yaml",
  "tools",
] as const;

const REPO_LOCAL_LINT_PATH_SURFACES = [
  ".github",
  "docs",
  "examples",
  "npm",
  "tests",
  "tools",
  "package.json",
  "pnpm-lock.yaml",
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

test("oxlint plugin package lives in the canonical package directory", () => {
  const canonicalDir = path.join(root, CANONICAL_OXLINT_PACKAGE_DIR);
  const misspelledDir = path.join(root, MISSPELLED_OXLINT_PACKAGE_DIR);
  const manifest = JSON.parse(
    readRepoFile(CANONICAL_OXLINT_PACKAGE_DIR, "package.json"),
  ) as OxlintPluginManifest;

  assert.equal(fs.statSync(canonicalDir).isDirectory(), true);
  assert.equal(fs.existsSync(misspelledDir), false);
  assert.equal(manifest.name, "oxlint-plugin-vize");
  assert.equal(manifest.repository?.directory, CANONICAL_OXLINT_PACKAGE_DIR);
});

test("repo-local oxlint package path references use the canonical spelling", () => {
  const offenders: string[] = [];

  for (const relativePath of collectLintPathFiles()) {
    const source = normalizeRepoText(fs.readFileSync(path.join(root, relativePath), "utf8"));
    const lines = source.split("\n");
    for (const [index, line] of lines.entries()) {
      if (line.includes(MISSPELLED_OXLINT_PACKAGE_DIR)) {
        offenders.push(`${relativePath}:${index + 1}: ${line.trim()}`);
      }
    }
  }

  assert.deepEqual(offenders, []);
});

type OxlintPluginManifest = {
  readonly name?: unknown;
  readonly repository?: {
    readonly directory?: unknown;
  };
};

function collectLintSurfaceFiles(): string[] {
  return USER_VISIBLE_LINT_SURFACES.flatMap((surface) => collectFiles(surface)).sort();
}

function collectLintPathFiles(): string[] {
  return REPO_LOCAL_LINT_PATH_SURFACES.flatMap((surface) => collectFiles(surface))
    .filter((relativePath) => relativePath !== "tests/tooling/oxlint-spelling.test.ts")
    .sort();
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

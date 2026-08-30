import fs from "node:fs";
import path from "node:path";

import { hasVueLikeExtension } from "../file-kinds.ts";

const GLOB_PATTERN = /[*?[\]{}]/u;

export function collectVueLikeFilesFromTargets(cwd: string, targets: readonly string[]): string[] {
  const files = new Set<string>();

  for (const target of targets) {
    for (const file of collectVueLikeFilesFromTarget(cwd, target)) {
      files.add(file);
    }
  }

  return [...files].sort();
}

function collectVueLikeFilesFromTarget(cwd: string, target: string): string[] {
  if (GLOB_PATTERN.test(target)) {
    return fs
      .globSync(target, {
        cwd,
        withFileTypes: false,
        exclude: ["**/node_modules/**", "**/.git/**"],
      })
      .map((entry) => path.resolve(cwd, entry))
      .filter(isSupportedLintFile);
  }

  const absoluteTarget = path.resolve(cwd, target);
  if (!fs.existsSync(absoluteTarget)) {
    return [];
  }

  const stat = fs.statSync(absoluteTarget);
  if (stat.isDirectory()) {
    return fs
      .globSync("**/*", {
        cwd: absoluteTarget,
        withFileTypes: false,
        exclude: ["**/node_modules/**", "**/.git/**"],
      })
      .map((entry) => path.resolve(absoluteTarget, entry))
      .filter(isSupportedLintFile);
  }

  return stat.isFile() && hasVueLikeExtension(absoluteTarget) ? [absoluteTarget] : [];
}

function isSupportedLintFile(filename: string): boolean {
  if (!hasVueLikeExtension(filename)) {
    return false;
  }

  try {
    return fs.statSync(filename).isFile();
  } catch {
    return false;
  }
}

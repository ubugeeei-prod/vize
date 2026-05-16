import fs from "node:fs";
import path from "node:path";

const GLOB_PATTERN = /[*?[\]{}]/u;

export function collectVueFilesFromTargets(cwd: string, targets: readonly string[]): string[] {
  const files = new Set<string>();

  for (const target of targets) {
    for (const file of collectVueFilesFromTarget(cwd, target)) {
      files.add(file);
    }
  }

  return [...files];
}

function collectVueFilesFromTarget(cwd: string, target: string): string[] {
  if (GLOB_PATTERN.test(target)) {
    return fs
      .globSync(target, {
        cwd,
        withFileTypes: false,
        exclude: ["**/node_modules/**", "**/.git/**"],
      })
      .map((entry) => path.resolve(cwd, entry))
      .filter(isVueFile);
  }

  const absoluteTarget = path.resolve(cwd, target);
  if (!fs.existsSync(absoluteTarget)) {
    return [];
  }

  const stat = fs.statSync(absoluteTarget);
  if (stat.isDirectory()) {
    return fs
      .globSync("**/*.vue", {
        cwd: absoluteTarget,
        withFileTypes: false,
        exclude: ["**/node_modules/**", "**/.git/**"],
      })
      .map((entry) => path.resolve(absoluteTarget, entry));
  }

  return isVueFile(absoluteTarget) ? [absoluteTarget] : [];
}

function isVueFile(filename: string): boolean {
  return filename.endsWith(".vue");
}

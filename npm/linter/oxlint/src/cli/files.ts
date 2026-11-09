import fs from "node:fs";
import path from "node:path";

import { hasVueLikeExtension } from "../file-kinds.ts";

const GLOB_PATTERN = /[*?[\]{}]/u;

export function collectVueLikeFilesFromTargets(
  cwd: string,
  targets: readonly string[],
  onCandidate?: (filename: string) => void,
): string[] {
  const files = new Set<string>();

  for (const target of targets) {
    for (const file of collectVueLikeFilesFromTarget(cwd, target, onCandidate)) {
      files.add(file);
    }
  }

  return [...files].sort();
}

function collectVueLikeFilesFromTarget(
  cwd: string,
  target: string,
  onCandidate?: (filename: string) => void,
): string[] {
  const observe = (filename: string) => {
    onCandidate?.(filename);
    return filename;
  };
  const absoluteTarget = observe(path.resolve(cwd, target));
  if (GLOB_PATTERN.test(target)) {
    return fs
      .globSync(target, {
        cwd,
        withFileTypes: false,
        exclude: ["**/node_modules/**", "**/.git/**"],
      })
      .map((entry) => observe(path.resolve(cwd, entry)))
      .filter(isSupportedLintFile);
  }

  if (!fs.existsSync(absoluteTarget)) {
    return [];
  }

  const stat = fs.statSync(absoluteTarget);
  if (stat.isDirectory()) {
    const files: string[] = [];
    walkDirectoryFiles(absoluteTarget, (file) => {
      observe(file);
      if (isSupportedLintFile(file)) files.push(file);
    });
    return files;
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

/**
 * Visit the same filesystem domain as a directory lint target, including dot
 * entries selected by Oxlint. Real directory identities bound symlink cycles;
 * unreadable or concurrently removed entries cannot invalidate other files.
 */
function walkDirectoryFiles(directory: string, visit: (file: string) => void): void {
  const pending = [directory];
  const visitedDirectories = new Set<string>();

  while (pending.length > 0) {
    const current = pending.pop() as string;
    let identity: string;
    let entries: fs.Dirent[];
    try {
      identity = fs.realpathSync(current);
      if (visitedDirectories.has(identity)) continue;
      visitedDirectories.add(identity);
      entries = fs.readdirSync(current, { withFileTypes: true });
    } catch {
      continue;
    }

    for (const entry of entries) {
      if (entry.name === ".git" || entry.name === "node_modules") continue;
      const file = path.join(current, entry.name);
      if (entry.isDirectory()) {
        visit(file);
        pending.push(file);
        continue;
      }
      if (!entry.isSymbolicLink()) {
        if (entry.isFile()) visit(file);
        continue;
      }
      try {
        const target = fs.statSync(file);
        if (target.isDirectory()) {
          visit(file);
          pending.push(file);
        } else if (target.isFile()) visit(file);
      } catch {
        // A dangling or concurrently removed link is not an Oxlint candidate.
      }
    }
  }
}

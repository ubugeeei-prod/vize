import fs from "node:fs";
import path from "node:path";

import { hasVueLikeExtension } from "../file-kinds.ts";

const GLOB_PATTERN = /[*?[\]{}]/u;

/**
 * Resolves lint targets (files, directories or globs) to the Vue-like files among them,
 * reporting every path it visits, including non-Vue ones, through `onCandidate`.
 */
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

/** Resolves one lint target; directory targets are walked with dot-entries included. */
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
    return walkDirectory(absoluteTarget)
      .map((entry) => observe(entry))
      .filter(isSupportedLintFile);
  }

  return stat.isFile() && hasVueLikeExtension(absoluteTarget) ? [absoluteTarget] : [];
}

/** Whether `filename` is an existing regular file with a Vue-like extension. */
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
 * Lists every entry (files and directories) below `directory`, dot-files and dot-directories included.
 * `fs.globSync("**\/*")` skips them, but Oxlint lints files such as
 * `.lintstagedrc.cjs`; leaving them out made the selected set larger than the
 * candidate set and aborted the whole Vue transport for a `.` target.
 */
function walkDirectory(directory: string): string[] {
  const files: string[] = [];
  const pending = [directory];
  const seen = new Set<string>();
  while (pending.length > 0) {
    const current = pending.pop() as string;
    let real: string;
    try {
      real = fs.realpathSync(current);
    } catch {
      continue;
    }
    if (seen.has(real)) continue;
    seen.add(real);
    let entries: fs.Dirent[];
    try {
      entries = fs.readdirSync(current, { withFileTypes: true });
    } catch {
      // Unreadable (permissions, removed mid-walk): skip this directory, keep walking.
      continue;
    }
    for (const entry of entries) {
      if (entry.name === "node_modules" || entry.name === ".git") continue;
      const full = path.join(current, entry.name);
      let isDirectory = entry.isDirectory();
      if (entry.isSymbolicLink()) {
        try {
          isDirectory = fs.statSync(full).isDirectory();
        } catch {
          continue;
        }
      }
      // Directories are listed too: `fs.globSync` did, and callers bind them as candidates.
      files.push(full);
      if (isDirectory) pending.push(full);
    }
  }
  return files;
}

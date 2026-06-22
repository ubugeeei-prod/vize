import fs from "node:fs";
import path from "node:path";

import { matchGlob, resolveScanRoots, shouldProcess } from "../utils.js";

export function isStorybookTsxInput(filePath: string): boolean {
  return /\.(?:stories|story)\.tsx$/i.test(filePath);
}

export async function scanStorybookTsxInputs(
  root: string,
  include: string[],
  exclude: string[],
): Promise<string[]> {
  const files = new Set<string>();
  const visitedDirs = new Set<string>();

  async function scan(dir: string): Promise<void> {
    const resolvedDir = path.resolve(dir);
    if (visitedDirs.has(resolvedDir)) return;
    visitedDirs.add(resolvedDir);

    let entries: fs.Dirent[];
    try {
      entries = await fs.promises.readdir(resolvedDir, { withFileTypes: true });
    } catch {
      return;
    }

    for (const entry of entries) {
      const fullPath = path.join(resolvedDir, entry.name);
      if (isExcluded(fullPath, entry.name, exclude, root)) continue;

      if (entry.isDirectory()) {
        await scan(fullPath);
      } else if (
        entry.isFile() &&
        isStorybookTsxInput(entry.name) &&
        shouldProcess(fullPath, include, exclude, root)
      ) {
        files.add(fullPath);
      }
    }
  }

  for (const scanRoot of resolveScanRoots(root, include)) {
    await scan(scanRoot);
  }

  return [...files].sort();
}

export async function assertNoUnsupportedStorybookTsxInputs(
  root: string,
  include: string[],
  exclude: string[],
): Promise<void> {
  const files = await scanStorybookTsxInputs(root, include, exclude);
  if (files.length > 0) {
    throw new Error(formatUnsupportedStorybookTsxInputError(root, files));
  }
}

export function formatUnsupportedStorybookTsxInputError(root: string, files: string[]): string {
  const shown = files
    .slice(0, 5)
    .map((file) => `  - ${relativePath(root, file)}`)
    .join("\n");
  const remaining = files.length > 5 ? `\n  ... and ${files.length - 5} more` : "";

  return [
    "[musea] Storybook TSX files matched by include are not supported as Musea art inputs yet.",
    "Remove *.stories.tsx from musea.include or migrate those stories to .art.vue files.",
    "Matched files:",
    `${shown}${remaining}`,
  ].join("\n");
}

function isExcluded(filePath: string, name: string, exclude: string[], root: string): boolean {
  return exclude.some(
    (pattern) => matchesPathPattern(filePath, pattern, root) || matchGlob(name, pattern),
  );
}

function matchesPathPattern(filePath: string, pattern: string, root: string): boolean {
  const candidate = path.isAbsolute(pattern)
    ? path.resolve(filePath)
    : path.relative(root, filePath);
  return matchGlob(candidate, pattern);
}

function relativePath(root: string, filePath: string): string {
  return path.relative(root, filePath).split(path.sep).join("/");
}

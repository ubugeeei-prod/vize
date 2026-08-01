import path from "node:path";

import type { ResolvedVizeNuxtLintCheckerOptions } from "./options.ts";

const GLOB_TOKEN = /[*?[\]{}()]/u;

function normalize(value: string): string {
  const normalized = value.replaceAll("\\", "/").replace(/\/{2,}/gu, "/");
  if (normalized === "/" || /^[A-Za-z]:\/$/u.test(normalized)) return normalized;
  return normalized.replace(/\/$/u, "");
}

function relativeToRoot(file: string, rootDir: string): string | undefined {
  const root = normalize(rootDir);
  const absolute = normalize(file);
  const prefix = `${root}/`;
  if (absolute.startsWith(prefix)) return absolute.slice(prefix.length);

  const relative = path.relative(rootDir, file);
  const normalized = normalize(relative);
  return normalized === ".." || normalized.startsWith("../") ? undefined : normalized;
}

function matchesDirectory(candidate: string, pattern: string): boolean {
  if (GLOB_TOKEN.test(pattern)) return false;
  return candidate === pattern || candidate.startsWith(`${pattern}/`);
}

function matchesPattern(
  absolute: string,
  relative: string | undefined,
  rawPattern: string,
): boolean {
  const pattern = normalize(rawPattern);
  const absolutePattern = path.isAbsolute(pattern) || /^[A-Za-z]:\//u.test(pattern);
  const candidates = absolutePattern ? [absolute] : relative === undefined ? [] : [relative];
  for (const candidate of candidates) {
    if (matchesDirectory(candidate, pattern) || path.matchesGlob(candidate, pattern)) return true;
  }
  return false;
}

/** Whether one watcher path belongs to the checker include/exclude contract. */
export function matchesNuxtLintCheckerFile(
  file: string,
  rootDir: string,
  options: Pick<ResolvedVizeNuxtLintCheckerOptions, "exclude" | "include">,
): boolean {
  const absolute = normalize(file);
  const relative = relativeToRoot(file, rootDir);
  if (options.exclude.some((pattern) => matchesPattern(absolute, relative, pattern))) return false;
  return options.include.some((pattern) => matchesPattern(absolute, relative, pattern));
}

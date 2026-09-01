import fs from "node:fs";
import path from "node:path";

import { HttpError } from "./http-error.js";

function realpathNearest(targetPath: string): string {
  let current = path.resolve(targetPath);
  const missingParts: string[] = [];

  while (true) {
    try {
      const real = fs.realpathSync.native(current);
      return missingParts.length > 0 ? path.join(real, ...missingParts.reverse()) : real;
    } catch {
      const parent = path.dirname(current);
      if (parent === current) {
        return path.resolve(targetPath);
      }
      missingParts.push(path.basename(current));
      current = parent;
    }
  }
}

function isResolvedPathInside(parentDir: string, candidatePath: string): boolean {
  const parent = path.resolve(parentDir);
  const candidate = path.resolve(candidatePath);
  const relative = path.relative(parent, candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

export function isPathInside(parentDir: string, candidatePath: string): boolean {
  return isResolvedPathInside(realpathNearest(parentDir), realpathNearest(candidatePath));
}

export function isPathInsideAny(parentDirs: string[], candidatePath: string): boolean {
  const candidate = realpathNearest(candidatePath);
  return parentDirs.some((parentDir) =>
    isResolvedPathInside(realpathNearest(parentDir), candidate),
  );
}

/**
 * True when `candidatePath` stays inside the project, or inside an extra root
 * that was configured to live outside the project.
 *
 * Extra roots whose lexical path is still inside the project are not trusted
 * after realpath — a planted `src` → `/etc` symlink must not widen the
 * readable boundary.
 */
export function isTrustedSourcePath(
  projectRoot: string,
  extraLexicalRoots: readonly string[],
  candidatePath: string,
): boolean {
  const lexicalProject = path.resolve(projectRoot);
  const realProject = realpathNearest(projectRoot);
  const realCandidate = realpathNearest(candidatePath);

  if (isResolvedPathInside(realProject, realCandidate)) {
    return true;
  }

  for (const lexicalRoot of extraLexicalRoots) {
    const resolvedLexical = path.resolve(lexicalRoot);
    if (isResolvedPathInside(lexicalProject, resolvedLexical)) {
      continue;
    }
    if (isResolvedPathInside(realpathNearest(resolvedLexical), realCandidate)) {
      return true;
    }
  }

  return false;
}

export function resolveTrustedSourcePath(
  projectRoot: string,
  extraLexicalRoots: readonly string[],
  candidatePath: string,
  label = "path",
): string {
  if (candidatePath.includes("\0")) {
    throw new HttpError(`${label} contains an invalid character`, 400);
  }

  const resolved = path.isAbsolute(candidatePath)
    ? path.resolve(candidatePath)
    : path.resolve(path.resolve(projectRoot), candidatePath);

  if (!isTrustedSourcePath(projectRoot, extraLexicalRoots, resolved)) {
    throw new HttpError(`${label} escapes the allowed directory`, 400);
  }

  return resolved;
}

export function resolveInside(parentDir: string, candidatePath: string, label = "path"): string {
  return resolveInsideAny([parentDir], candidatePath, label);
}

/**
 * Resolve a Vue source path that stays inside `parentDir` and still has a
 * `.vue` suffix after realpath. A planted `Evil.vue` → `.env` symlink must
 * not be readable just because the request path ends in `.vue`.
 */
export function resolveInsideVueFile(
  parentDir: string,
  candidatePath: string,
  label = "path",
): string {
  return requireVueRealpath(resolveInside(parentDir, candidatePath, label), candidatePath, label);
}

/**
 * Like `resolveTrustedSourcePath`, but the request path and the realpath
 * target must both keep a `.vue` suffix. Art-declared `component` fields
 * that point at `Evil.vue` → `.env` must not be readable.
 */
export function resolveTrustedVueSourcePath(
  projectRoot: string,
  extraLexicalRoots: readonly string[],
  candidatePath: string,
  label = "path",
): string {
  return requireVueRealpath(
    resolveTrustedSourcePath(projectRoot, extraLexicalRoots, candidatePath, label),
    candidatePath,
    label,
  );
}

function requireVueRealpath(resolved: string, candidatePath: string, label: string): string {
  if (!candidatePath.endsWith(".vue")) {
    throw new HttpError(`${label} must be a .vue file`, 400);
  }

  let real: string;
  try {
    real = fs.realpathSync.native(resolved);
  } catch {
    throw new HttpError(`${label} must be a readable .vue file`, 400);
  }
  if (!real.endsWith(".vue")) {
    throw new HttpError(`${label} must be a .vue file`, 400);
  }

  return resolved;
}

export function resolveInsideAny(
  parentDirs: string[],
  candidatePath: string,
  label = "path",
): string {
  if (candidatePath.includes("\0")) {
    throw new HttpError(`${label} contains an invalid character`, 400);
  }

  if (parentDirs.length === 0) {
    throw new HttpError(`No allowed directories configured for ${label}`, 500);
  }

  const parent = path.resolve(parentDirs[0] ?? ".");
  const resolved = path.isAbsolute(candidatePath)
    ? path.resolve(candidatePath)
    : path.resolve(parent, candidatePath);

  if (!isPathInsideAny(parentDirs, resolved)) {
    throw new HttpError(`${label} escapes the allowed directory`, 400);
  }

  return resolved;
}

export function resolveUrlPathInside(
  parentDir: string,
  requestUrl: string,
  label = "path",
): string {
  const rawPath = requestUrl.split(/[?#]/, 1)[0] || "/";
  let pathname = decodeUrlComponent(rawPath, label);

  pathname = pathname.replaceAll("\\", "/");
  if (pathname.split("/").includes("..")) {
    throw new HttpError(`${label} must not contain parent directory segments`, 400);
  }

  const relativePath = `.${pathname}`;
  return resolveInside(parentDir, relativePath, label);
}

export function decodeUrlComponent(value: string, label = "path"): string {
  try {
    return decodeURIComponent(value);
  } catch {
    throw new HttpError(`${label} is not valid URL encoding`, 400);
  }
}

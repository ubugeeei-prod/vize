// Lint a fixture where nothing above it provides packages.
//
// A fixture under `tests/_fixtures/_git` resolves imports by walking up to the
// repository's own `tests/node_modules` and `node_modules`, so Patina's type
// resolution could read a package version the fixture never installed. Both
// providers instead lint a mirror of the fixture whose ancestors hold no
// `node_modules`: files are hard links (copies where linking fails), symbolic
// links keep their authored targets, and each installed `node_modules` is one
// link to the fixture's own, so packages resolve exactly as the fixture's
// install provides them.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { Json } from "./value.ts";

/**
 * A directory for mirrors whose ancestors hold no `node_modules`. Mirrors on
 * another device copy instead of linking, which is slower but equivalent.
 */
export function isolationRoot(base = os.tmpdir()): string {
  const root = path.join(base, "vize-lint-divergence");
  fs.mkdirSync(root, { recursive: true });
  const ancestor = ancestorWithPackages(root);
  if (ancestor !== null)
    throw new Error(
      `${root} resolves packages from ${ancestor}; set TMPDIR outside any package tree`,
    );
  return root;
}

/** The nearest ancestor of `directory` (itself included) with `node_modules`. */
export function ancestorWithPackages(directory: string): string | null {
  for (let current = path.resolve(directory); ; current = path.dirname(current)) {
    if (fs.existsSync(path.join(current, "node_modules"))) return current;
    if (path.dirname(current) === current) return null;
  }
}

/** `value` with every path string under `from` moved under `to`. */
export function relocatePaths(value: Json, from: string, to: string): Json {
  if (typeof value === "string")
    return value === from || value.startsWith(from + path.sep)
      ? to + value.slice(from.length)
      : value;
  if (Array.isArray(value)) return value.map((item) => relocatePaths(item, from, to));
  if (value !== null && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, relocatePaths(item, from, to)]),
    );
  return value;
}

/** Run `run` with a mirror of `fixtureDir` under `rootDir`, then remove it. */
export function withIsolatedFixture<Result>(
  fixtureDir: string,
  rootDir: string,
  id: string,
  run: (mirrorDir: string) => Result,
): Result {
  const mirrorDir = fs.mkdtempSync(path.join(rootDir, `${id}-`));
  try {
    mirrorFixture(fixtureDir, mirrorDir);
    return run(mirrorDir);
  } finally {
    fs.rmSync(mirrorDir, { recursive: true, force: true });
  }
}

export function mirrorFixture(fixtureDir: string, mirrorDir: string): void {
  const walk = (relative: string): void => {
    for (const entry of fs.readdirSync(path.join(fixtureDir, relative), { withFileTypes: true })) {
      if (entry.name === ".git") continue;
      const child = path.join(relative, entry.name);
      const source = path.join(fixtureDir, child),
        target = path.join(mirrorDir, child);
      if (entry.isSymbolicLink()) fs.symlinkSync(fs.readlinkSync(source), target);
      else if (entry.isDirectory() && entry.name === "node_modules")
        fs.symlinkSync(source, target, "junction");
      else if (entry.isDirectory()) {
        fs.mkdirSync(target);
        walk(child);
      } else if (entry.isFile()) {
        try {
          fs.linkSync(source, target);
        } catch {
          fs.copyFileSync(source, target);
        }
      }
    }
  };
  walk("");
}

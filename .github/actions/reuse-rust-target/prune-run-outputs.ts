// A reused target directory must look like a fresh one except for Cargo's own
// build cache. Tests and tools also write run outputs under `target/`
// (differential evidence, staging projects, receipts, nextest reports); left on
// a sticky disk they would accumulate and could be uploaded or read as if this
// run had produced them. Everything that is not Cargo's cache is removed.
import { existsSync, readdirSync, rmSync } from "node:fs";
import type { Dirent } from "node:fs";
import { join } from "node:path";

import { manifestName } from "./source-mtimes.ts";

// Cargo's own state inside one profile directory (`target/<profile>` or
// `target/<triple>/<profile>`). Top-level files there are uplifted copies that
// Cargo restores for fresh units without recompiling.
const profileCache = new Set([".fingerprint", "build", "deps", "examples", "incremental"]);
// Cargo's own files at the root of a target directory.
const targetFiles = new Set([".rustc_info.json", "CACHEDIR.TAG", manifestName]);
// target/<triple>/<profile>/.fingerprint is the deepest root a nested target can have.
const maxProfileDepth = 3;

const entries = (directory: string): Dirent[] => {
  try {
    return readdirSync(directory, { withFileTypes: true });
  } catch {
    return [];
  }
};

const isProfile = (directory: string) => existsSync(join(directory, ".fingerprint"));

function containsProfile(directory: string, depth: number): boolean {
  if (isProfile(directory)) return true;
  if (depth === 0) return false;
  return entries(directory).some(
    (entry) => entry.isDirectory() && containsProfile(join(directory, entry.name), depth - 1),
  );
}

export function pruneRunOutputs(target: string): string[] {
  const removed: string[] = [];
  const remove = (path: string) => {
    rmSync(path, { recursive: true, force: true });
    removed.push(path);
  };
  const pruneProfile = (profile: string) => {
    for (const entry of entries(profile)) {
      const path = join(profile, entry.name);
      // Uplifted binaries and dep-info files are relinked by Cargo while fresh,
      // so only the cache directories stay; receipts written beside them go.
      if (!(entry.isDirectory() && profileCache.has(entry.name))) remove(path);
    }
  };
  const pruneTarget = (directory: string, depth: number) => {
    for (const entry of entries(directory)) {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        if (isProfile(path)) pruneProfile(path);
        // A target triple or a nested target directory such as llvm-cov-target.
        else if (depth < maxProfileDepth && containsProfile(path, maxProfileDepth - depth - 1))
          pruneTarget(path, depth + 1);
        else remove(path);
      } else if (!targetFiles.has(entry.name)) remove(path);
    }
  };
  pruneTarget(target, 0);
  return removed;
}

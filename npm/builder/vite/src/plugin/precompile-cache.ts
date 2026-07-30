/**
 * Persistent (on-disk) pre-compile cache.
 *
 * `state.precompileMetadata` plus `state.cache` already skip recompilation
 * inside one process, but both are empty on every process start, so `vite
 * build`, CI, and each dev-server start recompile the whole scan from scratch.
 * This module backs them with a manifest under
 * `node_modules/.vize/vite-precompile/` so a cold process can restore the
 * previous run's output.
 *
 * The two invalidation gates -- manifest identity and per-entry source hash --
 * live in `./precompile-cache-key.ts`, which documents why each one is safe.
 * Two more gates live here:
 *
 * - **Shape.** Entries are validated before use and dropped individually if
 *   they do not describe a complete `CompiledModule`. The manifest's own
 *   `format`/`key` are re-checked after parsing, so a manifest reached by any
 *   route other than its key still gets rejected.
 * - **`src` imports.** SFCs that pull blocks in through `<script src>` /
 *   `<template src>` / `<style src>` are never persisted: their output depends
 *   on files whose content this cache does not hash.
 *
 * A missing, truncated, or corrupt manifest degrades to a full recompile: every
 * read is guarded and any failure yields an empty cache. Writes go through a
 * sibling temp file and a rename, so an interrupted write cannot be read back.
 *
 * Set `VIZE_PRECOMPILE_CACHE=0` to force a full recompile without editing any
 * config.
 */

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

import type { CompiledModule } from "../types.ts";
import { PRECOMPILE_CACHE_FORMAT, computePrecompileCacheKey } from "./precompile-cache-key.ts";

export { hashPrecompileSource, PRECOMPILE_CACHE_FORMAT } from "./precompile-cache-key.ts";

/** Manifest location, relative to the Vite root. */
export const PRECOMPILE_CACHE_DIR = path.join("node_modules", ".vize", "vite-precompile");

/** Set to `0`/`false` to force a full recompile without editing the config. */
export const PRECOMPILE_CACHE_ENV = "VIZE_PRECOMPILE_CACHE";

type Diagnostic = (message: string, error?: unknown) => void;

interface PrecompileCacheEntry {
  hash: string;
  module: CompiledModule;
}

interface PrecompileCacheManifest {
  format: number;
  key: string;
  entries: Record<string, PrecompileCacheEntry>;
}

export interface PrecompileCache {
  /** Absolute manifest path, or `null` when the cache is disabled. */
  readonly file: string | null;
  /** Compiled module for `file` when the persisted source hash matches. */
  get(file: string, sourceHash: string): CompiledModule | undefined;
  /** Stage `module` for the next process. No-op for non-persistable modules. */
  set(file: string, sourceHash: string, module: CompiledModule): void;
  /** Forget `file` (compile failure, unreadable source, deleted file). */
  delete(file: string): void;
  /** Drop every entry outside `files`, so deleted files do not accumulate. */
  retain(files: Iterable<string>): void;
  /** Write the manifest atomically when something changed. Never throws. */
  flush(): boolean;
}

/**
 * Whether `module` may be persisted.
 *
 * Modules assembled from `src` imports depend on sibling files that this cache
 * does not hash, so they are recompiled on every cold start instead.
 */
export function isPersistablePrecompileModule(module: CompiledModule): boolean {
  return !module.dependencies || module.dependencies.length === 0;
}

function isCompiledModule(value: unknown): value is CompiledModule {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const module = value as Partial<CompiledModule>;
  if (typeof module.code !== "string" || typeof module.scopeId !== "string") {
    return false;
  }
  if (typeof module.hasScoped !== "boolean") {
    return false;
  }
  if (module.css !== undefined && typeof module.css !== "string") {
    return false;
  }
  if (module.styles !== undefined && !Array.isArray(module.styles)) {
    return false;
  }
  if (module.macroArtifacts !== undefined && !Array.isArray(module.macroArtifacts)) {
    return false;
  }
  // `src`-import modules are never written; reject them if one shows up anyway.
  return isPersistablePrecompileModule(module as CompiledModule);
}

function isCacheEntry(value: unknown): value is PrecompileCacheEntry {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const entry = value as Partial<PrecompileCacheEntry>;
  return typeof entry.hash === "string" && entry.hash.length > 0 && isCompiledModule(entry.module);
}

/** Whether the environment forces the cache off. */
export function isPrecompileCacheDisabledByEnv(env: NodeJS.ProcessEnv = process.env): boolean {
  const value = env[PRECOMPILE_CACHE_ENV];
  return value === "0" || value === "false";
}

const disabledCache: PrecompileCache = {
  file: null,
  get: () => undefined,
  set: () => {},
  delete: () => {},
  retain: () => {},
  flush: () => false,
};

/** A cache that never hits and never writes. */
export function createDisabledPrecompileCache(): PrecompileCache {
  return disabledCache;
}

export interface OpenPrecompileCacheOptions {
  /** Vite root; the manifest lives under `<root>/node_modules/.vize`. */
  root: string;
  /** Resolved native batch compile options for this run. */
  compileOptions: unknown;
  /** Reports read/write problems without failing the build. */
  onDiagnostic?: Diagnostic;
  env?: NodeJS.ProcessEnv;
}

export function openPrecompileCache(options: OpenPrecompileCacheOptions): PrecompileCache {
  const { root, compileOptions, onDiagnostic, env = process.env } = options;
  if (!root || isPrecompileCacheDisabledByEnv(env)) {
    return createDisabledPrecompileCache();
  }

  const key = computePrecompileCacheKey(compileOptions);
  const file = path.join(root, PRECOMPILE_CACHE_DIR, `${key}.json`);
  const entries = readManifestEntries(file, key, onDiagnostic);
  let dirty = false;

  return {
    file,
    get(filePath, sourceHash) {
      const entry = entries.get(filePath);
      return entry && entry.hash === sourceHash ? entry.module : undefined;
    },
    set(filePath, sourceHash, module) {
      if (!isPersistablePrecompileModule(module)) {
        if (entries.delete(filePath)) {
          dirty = true;
        }
        return;
      }
      const existing = entries.get(filePath);
      if (existing?.hash === sourceHash && existing.module === module) {
        return;
      }
      entries.set(filePath, { hash: sourceHash, module });
      dirty = true;
    },
    delete(filePath) {
      if (entries.delete(filePath)) {
        dirty = true;
      }
    },
    retain(files) {
      const keep = files instanceof Set ? files : new Set(files);
      // Deleting the entry a Map iterator is currently on is well defined.
      for (const filePath of entries.keys()) {
        if (!keep.has(filePath)) {
          entries.delete(filePath);
          dirty = true;
        }
      }
    },
    flush() {
      if (!dirty) {
        return false;
      }
      const manifest: PrecompileCacheManifest = {
        format: PRECOMPILE_CACHE_FORMAT,
        key,
        entries: Object.fromEntries(entries),
      };
      if (!writeManifest(file, manifest, onDiagnostic)) {
        return false;
      }
      dirty = false;
      return true;
    },
  };
}

/**
 * Parse the manifest, or return an empty map.
 *
 * A missing, truncated, corrupt, or foreign manifest is indistinguishable from
 * no cache at all, which is exactly the safe outcome: recompile everything.
 */
function readManifestEntries(
  file: string,
  key: string,
  onDiagnostic?: Diagnostic,
): Map<string, PrecompileCacheEntry> {
  const entries = new Map<string, PrecompileCacheEntry>();
  let raw: string;
  try {
    raw = fs.readFileSync(file, "utf-8");
  } catch {
    return entries;
  }

  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (error) {
    onDiagnostic?.(`Discarding corrupt pre-compile cache ${file}:`, error);
    return entries;
  }

  const manifest = parsed as Partial<PrecompileCacheManifest> | null;
  if (
    typeof manifest !== "object" ||
    manifest === null ||
    manifest.format !== PRECOMPILE_CACHE_FORMAT ||
    manifest.key !== key ||
    typeof manifest.entries !== "object" ||
    manifest.entries === null
  ) {
    onDiagnostic?.(`Ignoring pre-compile cache ${file}: unrecognized manifest`);
    return entries;
  }

  for (const [filePath, entry] of Object.entries(manifest.entries)) {
    if (isCacheEntry(entry)) {
      entries.set(filePath, entry);
    }
  }
  return entries;
}

/** Write through a sibling temp file so a crash cannot leave a partial manifest. */
function writeManifest(
  file: string,
  manifest: PrecompileCacheManifest,
  onDiagnostic?: Diagnostic,
): boolean {
  const temp = `${file}.${process.pid}.${crypto.randomBytes(4).toString("hex")}.tmp`;
  try {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(temp, JSON.stringify(manifest));
    fs.renameSync(temp, file);
    return true;
  } catch (error) {
    try {
      fs.rmSync(temp, { force: true });
    } catch {
      // best effort
    }
    onDiagnostic?.(`Failed to write pre-compile cache ${file}:`, error);
    return false;
  }
}

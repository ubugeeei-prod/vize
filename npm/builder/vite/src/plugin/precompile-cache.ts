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
 * Two more gates live in `./precompile-cache-store.ts`, which owns the on-disk
 * container:
 *
 * - **Shape.** Entries are validated before use and dropped individually if
 *   they do not describe a complete `CompiledModule`. The container's own
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
import { computePrecompileCacheKey } from "./precompile-cache-key.ts";
import {
  PRECOMPILE_CACHE_EXTENSION,
  decodePrecompileManifest,
  encodePrecompileManifest,
  isPersistablePrecompileModule,
  type PrecompileCacheEntry,
} from "./precompile-cache-store.ts";

export { hashPrecompileSource, PRECOMPILE_CACHE_FORMAT } from "./precompile-cache-key.ts";
export {
  PRECOMPILE_CACHE_EXTENSION,
  decodePrecompileManifest,
  encodePrecompileManifest,
  isPersistablePrecompileModule,
} from "./precompile-cache-store.ts";

/** Manifest location, relative to the Vite root. */
export const PRECOMPILE_CACHE_DIR = path.join("node_modules", ".vize", "vite-precompile");

/** Set to `0`/`false` to force a full recompile without editing the config. */
export const PRECOMPILE_CACHE_ENV = "VIZE_PRECOMPILE_CACHE";

type Diagnostic = (message: string, error?: unknown) => void;

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
  const file = path.join(root, PRECOMPILE_CACHE_DIR, `${key}${PRECOMPILE_CACHE_EXTENSION}`);
  const entries = readManifestEntries(file, key, root, onDiagnostic);
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
      let container: Buffer;
      try {
        container = encodePrecompileManifest({ key, root, entries });
      } catch (error) {
        onDiagnostic?.(`Failed to encode pre-compile cache ${file}:`, error);
        return false;
      }
      if (!writeManifest(file, container, onDiagnostic)) {
        return false;
      }
      removeFormat1Manifests(path.dirname(file));
      dirty = false;
      return true;
    },
  };
}

/**
 * Drop the `.json` manifests format 1 left behind.
 *
 * Nothing can read them any more, and they are the reason this change exists:
 * ~9 KB per SFC, so ~27 MB for a 3000-SFC project sitting in `node_modules`
 * forever after an upgrade. Only `.json` is removed, which is exactly the set
 * format 1 wrote; every live manifest is a `.vpc`, and a project legitimately
 * keeps one per compile-option set. Best effort -- a failure here is not worth
 * a diagnostic, let alone a failed build.
 */
function removeFormat1Manifests(dir: string): void {
  try {
    for (const name of fs.readdirSync(dir)) {
      if (name.endsWith(".json")) {
        fs.rmSync(path.join(dir, name), { force: true });
      }
    }
  } catch {
    // best effort
  }
}

/**
 * Read the container, or return an empty map.
 *
 * A missing, truncated, corrupt, or foreign manifest is indistinguishable from
 * no cache at all, which is exactly the safe outcome: recompile everything.
 */
function readManifestEntries(
  file: string,
  key: string,
  root: string,
  onDiagnostic?: Diagnostic,
): Map<string, PrecompileCacheEntry> {
  let bytes: Buffer;
  try {
    bytes = fs.readFileSync(file);
  } catch {
    return new Map();
  }
  return (
    decodePrecompileManifest(bytes, {
      key,
      root,
      onReject: (reason) => onDiagnostic?.(`Ignoring pre-compile cache ${file}: ${reason}`),
    }) ?? new Map()
  );
}

/** Write through a sibling temp file so a crash cannot leave a partial manifest. */
function writeManifest(file: string, container: Buffer, onDiagnostic?: Diagnostic): boolean {
  const temp = `${file}.${process.pid}.${crypto.randomBytes(4).toString("hex")}.tmp`;
  try {
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(temp, container);
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

/**
 * Shared harness for the persistent pre-compile cache suites.
 *
 * A fresh `VizePluginState` is a fresh process as far as this cache is
 * concerned: `state.cache` and `state.precompileMetadata` start empty, so
 * anything a second `coldRun` reuses came off disk. Both cache suites --
 * staleness in `plugin/precompile-cache.test.ts` and container damage in
 * `plugin/precompile-cache-corrupt.test.ts` -- drive `compileAll` that way, so
 * the setup lives here rather than in either of them.
 */

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { compileAll } from "../plugin/precompile-run.ts";
import {
  PRECOMPILE_CACHE_DIR,
  PRECOMPILE_CACHE_EXTENSION,
  decodePrecompileManifest,
} from "../plugin/precompile-cache.ts";
import { DEFAULT_PRECOMPILE_BATCH_SIZE, type VizePluginState } from "../plugin/state.ts";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testRoot = path.resolve(
  __dirname,
  "../../../../target/vize-tests/tests/vite-plugin-vize/precompile-cache",
);
fs.rmSync(testRoot, { recursive: true, force: true });
fs.mkdirSync(testRoot, { recursive: true });

export const BASE_SOURCE = `<template><div class="a">one</div></template>
<script setup>const label = "one";</script>
<style scoped>.a { color: red; }</style>
`;

let caseId = 0;

/** A throwaway project with one `src/App.vue`. */
export function makeProject(source: string): { root: string; file: string } {
  const root = path.join(testRoot, `case-${++caseId}`);
  const srcDir = path.join(root, "src");
  fs.mkdirSync(srcDir, { recursive: true });
  const file = path.join(srcDir, "App.vue");
  fs.writeFileSync(file, source);
  return { root, file };
}

export function makeState(
  root: string,
  options: VizePluginState["mergedOptions"] = {},
  info: (line: string) => void = () => {},
): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: ["src/**/*.vue"],
    precompileBatchSize: DEFAULT_PRECOMPILE_BATCH_SIZE,
    ignorePatterns: [],
    mergedOptions: options,
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: {
      log() {},
      info: (...args: unknown[]) => info(args.join(" ")),
      warn() {},
      error() {},
    } as never,
  };
}

/** Runs one cold `compileAll` (fresh state == fresh process) and reports the log. */
export async function coldRun(
  root: string,
  options: VizePluginState["mergedOptions"] = {},
): Promise<{ state: VizePluginState; log: string }> {
  const lines: string[] = [];
  const state = makeState(root, options, (line) => lines.push(line));
  await compileAll(state);
  return { state, log: lines.join("\n") };
}

function counted(log: string, pattern: RegExp): number {
  return Number(pattern.exec(log)?.[1] ?? "-1");
}

export function restoredCount(log: string): number {
  return counted(log, /(\d+) restored from disk/);
}

export function recompiledCount(log: string): number {
  return counted(log, /(\d+) recompiled/);
}

export function manifestFiles(root: string): string[] {
  const dir = path.join(root, PRECOMPILE_CACHE_DIR);
  const ext = PRECOMPILE_CACHE_EXTENSION;
  return fs.existsSync(dir) ? fs.readdirSync(dir).filter((name) => name.endsWith(ext)) : [];
}

/** The one container under `root`. Throws rather than picking by `readdirSync` order. */
export function manifestPath(root: string): string {
  const files = manifestFiles(root);
  if (files.length !== 1) {
    throw new Error(`expected exactly one container under ${root}, found ${files.length}`);
  }
  return path.join(root, PRECOMPILE_CACHE_DIR, files[0]!);
}

/** The cache key a container is filed under, taken from its file name. */
export function manifestKey(root: string): string {
  return path.basename(manifestPath(root), PRECOMPILE_CACHE_EXTENSION);
}

/** Which files the container on disk actually holds entries for. */
export function persistedFiles(root: string): string[] {
  const bytes = fs.readFileSync(manifestPath(root));
  const entries = decodePrecompileManifest(bytes, { key: manifestKey(root), root });
  return [...(entries?.keys() ?? [])].sort();
}

/** Rewrite the container's header line, leaving both compressed bodies intact. */
export function patchManifestHeader(root: string, patch: Record<string, unknown>): void {
  const file = manifestPath(root);
  const bytes = fs.readFileSync(file);
  const end = bytes.indexOf(0x0a);
  const header = { ...(JSON.parse(bytes.toString("utf8", 0, end)) as object), ...patch };
  fs.writeFileSync(file, Buffer.concat([Buffer.from(JSON.stringify(header)), bytes.subarray(end)]));
}

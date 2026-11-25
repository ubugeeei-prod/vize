/**
 * Compiles an SFC once for the client and the SSR environments of one build.
 *
 * `vite build --app` builds its environments one after another, and its client
 * and SSR environments usually import the same components. Each environment
 * used to compile them in native batches of its own, so the second one parsed
 * every file again, selected its TypeScript project again, and read and
 * resolved its imported types again. Now, while a sibling environment that
 * compiles for the other target has not started yet, a batch compiles each
 * file for both targets in one native call (`compileSfcBatchForTargets`), and
 * the targets share the parse and one imported-type graph.
 *
 * The other target's result waits here until its environment asks for the
 * file. A request takes it only when its own compile options are the options
 * it was compiled with, in the same filesystem epoch. The result then enters
 * that environment's cache, which checks it as it checks any batch result: the
 * text the pipeline delivers, and the revision of the type project. Its
 * warnings are printed by that environment, once.
 *
 * Nothing that depends on the environment is compiled ahead: a file whose
 * `src` blocks name modules the bundler resolves, or whose options differ from
 * the sibling's in anything but the target (`ssr`, `vapor`, `inlineTemplate`).
 * A result with errors is not kept, so the file compiles again in its own
 * environment and raises its error there.
 *
 * A config file is evaluated once per environment, so the environments of one
 * build are often separate plugin instances. What they share lives in this
 * module and is keyed by the build's inline config, which every environment of
 * a builder is resolved from.
 */

import { parseSfc } from "@vizejs/native";

import { listSfcSrcRequests } from "../../../shared/sfc-src.ts";
import { buildCompileBatchOptions, type CompileBatchOptions } from "../compile-options.ts";
import type { BatchCompileOptionsNapi, CompiledModule } from "../types.ts";
import { normalizeCachePath } from "./cache-path.ts";
import type { FilesystemEpoch } from "./filesystem-epoch.ts";

/** The part of a Vite environment this module reads. */
export interface BuildEnvironment {
  name?: string;
  config?: {
    command?: string;
    builder?: unknown;
    inlineConfig?: unknown;
    environments?: Record<string, { consumer?: string } | undefined>;
  };
}

/** The other target a batch also compiles, for a sibling environment of `build`. */
export interface SiblingTarget {
  ssr: boolean;
  build: object;
  epoch: FilesystemEpoch;
}

export interface SiblingResult {
  compiled: CompiledModule;
  warnings: readonly string[];
}

interface Kept extends SiblingResult {
  build: object;
  options: string;
}

/** The environments of each build that started, and that closed. */
const builds = new WeakMap<object, { started: Set<string>; closed: Set<string> }>();
/** Sibling results by file, valid in one generation of one epoch. */
const kept = new WeakMap<FilesystemEpoch, { generation: number; files: Map<string, Kept[]> }>();

function buildOf(environment: BuildEnvironment | undefined): object | undefined {
  const config = environment?.config;
  if (!config?.builder || config.command !== "build") return undefined;
  const build = config.inlineConfig;
  return typeof build === "object" && build !== null ? build : undefined;
}

function recordOf(build: object) {
  let record = builds.get(build);
  if (!record) {
    record = { started: new Set(), closed: new Set() };
    builds.set(build, record);
  }
  return record;
}

/** An environment of a build started; a batch no longer compiles ahead for it. */
export function noteEnvironmentStarted(environment: BuildEnvironment | undefined): void {
  const build = buildOf(environment);
  if (build && environment?.name !== undefined) recordOf(build).started.add(environment.name);
}

/** An environment of a build closed. Once all of them did, nothing kept for it is needed. */
export function noteEnvironmentClosed(
  environment: BuildEnvironment | undefined,
  epoch: FilesystemEpoch | undefined,
): void {
  const build = buildOf(environment);
  if (!build || environment?.name === undefined) return;
  const record = recordOf(build);
  record.closed.add(environment.name);
  const names = Object.keys(environment.config?.environments ?? {});
  if (!epoch || !names.every((name) => record.closed.has(name))) return;
  const files = kept.get(epoch)?.files;
  if (!files) return;
  for (const [file, entries] of files) {
    const rest = entries.filter((entry) => entry.build !== build);
    if (rest.length > 0) files.set(file, rest);
    else files.delete(file);
  }
}

/**
 * The target a batch of `environment` also compiles: the other one, when an
 * environment of the same build compiles for it and has not started yet.
 */
export function siblingTargetFor(
  environment: BuildEnvironment | undefined,
  ssr: boolean,
  epoch: FilesystemEpoch | undefined,
): SiblingTarget | undefined {
  const build = buildOf(environment);
  if (!build || !epoch) return undefined;
  const started = builds.get(build)?.started;
  for (const [name, options] of Object.entries(environment?.config?.environments ?? {})) {
    if (name === environment?.name || started?.has(name)) continue;
    if ((options?.consumer === "server") !== ssr) return { ssr: !ssr, build, epoch };
  }
  return undefined;
}

/**
 * The compile options a sibling result is identified by: the native options,
 * and the root its scope identity is derived from.
 */
export function siblingOptionsKey(options: CompileBatchOptions): string {
  return JSON.stringify([options.root ?? null, buildCompileBatchOptions(options)]);
}

/** Options that set the target, which a multi-target call sets per target. */
const TARGET_OPTIONS = new Set<keyof BatchCompileOptionsNapi>(["ssr", "vapor", "inlineTemplate"]);

/** Whether `own` and `sibling` differ only in the options of the target. */
export function sameOutsideTarget(
  own: BatchCompileOptionsNapi,
  sibling: BatchCompileOptionsNapi,
): boolean {
  const strip = (options: BatchCompileOptionsNapi) =>
    JSON.stringify(
      Object.fromEntries(
        Object.entries(options).filter(
          ([key]) => !TARGET_OPTIONS.has(key as keyof BatchCompileOptionsNapi),
        ),
      ),
    );
  return strip(own) === strip(sibling);
}

/** Whether the file's compile input is the same in every environment. */
export function compilesAlikeEverywhere(file: string, source: string): boolean {
  return listSfcSrcRequests(parseSfc, file, source).length === 0;
}

function filesOf(epoch: FilesystemEpoch): Map<string, Kept[]> {
  let store = kept.get(epoch);
  if (!store || store.generation !== epoch.generation) {
    store = { generation: epoch.generation, files: new Map() };
    kept.set(epoch, store);
  }
  return store.files;
}

/** Keep the result a batch compiled for `target`, until its environment asks. */
export function keepSiblingResult(
  target: SiblingTarget,
  file: string,
  options: string,
  result: SiblingResult,
): void {
  const files = filesOf(target.epoch);
  const key = normalizeCachePath(file);
  const entries = (files.get(key) ?? []).filter((entry) => entry.options !== options);
  entries.push({ ...result, build: target.build, options });
  files.set(key, entries);
}

/** Whether anything is kept, so a request without sibling results costs one lookup. */
export function hasSiblingResults(epoch: FilesystemEpoch | undefined): boolean {
  if (!epoch) return false;
  const store = kept.get(epoch);
  return !!store && store.generation === epoch.generation && store.files.size > 0;
}

/** Take the result kept for `file` and compiled with `options`, if any. */
export function takeSiblingResult(
  epoch: FilesystemEpoch,
  file: string,
  options: string,
): SiblingResult | undefined {
  const files = filesOf(epoch);
  const key = normalizeCachePath(file);
  const entries = files.get(key);
  const index = entries?.findIndex((entry) => entry.options === options) ?? -1;
  if (!entries || index < 0) return undefined;
  const [entry] = entries.splice(index, 1);
  if (entries.length === 0) files.delete(key);
  return { compiled: entry.compiled, warnings: entry.warnings };
}

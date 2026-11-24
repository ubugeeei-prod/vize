/**
 * Compiles SFCs when the module graph asks for them, in native batches.
 *
 * A bundler resolves the imports of a module together and loads them
 * afterwards, so `resolveId` has seen a component's siblings by the time the
 * component reaches `transform`. Each compiled-module cache keeps the `.vue`
 * files its environment resolved or loaded and has not compiled. A request
 * compiles its own file together with those in one native batch, and the
 * siblings' requests find their result in the cache. An environment that
 * resolves no `.vue` file compiles nothing, and a file nothing imports is never
 * read.
 *
 * Only the requested file is compiled from the text the pipeline delivered. The
 * others are compiled from disk. `transformSfcModule` reuses a cached module
 * only for the text it was compiled from, so a file whose text a host transform
 * rewrites is compiled again by its own request.
 *
 * What a batch learns about another file is delivered by that file's own
 * request. A compiler error is not kept: the file compiles again when it is
 * requested, and the error is raised on its module. Warnings of a file that
 * compiled are kept until its request reuses the result.
 */

import fs from "node:fs";
import { parseSfc } from "@vizejs/native";

import { listSfcSrcRequests, type HostResolve } from "../../../shared/sfc-src.ts";
import { compileFile, VizeSfcCompileError } from "../compiler.ts";
import type { BatchFileInput, CompiledModule } from "../types.ts";
import { normalizeCachePath } from "./cache-path.ts";
import { CompiledModuleCache } from "./compiled-module-cache.ts";
import { DEFAULT_PRECOMPILE_BATCH_MAX_BYTES, DEFAULT_PRECOMPILE_BATCH_SIZE } from "./precompile.ts";
import type { PrecompileInputOwner } from "./precompile-owner.ts";
import { compileScopedBatch } from "./precompile-scopes.ts";
import { prepareSfcSrcRequests } from "./src-requests.ts";
import { getCompileOptionsForRequest, getEnvironmentCache, type VizePluginState } from "./state.ts";

interface DemandQueue {
  /** Resolved or loaded `.vue` files not compiled yet, in the order they were seen. */
  resolved: Set<string>;
  /** The batch compiling each file, so a request for one awaits it. */
  inFlight: Map<string, Promise<void>>;
  /** Warnings of files another file's batch compiled, with the text compiled. */
  warnings: Map<string, { source: string; warnings: readonly string[] }>;
}

// Keyed by the cache a queue fills. State copies share the cache objects.
const queues = new WeakMap<object, DemandQueue>();
const inputOwners = new WeakMap<object, readonly PrecompileInputOwner[]>();

function queueOf(cache: object): DemandQueue {
  let queue = queues.get(cache);
  if (!queue) {
    queue = { resolved: new Set(), inFlight: new Map(), warnings: new Map() };
    queues.set(cache, queue);
  }
  return queue;
}

/** Record that the module graph behind `cache` resolved or loaded the SFC `file`. */
export function requestSfcCompile(cache: Map<string, CompiledModule>, file: string): void {
  if (cache.has(file)) return;
  const queue = queueOf(cache);
  file = normalizeCachePath(file);
  if (!queue.inFlight.has(file)) queue.resolved.add(file);
}

/** Companion plugins whose raw inputs must never be compiled from disk. */
export function setCompileInputOwners(
  state: Pick<VizePluginState, "cache">,
  owners: readonly PrecompileInputOwner[],
): void {
  inputOwners.set(state.cache, owners);
}

/** Forget what a finished build resolved. Its caches are cleared with it. */
export function resetDemandCompiles(state: Pick<VizePluginState, "cache" | "ssrCache">): void {
  queues.delete(state.cache);
  queues.delete(state.ssrCache);
}

export interface DemandCompileOptions {
  /** The bundler's resolver, for `src` module requests. */
  resolve?: HostResolve;
  /** Whether the requested file's compiler warnings are printed. @default true */
  logWarnings?: boolean;
}

/**
 * Compile `file` from `source`, with the files its environment resolved and
 * has not compiled. Concurrent requests for a file of a running batch await
 * that batch.
 */
export async function compileRequestedSfc(
  state: VizePluginState,
  file: string,
  source: string,
  ssr: boolean,
  options: DemandCompileOptions = {},
): Promise<CompiledModule> {
  const cache = getEnvironmentCache(state, ssr);
  const queue = queueOf(cache);
  const key = normalizeCachePath(file);
  for (let running = queue.inFlight.get(key); running; running = queue.inFlight.get(key)) {
    await running;
    const compiled = cache.get(file);
    if (compiled?.source === source) {
      reportBatchedWarnings(cache, file, source, options.logWarnings);
      return compiled;
    }
  }

  let files: BatchFileInput[] = [
    { path: file, source },
    ...takeResolvedFiles(state, queue, cache, key, source),
  ];
  let finish!: () => void;
  const batch = new Promise<void>((resolve) => {
    finish = resolve;
  });
  const keys = files.map((entry) => normalizeCachePath(entry.path));
  for (const member of keys) queue.inFlight.set(member, batch);
  try {
    const resolve = options.resolve;
    if (resolve) {
      const prepared: BatchFileInput[] = [];
      for (const entry of files) {
        try {
          await prepareSfcSrcRequests(state, resolve, entry.path, entry.source);
          prepared.push(entry);
        } catch (error) {
          // Another file's unresolved `src` request is reported by its own request.
          if (entry.path === file) throw error;
        }
      }
      files = prepared;
    }
    return compileFiles(state, queue, cache, ssr, files, options.logWarnings);
  } finally {
    for (const member of keys) {
      if (queue.inFlight.get(member) === batch) queue.inFlight.delete(member);
    }
    finish();
  }
}

/**
 * `compileRequestedSfc` for a hook that cannot await. It leaves a file whose
 * `src` module requests need the bundler's resolver to a later request.
 */
export function compileRequestedSfcNow(
  state: VizePluginState,
  file: string,
  source: string,
  ssr: boolean,
): CompiledModule {
  const cache = getEnvironmentCache(state, ssr);
  const queue = queueOf(cache);
  const members = takeResolvedFiles(
    state,
    queue,
    cache,
    normalizeCachePath(file),
    source,
    (member) => listSfcSrcRequests(parseSfc, member.path, member.source).length === 0,
  );
  return compileFiles(state, queue, cache, ssr, [{ path: file, source }, ...members]);
}

/** Print the warnings a batch collected for `file`, once its request reuses the result. */
export function reportBatchedWarnings(
  cache: Map<string, CompiledModule>,
  file: string,
  source: string,
  log = true,
): void {
  const queue = queues.get(cache);
  if (!queue || queue.warnings.size === 0) return;
  const key = normalizeCachePath(file);
  const batched = queue.warnings.get(key);
  if (!batched) return;
  queue.warnings.delete(key);
  if (log && batched.source === source) printWarnings(file, batched.warnings);
}

function printWarnings(file: string, warnings: readonly string[]): void {
  for (const warning of warnings) console.warn(`[vize] Warning in ${file}: ${warning}`);
}

function* candidates(queue: DemandQueue, cache: object): Iterable<string> {
  yield* queue.resolved;
  // A hot update evicts the owners of a changed dependency. They are requested again.
  if (cache instanceof CompiledModuleCache) yield* cache.evictedFiles();
}

/**
 * The files to compile with a request, within the limits of one native batch
 * (`precompileBatchSize` files, and the byte budget the startup scan uses).
 */
function takeResolvedFiles(
  state: VizePluginState,
  queue: DemandQueue,
  cache: Map<string, CompiledModule>,
  requested: string,
  requestedSource: string,
  accepts: (file: BatchFileInput) => boolean = () => true,
): BatchFileInput[] {
  const limit =
    state.precompileBatchSize > 0 ? state.precompileBatchSize : DEFAULT_PRECOMPILE_BATCH_SIZE;
  const owners = inputOwners.get(state.cache) ?? [];
  const files: BatchFileInput[] = [];
  // An evicted file that was resolved again is a candidate twice.
  const seen = new Set([requested]);
  let bytes = Buffer.byteLength(requestedSource);
  queue.resolved.delete(requested);
  for (const file of candidates(queue, cache)) {
    if (files.length + 1 >= limit) break;
    if (seen.has(file) || queue.inFlight.has(file)) continue;
    seen.add(file);
    let source: string;
    try {
      source = fs.readFileSync(file, "utf-8");
    } catch (error) {
      queue.resolved.delete(file);
      // A removed file is never requested again.
      if ((error as NodeJS.ErrnoException).code === "ENOENT") cache.delete(file);
      continue;
    }
    if (cache.get(file)?.source === source) {
      queue.resolved.delete(file);
      continue;
    }
    const entry = { path: file, source };
    if (!accepts(entry)) continue;
    const size = Buffer.byteLength(source);
    if (bytes + size > DEFAULT_PRECOMPILE_BATCH_MAX_BYTES) break;
    queue.resolved.delete(file);
    // A companion turns its raw input into another SFC text before the compile.
    if (ownsInput(owners, file, source)) continue;
    files.push(entry);
    bytes += size;
  }
  return files;
}

function ownsInput(owners: readonly PrecompileInputOwner[], file: string, source: string): boolean {
  try {
    return owners.some((owns) => owns(file, source));
  } catch {
    // The companion reports its failure when the file itself is requested.
    return true;
  }
}

/** Compile `files`, the first being the requested one, and return its module. */
function compileFiles(
  state: VizePluginState,
  queue: DemandQueue,
  cache: Map<string, CompiledModule>,
  ssr: boolean,
  files: BatchFileInput[],
  logWarnings = true,
): CompiledModule {
  const { path: file, source } = files[0];
  const alone = (): CompiledModule =>
    compileFile(file, cache, getCompileOptionsForRequest(state, ssr, file), source, {
      logWarnings,
    });
  if (files.length === 1) return alone();

  let failed = false;
  let failure: unknown;
  let results: ReturnType<typeof compileScopedBatch>["results"];
  try {
    ({ results } = compileScopedBatch(files, state, ssr, {
      log: false,
      onFileFailure(path, error) {
        if (path !== file) return;
        failed = true;
        failure = error;
      },
    }));
  } catch (error) {
    // A failure of the batch as a whole says nothing about the requested file.
    state.logger.log(`transform: batch with ${file} failed, compiling it alone`, error);
    return alone();
  }
  if (failed) throw failure;

  const sources = new Map(files.map((entry) => [entry.path, entry.source]));
  let own: (typeof results)[number] | undefined;
  for (const result of results) {
    if (result.path === file) {
      own = result;
    } else if (result.errors.length === 0 && result.warnings.length > 0) {
      queue.warnings.set(normalizeCachePath(result.path), {
        source: sources.get(result.path) ?? "",
        warnings: result.warnings,
      });
    }
  }
  if (!own) return alone();
  if (own.errors.length > 0) throw new VizeSfcCompileError(file, own.errors);
  if (logWarnings) printWarnings(file, own.warnings);
  return cache.get(file) ?? alone();
}

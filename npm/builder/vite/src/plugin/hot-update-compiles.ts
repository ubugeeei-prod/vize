/**
 * Compiles the SFCs a wave of hot updates changed, in native batches.
 *
 * When many files change at once (a branch switch, a formatter, a code
 * generator), the watcher reports each file on its own and Vite runs the client
 * `hotUpdate` hook once per file, concurrently. Each hook needs its file's new
 * output to classify the update. Compiling it there one file at a time cost one
 * native call per file, and every call paid again what a batch shares, such as
 * resolving the types its imports name. The calls also ran one after another
 * on the event loop, so the last hook of a wave waited for all of them.
 *
 * A hook instead queues its file and awaits the wave. The wave compiles when
 * the event loop has run the callbacks that were already due, so the hooks
 * whose reads finished by then join it. While it compiles, the next reads
 * finish and form the next wave. Each wave compiles in native batches of at
 * most `precompileBatchSize` files and the byte budget the startup scan uses.
 * A wave of one file compiles alone, as a single edit always did.
 *
 * Every hook gets its own file's result. A compiler error, or a failure to
 * inline the file's `src` blocks, is raised to that file's hook alone. The
 * other files of its batch still compile, and their warnings are printed.
 */

import { compileFile, VizeSfcCompileError } from "../compiler.ts";
import type { CompiledModule } from "../types.ts";
import { DEFAULT_PRECOMPILE_BATCH_MAX_BYTES, DEFAULT_PRECOMPILE_BATCH_SIZE } from "./precompile.ts";
import { compileScopedBatch } from "./precompile-scopes.ts";
import { getCompileOptionsForRequest, type VizePluginState } from "./state.ts";

interface QueuedFile {
  source: string;
  waiters: { resolve: (compiled: HotUpdateCompile) => void; reject: (error: unknown) => void }[];
}

export interface HotUpdateCompile {
  compiled: CompiledModule;
  /**
   * Whether an earlier hook of the wave queued the same file. Compiled one
   * after the other, that hook's compile would have been this one's baseline.
   */
  repeated: boolean;
}

// Keyed by the client cache the waves fill. State copies share the cache.
const waves = new WeakMap<object, Map<string, QueuedFile>>();

/**
 * Compile the client output of `file` from `source` with the other SFCs of
 * its hot-update wave, into `state.cache`. A file queued twice in one wave is
 * compiled once, from the text queued last, and the later hooks learn it is
 * `repeated`.
 */
export function compileHotUpdatedSfc(
  state: VizePluginState,
  file: string,
  source: string,
): Promise<HotUpdateCompile> {
  const cache = state.cache;
  let wave = waves.get(cache);
  if (!wave) {
    const queued = new Map<string, QueuedFile>();
    waves.set(cache, queued);
    wave = queued;
    setImmediate(() => {
      waves.delete(cache);
      compileWave(state, queued);
    });
  }
  const queued = wave.get(file);
  return new Promise((resolve, reject) => {
    if (queued) {
      queued.source = source;
      queued.waiters.push({ resolve, reject });
    } else {
      wave.set(file, { source, waiters: [{ resolve, reject }] });
    }
  });
}

function compileWave(state: VizePluginState, wave: Map<string, QueuedFile>): void {
  const limit =
    state.precompileBatchSize > 0 ? state.precompileBatchSize : DEFAULT_PRECOMPILE_BATCH_SIZE;
  let batch: [string, QueuedFile][] = [];
  let bytes = 0;
  for (const entry of wave) {
    const size = Buffer.byteLength(entry[1].source);
    if (
      batch.length > 0 &&
      (batch.length >= limit || bytes + size > DEFAULT_PRECOMPILE_BATCH_MAX_BYTES)
    ) {
      compileBatchOf(state, batch);
      batch = [];
      bytes = 0;
    }
    batch.push(entry);
    bytes += size;
  }
  if (batch.length > 0) compileBatchOf(state, batch);
}

function settle(queued: QueuedFile, run: () => CompiledModule): void {
  let compiled: CompiledModule;
  try {
    compiled = run();
  } catch (error) {
    for (const waiter of queued.waiters) waiter.reject(error);
    return;
  }
  queued.waiters.forEach((waiter, index) => waiter.resolve({ compiled, repeated: index > 0 }));
}

function compileAlone(state: VizePluginState, file: string, source: string): CompiledModule {
  return compileFile(file, state.cache, getCompileOptionsForRequest(state, false, file), source);
}

function compileBatchOf(state: VizePluginState, batch: [string, QueuedFile][]): void {
  if (batch.length === 1) {
    const [file, queued] = batch[0];
    settle(queued, () => compileAlone(state, file, queued.source));
    return;
  }

  const failures = new Map<string, unknown>();
  let results: ReturnType<typeof compileScopedBatch>["results"];
  try {
    ({ results } = compileScopedBatch(
      batch.map(([path, queued]) => ({ path, source: queued.source })),
      state,
      false,
      {
        log: false,
        onFileFailure(path, error) {
          failures.set(path, error);
        },
      },
    ));
  } catch (error) {
    // A failure of the batch as a whole says nothing about any one file.
    state.logger.log(
      `hot update: batch of ${batch.length} SFCs failed, compiling each alone`,
      error,
    );
    for (const [file, queued] of batch)
      settle(queued, () => compileAlone(state, file, queued.source));
    return;
  }

  const byPath = new Map(results.map((result) => [result.path, result]));
  for (const [file, queued] of batch) {
    settle(queued, () => {
      if (failures.has(file)) throw failures.get(file);
      const result = byPath.get(file);
      if (!result) return compileAlone(state, file, queued.source);
      if (result.errors.length > 0) throw new VizeSfcCompileError(file, result.errors);
      for (const warning of result.warnings) console.warn(`[vize] Warning in ${file}: ${warning}`);
      const compiled = state.cache.get(file);
      return compiled?.source === queued.source
        ? compiled
        : compileAlone(state, file, queued.source);
    });
  }
}

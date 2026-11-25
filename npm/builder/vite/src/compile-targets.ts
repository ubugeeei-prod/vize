/**
 * One native call that compiles a batch for its own target and for a sibling
 * environment's (see `plugin/sibling-compiles.ts`).
 */

import * as native from "@vizejs/native";

import { buildCompileBatchOptions, type CompileBatchOptions } from "./compile-options.ts";
import {
  batchCompiledModule,
  finishBatch,
  prepareBatch,
  type CompileBatchReporting,
  type NativeBatchResult,
} from "./compiler.ts";
import type { BatchCompileOptionsNapi, BatchFileInput, CompiledModule } from "./types.ts";

/** What one target of a multi-target call sets; the rest is the call's options. */
interface BatchCompileTarget {
  ssr: boolean;
  vapor: boolean;
  inlineTemplate: boolean;
}

type CompileSfcBatchForTargetsFn = (
  files: BatchFileInput[],
  options: BatchCompileOptionsNapi,
  targets: BatchCompileTarget[],
) => { targets: NativeBatchResult[]; dependenciesShared: boolean[]; timeMs: number };

const { compileSfcBatchForTargets } = native as {
  compileSfcBatchForTargets?: CompileSfcBatchForTargetsFn;
};

/** Whether the native binding compiles a batch for several targets in one call. */
export function canCompileForTargets(): boolean {
  return typeof compileSfcBatchForTargets === "function";
}

function targetOf(options: BatchCompileOptionsNapi): BatchCompileTarget {
  return {
    ssr: options.ssr ?? false,
    vapor: options.vapor ?? false,
    inlineTemplate: options.inlineTemplate ?? false,
  };
}

export interface TargetsBatchResult {
  /** The batch's own results, cached and reported as `compileBatch` does. */
  own: NativeBatchResult;
  /** The sibling target's modules, for the files that compiled for both targets. */
  sibling: Map<string, { compiled: CompiledModule; warnings: readonly string[] }>;
}

/**
 * `compileBatch` for `options`, which also compiles each file with `sibling`,
 * the same options for the other target, in the same native call.
 */
export function compileBatchForTargets(
  files: { path: string; source: string }[],
  cache: Map<string, CompiledModule>,
  options: CompileBatchOptions,
  sibling: CompileBatchOptions,
  reporting: CompileBatchReporting = {},
): TargetsBatchResult {
  if (!compileSfcBatchForTargets) {
    throw new Error("[vize] this native binding cannot compile a batch for several targets");
  }
  const batch = prepareBatch(files, options, reporting);
  const ownOptions = buildCompileBatchOptions(options);
  const {
    targets: [own, other],
    dependenciesShared,
  } = compileSfcBatchForTargets(batch.files, ownOptions, [
    targetOf(ownOptions),
    targetOf(buildCompileBatchOptions(sibling)),
  ]);
  // The sibling target leaves out the dependencies it shares with this one.
  dependenciesShared.forEach((shared, index) => {
    const siblingResult = other.results[index];
    if (shared && siblingResult) siblingResult.dependencies = own.results[index]?.dependencies;
  });
  finishBatch(own, batch, cache, reporting.log ?? true);
  const ownErrors = new Set(
    own.results.filter((result) => result.errors.length > 0).map((result) => result.path),
  );
  const siblingResults: TargetsBatchResult["sibling"] = new Map();
  for (const fileResult of other.results) {
    // A file is handed on only once its own result passed every check.
    if (fileResult.errors.length > 0 || ownErrors.has(fileResult.path)) continue;
    if (batch.failed.has(fileResult.path)) continue;
    const compiled = batchCompiledModule(fileResult, batch);
    const ownModule = cache.get(fileResult.path);
    siblingResults.set(fileResult.path, {
      compiled: ownModule ? sharingEqualParts(compiled, ownModule) : compiled,
      warnings: fileResult.warnings,
    });
  }
  return { own, sibling: siblingResults };
}

function sameStrings(left: readonly string[], right: readonly string[]): boolean {
  if (left.length !== right.length) return false;
  for (let index = 0; index < left.length; index++) {
    if (left[index] !== right[index]) return false;
  }
  return true;
}

/**
 * `sibling`, holding `own`'s values for the parts both targets compiled alike.
 *
 * The parts that do not depend on the target are equal in both results, but
 * arrive as separate copies. A component typed through a large barrel lists
 * every module of it as a dependency, so keeping one copy is what keeps the
 * sibling's result from doubling the memory a build holds.
 */
function sharingEqualParts(sibling: CompiledModule, own: CompiledModule): CompiledModule {
  const shared = { ...sibling };
  if (own.dependencies && sibling.dependencies) {
    if (sameStrings(own.dependencies, sibling.dependencies)) shared.dependencies = own.dependencies;
  }
  if (
    own.typeProject &&
    sibling.typeProject &&
    own.typeProject.root === sibling.typeProject.root &&
    own.typeProject.revision === sibling.typeProject.revision
  ) {
    shared.typeProject = own.typeProject;
  }
  for (const key of ["styles", "customBlocks", "macroArtifacts"] as const) {
    if (own[key] !== undefined && JSON.stringify(own[key]) === JSON.stringify(sibling[key])) {
      Object.assign(shared, { [key]: own[key] });
    }
  }
  return shared;
}

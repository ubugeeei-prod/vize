import { compileBatch, type CompileBatchReporting, type NativeBatchResult } from "../compiler.ts";
import { canCompileForTargets, compileBatchForTargets } from "../compile-targets.ts";
import { buildCompileBatchOptions } from "../compile-options.ts";
import { getCompileOptionsForRequest, getEnvironmentCache, type VizePluginState } from "./state.ts";
import {
  compilesAlikeEverywhere,
  keepSiblingResult,
  sameOutsideTarget,
  siblingOptionsKey,
  type SiblingTarget,
} from "./sibling-compiles.ts";
import type { BatchFileInput } from "../types.ts";

type CompileOptions = ReturnType<typeof getCompileOptionsForRequest>;

/**
 * Compile `files` for their environment, one native batch per set of compile
 * options. With a `sibling` target, the files whose compile only the target
 * tells apart also compile for that target in the same call, and the results
 * are kept for the sibling environment (`sibling-compiles.ts`).
 */
export function compileScopedBatch(
  files: BatchFileInput[],
  state: VizePluginState,
  ssr = false,
  reporting?: CompileBatchReporting,
  sibling?: SiblingTarget,
) {
  const shared = sibling !== undefined && canCompileForTargets();
  const groups = new Map<
    string,
    { options: CompileOptions; sibling?: CompileOptions; files: BatchFileInput[] }
  >();
  for (const file of files) {
    const options = getCompileOptionsForRequest(state, ssr, file.path);
    const native = buildCompileBatchOptions(options);
    let key = JSON.stringify(native);
    let siblingOptions: CompileOptions | undefined;
    if (shared && compilesAlikeEverywhere(file.path, file.source)) {
      const candidate = getCompileOptionsForRequest(state, sibling.ssr, file.path);
      const candidateNative = buildCompileBatchOptions(candidate);
      if (sameOutsideTarget(native, candidateNative)) {
        siblingOptions = candidate;
        key += `\0${JSON.stringify(candidateNative)}`;
      }
    }
    const group = groups.get(key);
    if (group) group.files.push(file);
    else groups.set(key, { options, sibling: siblingOptions, files: [file] });
  }
  const results: Array<
    ReturnType<typeof compileBatch>["results"][number] | NativeBatchResult["results"][number]
  > = [];
  let timeMs = 0;
  for (const group of groups.values()) {
    const cache = getEnvironmentCache(state, ssr);
    if (sibling && group.sibling) {
      const compiled = compileBatchForTargets(
        group.files,
        cache,
        group.options,
        group.sibling,
        reporting,
      );
      const options = siblingOptionsKey(group.sibling);
      for (const [path, result] of compiled.sibling) {
        keepSiblingResult(sibling, path, options, result);
      }
      results.push(...compiled.own.results);
      timeMs += compiled.own.timeMs;
      continue;
    }
    const result = compileBatch(group.files, cache, group.options, reporting);
    results.push(...result.results);
    timeMs += result.timeMs;
  }
  return { results, timeMs };
}

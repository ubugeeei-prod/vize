import { compileBatch } from "../compiler.ts";
import { buildCompileBatchOptions } from "../compile-options.ts";
import { getCompileOptionsForRequest, type VizePluginState } from "./state.ts";
import type { BatchFileInput } from "../types.ts";

export function compileScopedBatch(files: BatchFileInput[], state: VizePluginState) {
  const groups = new Map<
    string,
    { options: ReturnType<typeof getCompileOptionsForRequest>; files: BatchFileInput[] }
  >();
  for (const file of files) {
    const options = getCompileOptionsForRequest(state, false, file.path);
    const key = JSON.stringify(buildCompileBatchOptions(options));
    const group = groups.get(key);
    if (group) group.files.push(file);
    else groups.set(key, { options, files: [file] });
  }
  const results: ReturnType<typeof compileBatch>["results"] = [];
  let timeMs = 0;
  for (const group of groups.values()) {
    const result = compileBatch(group.files, state.cache, group.options);
    results.push(...result.results);
    timeMs += result.timeMs;
  }
  return { results, timeMs };
}

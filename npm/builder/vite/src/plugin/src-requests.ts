import fs from "node:fs";
import { classifyVitePluginRequest, parseSfc } from "@vizejs/native";
import { listSfcSrcRequests, resolveSfcSrcRequests } from "../../../shared/sfc-src.ts";
import { fromPluginVisibleVirtualId } from "../virtual.ts";
import { getLoadableVueSfcPath } from "./load-sfc.ts";
import type { VizePluginState } from "./state.ts";

export type HostResolve = (request: string, importer: string) => Promise<string | null | undefined>;

type ResolvedIdLike = { id: string; external?: boolean | string } | null | undefined;

/** Vite's own resolver (aliases, packages, every `resolveId` plugin). */
export function viteHostResolve(
  resolve: (request: string, importer: string) => Promise<ResolvedIdLike>,
): HostResolve {
  return async (request, importer) => {
    const result = await resolve(request, importer);
    // An external module is not bundled, so there is no file to inline.
    return result && !result.external ? result.id : null;
  };
}

/**
 * Resolve one SFC's `src` module requests before the synchronous compile reads
 * them from `state.srcResolutions`. A request left unresolved fails that compile.
 */
export async function prepareSfcSrcRequests(
  state: Pick<VizePluginState, "srcResolutions">,
  resolve: HostResolve,
  file: string,
  source: string,
): Promise<void> {
  const requests = listSfcSrcRequests(parseSfc, file, source);
  if (requests.length === 0) {
    state.srcResolutions?.delete(file);
    return;
  }
  (state.srcResolutions ??= new Map()).set(
    file,
    await resolveSfcSrcRequests(file, requests, resolve),
  );
}

/** An SFC's source as `load` read it for the `src` request scan. */
export type LoadedSfcSource = { file: string; source: string };

/**
 * A `load` that compiles its SFC on demand resolves the SFC's requests first.
 * It returns the source it read, so that compile does not read the file again.
 */
export async function prepareLoadSrcRequests(
  state: VizePluginState,
  resolve: HostResolve,
  id: string,
): Promise<LoadedSfcSource | undefined> {
  if (!id.includes(".vue")) return undefined;
  const request = classifyVitePluginRequest(id);
  const file =
    request.vizeVirtualPath ?? fromPluginVisibleVirtualId(id) ?? getLoadableVueSfcPath(request);
  if (
    !file?.endsWith(".vue") ||
    state.cache.has(file) ||
    state.ssrCache.has(file) ||
    state.srcResolutions?.has(file) ||
    !fs.existsSync(file)
  ) {
    return undefined;
  }
  const source = fs.readFileSync(file, "utf-8");
  await prepareSfcSrcRequests(state, resolve, file, source);
  return { file, source };
}

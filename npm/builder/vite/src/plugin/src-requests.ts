import fs from "node:fs";
import { classifyVitePluginRequest } from "@vizejs/native";
import { resolveSfcSrcRequests } from "../../../shared/sfc-src.ts";
import { listSfcSrcRequests } from "../compiler.ts";
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
  const requests = listSfcSrcRequests(file, source);
  if (requests.length === 0) {
    state.srcResolutions?.delete(file);
    return;
  }
  (state.srcResolutions ??= new Map()).set(
    file,
    await resolveSfcSrcRequests(file, requests, resolve),
  );
}

/** A `load` that compiles its SFC on demand resolves the SFC's requests first. */
export async function prepareLoadSrcRequests(
  state: VizePluginState,
  resolve: HostResolve,
  id: string,
): Promise<void> {
  if (!id.includes(".vue")) return;
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
    return;
  }
  await prepareSfcSrcRequests(state, resolve, file, fs.readFileSync(file, "utf-8"));
}

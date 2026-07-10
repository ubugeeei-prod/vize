import { classifyVitePluginRequest, splitViteIdQuery } from "@vizejs/native";

import type { VizePluginState } from "./state.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";

type ViteResolveResult = { id: string; external?: boolean } | null;

interface ResolveContext {
  environment?: { name?: string };
  resolve(
    id: string,
    importer?: string,
    options?: { skipSelf: boolean },
  ): Promise<ViteResolveResult>;
}

type ResolveWithVite = (
  ctx: ResolveContext,
  state: VizePluginState,
  id: string,
  importer?: string,
  options?: { skipSelf: boolean },
) => Promise<ViteResolveResult>;

export function cleanVueSfcImporter(
  importer: string,
  request: ReturnType<typeof classifyVitePluginRequest> | null,
): string {
  let cleanImporter = request?.normalizedFsId ?? request?.normalizedVuePath ?? importer;

  if (cleanImporter.startsWith("/@id/__x00__")) {
    cleanImporter = cleanImporter.slice("/@id/__x00__".length);
  } else if (cleanImporter.startsWith("__x00__")) {
    cleanImporter = cleanImporter.slice("__x00__".length);
  }

  return classifyVitePluginRequest(cleanImporter).normalizedVuePath;
}

export async function resolveRelativeVueSfcImport(
  ctx: ResolveContext,
  state: VizePluginState,
  id: string,
  cleanImporter: string,
  isSsrRequest: boolean,
  isDependencyScan: boolean,
  resolveWithVite: ResolveWithVite,
): Promise<string | { id: string; external?: boolean } | null> {
  if (!id.startsWith("./") && !id.startsWith("../")) {
    return null;
  }

  const { request, querySuffix } = splitViteIdQuery(id);
  if (!request.endsWith(".vue")) {
    return null;
  }

  const resolved = await resolveWithVite(ctx, state, id, cleanImporter, { skipSelf: true });
  if (!resolved) {
    return null;
  }

  const resolvedRequest = classifyVitePluginRequest(resolved.id);
  const resolvedPath = resolvedRequest.normalizedFsId ?? resolvedRequest.normalizedVuePath;
  if (!resolvedPath || !resolvedPath.endsWith(".vue")) {
    return resolved;
  }

  const effectiveQuery = resolvedRequest.querySuffix || querySuffix;
  return isDependencyScan
    ? toVirtualId(resolvedPath, isSsrRequest)
    : toPluginVisibleVirtualId(resolvedPath, isSsrRequest, effectiveQuery);
}

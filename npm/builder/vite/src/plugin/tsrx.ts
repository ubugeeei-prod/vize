import * as native from "@vizejs/native";
import { classifyVitePluginRequest } from "@vizejs/native";
import type { TransformResult } from "vite";

import {
  compileTsrxModule,
  isTsrxFile,
  TSRX_INCLUDE_PATTERN,
  type CompileTsrxFn,
} from "../../../shared/tsrx.ts";
import { createFilter } from "../utils/index.ts";
import type { VizePluginState } from "./state.ts";

const compileTsrx = (native as unknown as { compileTsrx: CompileTsrxFn }).compileTsrx;

/** Asset and worker imports of a module keep Vite's own handling. */
function isAssetRequest(querySuffix: string | null | undefined): boolean {
  if (!querySuffix) return false;
  const params = new URLSearchParams(querySuffix.slice(1));
  return ["raw", "url", "worker", "sharedworker"].some((name) => params.has(name));
}

/**
 * Compile a `.tsrx` request through the native TSRX entry (#13140).
 *
 * No other Vite plugin reads `.tsrx`, so every `.tsrx` module compiles unless
 * the plugin's `exclude` matches it. The module is returned as JavaScript: as
 * for `.vue` modules, the native entry erases TypeScript with the `.vue`
 * whole-script erasure. A refused module throws with every diagnostic.
 *
 * Returns `undefined` for every other request.
 */
export function transformTsrxRequest(
  state: VizePluginState,
  code: string,
  id: string,
  options?: { ssr?: boolean },
): TransformResult | undefined {
  if (id.startsWith("\0") || !id.includes(".tsrx")) return undefined;
  const request = classifyVitePluginRequest(id);
  if (!isTsrxFile(request.path) || isAssetRequest(request.querySuffix)) return undefined;
  const realPath = request.normalizedFsId
    ? classifyVitePluginRequest(request.normalizedFsId).path
    : request.path;
  if (!createFilter(TSRX_INCLUDE_PATTERN, state.mergedOptions.exclude)(realPath)) {
    return undefined;
  }
  const fileOptions = state.fileCompilerOptions?.(realPath) ?? state.mergedOptions;
  const compiled = compileTsrxModule(compileTsrx, realPath, code, {
    ssr: options?.ssr ?? false,
    vapor: fileOptions.vapor ?? false,
    eraseTypes: true,
  });
  return { code: compiled.code, map: null };
}

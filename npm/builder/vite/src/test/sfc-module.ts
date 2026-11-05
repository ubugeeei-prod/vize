/**
 * Vite's two steps for an SFC main module, for suites that drive the hooks
 * directly: `load` serves the SFC text, then the compile stage compiles the
 * text the pipeline delivers.
 */

import { loadHook } from "../plugin/load.ts";
import {
  getSfcModuleRequest,
  transformSfcModule,
  type SfcModuleResult,
  type SfcModuleTransformOptions,
} from "../plugin/sfc-module.ts";
import { getEnvironmentCache, type VizePluginState } from "../plugin/state.ts";

/** The SFC text a hand-built fixture module stands for. */
export const FIXTURE_SFC_SOURCE = "<template><div /></template>\n";

/** `load` then compile, for an SFC that exists on disk. */
export async function loadSfcModule(
  state: VizePluginState,
  id: string,
  options: SfcModuleTransformOptions = {},
): Promise<SfcModuleResult | null> {
  const sfc = getSfcModuleRequest(state, id);
  const source = loadHook(state, id, { ssr: options.ssr });
  if (!sfc || source === null) return null;
  return transformSfcModule(state, typeof source === "string" ? source : source.code, sfc, options);
}

/**
 * The compile stage for a hand-built module cached under `id`'s file, as if
 * the pipeline delivered the text it was compiled from.
 */
export async function transformCachedSfcModule(
  state: VizePluginState,
  id: string,
  options: SfcModuleTransformOptions = {},
): Promise<SfcModuleResult> {
  const sfc = getSfcModuleRequest(state, id);
  if (!sfc) throw new Error(`${id} is not an SFC main module`);
  const cached = getEnvironmentCache(state, sfc.ssrId || !!options.ssr).get(sfc.realPath);
  if (!cached) throw new Error(`no fixture module is cached for ${sfc.realPath}`);
  cached.source ??= FIXTURE_SFC_SOURCE;
  return transformSfcModule(state, cached.source, sfc, options);
}

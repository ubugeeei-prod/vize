import fs from "node:fs";
import path from "node:path";
import * as vite from "vite";
import type { TransformResult } from "vite";

import type { VizePluginState } from "./state.ts";
import { applyDefineReplacements } from "../transform.ts";

type TransformOutput = { code: string; map?: unknown };
type OxcOptions = { lang: "ts"; sourcemap: false; target: "esnext" };
type EsbuildOptions = { loader: "ts"; sourcemap: false; target: "esnext" };
type TransformWithOxc = (
  code: string,
  id: string,
  options: OxcOptions,
) => TransformOutput | Promise<TransformOutput>;
type TransformWithEsbuild = (
  code: string,
  id: string,
  options: EsbuildOptions,
) => TransformOutput | Promise<TransformOutput>;

interface ViteTransformApi {
  transformWithOxc?: TransformWithOxc;
  transformWithEsbuild?: TransformWithEsbuild;
}

export function createVirtualTypeScriptTransformer(viteApi: ViteTransformApi) {
  return async (code: string, id: string): Promise<TransformOutput> => {
    // This pass only strips TypeScript from Vize virtual Vue modules.
    // Syntax lowering belongs to Vite's normal build target transform.
    if (typeof viteApi.transformWithOxc === "function") {
      return viteApi.transformWithOxc(code, id, {
        lang: "ts",
        sourcemap: false,
        target: "esnext",
      });
    }
    if (typeof viteApi.transformWithEsbuild === "function") {
      return viteApi.transformWithEsbuild(code, id, {
        loader: "ts",
        sourcemap: false,
        target: "esnext",
      });
    }
    throw new Error("Installed Vite does not expose transformWithOxc or transformWithEsbuild");
  };
}

/**
 * Report whether `realPath` names a module Vize's own compiler emitted.
 *
 * Vize's Rust emitter guarantees plain JavaScript for every module it produces
 * (`ensure_javascript_output` at the napi boundary), so re-running Vite's
 * TypeScript strip over emitter output is a pure re-print. Every emitted module
 * is recorded in one of the two environment caches, so cache membership is the
 * cheap, allocation-free proof that the code came from the emitter.
 *
 * The probe fails safe: a module the caches do not know about still gets the
 * strip, which keeps hand-written and malformed virtual modules behaving
 * exactly as they did before.
 */
function isVizeEmitterOutput(
  state: Pick<VizePluginState, "cache" | "ssrCache">,
  realPath: string,
): boolean {
  return state.cache.has(realPath) || state.ssrCache.has(realPath);
}

export const transformVirtualTypeScript = createVirtualTypeScriptTransformer(vite);

function getOxcDumpPath(root: string, realPath: string): string {
  const dumpDir = path.resolve(root || process.cwd(), "node_modules", ".vize", "oxc-dumps");
  fs.mkdirSync(dumpDir, { recursive: true });
  return path.join(dumpDir, `vize-oxc-error-${path.basename(realPath)}.ts`);
}

function getVirtualModuleDefines(
  state: Pick<VizePluginState, "clientViteDefine" | "isProduction" | "serverViteDefine">,
  ssr: boolean,
): Record<string, string> {
  return {
    "import.meta.client": ssr ? "false" : "true",
    "import.meta.server": ssr ? "true" : "false",
    "import.meta.dev": state.isProduction ? "false" : "true",
    "import.meta.test": "false",
    "import.meta.prerender": "false",
    ...(ssr ? state.serverViteDefine : state.clientViteDefine),
  };
}

function formatUnknownError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export async function transformVizeVirtualModule(
  state: VizePluginState,
  code: string,
  realPath: string,
  ssr: boolean,
  forceTypeScriptTransform = false,
): Promise<TransformResult | null> {
  // `?macro=true` artifacts are raw macro module code that never passed
  // through the emitter's JavaScript guarantee, so they always need the strip.
  const needsTsTransform = forceTypeScriptTransform || !isVizeEmitterOutput(state, realPath);
  try {
    const result = needsTsTransform ? await transformVirtualTypeScript(code, realPath) : { code };
    let transformed = result.code;
    if (transformed.includes("import.meta.")) {
      transformed = applyDefineReplacements(transformed, getVirtualModuleDefines(state, ssr));
    }
    // `map: null` is Rollup's "this transform did not move code, keep the map
    // the load hook produced" signal, which is what both branches above do:
    // `applyDefineReplacements` substitutes within a line, and the TypeScript
    // strip only runs for modules the emitter did not produce (`?macro=true`
    // artifacts and hand-written virtual modules), which never carried a map to
    // begin with (#3399).
    return transformed === code ? null : { code: transformed, map: null };
  } catch (e: unknown) {
    state.logger.error(`transformWithOxc failed for ${realPath}:`, e);
    let dumpPath: string | null = null;
    try {
      dumpPath = getOxcDumpPath(state.root, realPath);
      fs.writeFileSync(dumpPath, code, "utf-8");
      state.logger.error(`Dumped failing code to ${dumpPath}`);
    } catch (dumpError: unknown) {
      state.logger.error(`Failed to dump failing virtual module for ${realPath}:`, dumpError);
    }
    const message = [
      `[vize] Virtual module transform failed for ${realPath}: ${formatUnknownError(e)}`,
      dumpPath ? `Dumped failing code to ${dumpPath}` : null,
    ]
      .filter(Boolean)
      .join("\n");
    throw new Error(message);
  }
}

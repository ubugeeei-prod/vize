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
  isMacroArtifact = false,
): Promise<TransformResult | null> {
  // Vize loads virtual SFC modules as JavaScript: emitter output is checked at
  // the napi boundary and boundary placeholders are authored JS. Only raw
  // `?macro=true` script code still needs Vite's TypeScript strip.
  try {
    let transformed = isMacroArtifact
      ? (await transformVirtualTypeScript(code, realPath)).code
      : code;
    if (transformed.includes("import.meta.")) {
      transformed = applyDefineReplacements(transformed, getVirtualModuleDefines(state, ssr));
    }
    // `map: null` keeps the load hook's map: define replacements stay within a
    // line, and macro artifacts never carried a map (#3399).
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

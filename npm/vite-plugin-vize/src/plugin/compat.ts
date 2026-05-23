import type { Plugin, TransformResult } from "vite";
import { transformWithOxc } from "vite";
import { createRequire } from "node:module";
import { classifyVitePluginRequest } from "@vizejs/native";

import {
  getCompileOptionsForRequest,
  getEnvironmentCache,
  syncCollectedCssForFile,
  type VizePluginState,
} from "./state.ts";
import { compileFile } from "../compiler.ts";
import { generateOutput } from "../utils/index.ts";
import { scopeCssForPipeline } from "../utils/css.ts";
import { applyDefineReplacements } from "../transform.ts";

export function createVueCompatPlugin(state: VizePluginState): Plugin {
  let compilerSfc: unknown = null;
  const loadCompilerSfc = () => {
    if (!compilerSfc) {
      try {
        const require = createRequire(import.meta.url);
        compilerSfc = require("@vue/compiler-sfc");
      } catch {
        compilerSfc = { parse: () => ({ descriptor: {}, errors: [] }) };
      }
    }
    return compilerSfc;
  };

  return {
    name: "vite:vue",
    api: {
      get options() {
        return {
          compiler: loadCompilerSfc(),
          isProduction: state.isProduction ?? false,
          root: state.root ?? process.cwd(),
          template: {},
        };
      },
    },
  };
}

export function normalizeVirtualStyleId(id: string): string {
  if (!id.startsWith("\0") || !id.includes("?vue")) {
    return id;
  }

  return id
    .slice(1)
    .replace(/\.module\.\w+$/, "")
    .replace(/\.\w+$/, "");
}

export function transformScopedPreprocessorCss(code: string, id: string): string | null {
  const request = classifyVitePluginRequest(normalizeVirtualStyleId(id));
  if (
    !request.isVueStyleQuery ||
    !request.styleScoped ||
    !request.styleLang ||
    request.styleLang === "css"
  ) {
    return null;
  }

  return scopeCssForPipeline(code, request.styleScoped);
}

export function createStylePostTransformPlugin(): Plugin {
  return {
    name: "vize:style-post-transform",
    transform(code: string, id: string): TransformResult | null {
      const scoped = transformScopedPreprocessorCss(code, id);
      return scoped === null ? null : { code: scoped, map: null };
    },
  };
}

// Post-transform plugin to handle virtual SFC content from other plugins.
export function createPostTransformPlugin(state: VizePluginState): Plugin {
  return {
    name: "vize:post-transform",
    enforce: "post",
    async transform(
      code: string,
      id: string,
      transformOptions?: { ssr?: boolean },
    ): Promise<TransformResult | null> {
      if (
        !id.endsWith(".vue") &&
        !id.endsWith(".vue.ts") &&
        !id.includes("node_modules") &&
        id.endsWith(".setup.ts") &&
        /<script\s+setup[\s>]/.test(code)
      ) {
        state.logger.log(`post-transform: compiling virtual SFC content from ${id}`);
        try {
          const isSsr = !!transformOptions?.ssr;
          const compiled = compileFile(
            id,
            getEnvironmentCache(state, isSsr),
            getCompileOptionsForRequest(state, isSsr),
            code,
          );
          syncCollectedCssForFile(state, id, compiled);

          const output = generateOutput(compiled, {
            isProduction: state.isProduction,
            isDev: state.server !== null,
            ssr: isSsr,
            extractCss: state.extractCss,
            filePath: id,
          });

          const result = await transformWithOxc(output, id, { lang: "ts", sourcemap: false });
          const defines = transformOptions?.ssr ? state.serverViteDefine : state.clientViteDefine;
          let transformed = result.code;
          if (Object.keys(defines).length > 0) {
            transformed = applyDefineReplacements(transformed, defines);
          }
          return {
            code: transformed,
            map: null,
          };
        } catch (e: unknown) {
          state.logger.error(`Virtual SFC compilation failed for ${id}:`, e);
        }
      }
      return null;
    },
  };
}

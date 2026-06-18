import type { Plugin, TransformResult } from "vite";
import { createRequire } from "node:module";
import { classifyVitePluginRequest } from "@vizejs/native";

import {
  getCompileOptionsForRequest,
  getEnvironmentCache,
  shouldExtractCssForRequest,
  syncCollectedCssForFile,
  type VizePluginState,
} from "./state.ts";
import { compileFile } from "../compiler.ts";
import { generateOutput } from "../utils/index.ts";
import { scopeCssForPipeline } from "../utils/css.ts";
import { applyDefineReplacements } from "../transform.ts";
import { transformVirtualTypeScript } from "./vite-transform.ts";

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
  const withoutPrefix = id.startsWith("\0") ? id.slice(1) : id;
  if (!withoutPrefix.includes("?vue")) {
    return id;
  }

  return withoutPrefix.replace(/\.module\.\w+$/, "").replace(/\.\w+$/, "");
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

function stripQuery(id: string): string {
  const queryStart = id.search(/[?#]/);
  return queryStart === -1 ? id : id.slice(0, queryStart);
}

function isSfcLikeSource(code: string): boolean {
  return /^<(?:template|script|style)(?:\s|>|\/)/.test(code.trimStart());
}

function shouldPostTransformSfcLikeModule(state: VizePluginState, id: string): boolean {
  const filename = stripQuery(id);
  if (
    filename.endsWith(".vue") ||
    filename.endsWith(".vue.ts") ||
    filename.includes("node_modules")
  ) {
    return false;
  }
  if (filename.endsWith(".setup.ts")) {
    return true;
  }
  if (state.filter(filename) || state.filter(id)) {
    return true;
  }
  return /\.(?:md|markdown)$/i.test(filename);
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
      if (shouldPostTransformSfcLikeModule(state, id) && isSfcLikeSource(code)) {
        state.logger.log(`post-transform: compiling virtual SFC content from ${id}`);
        try {
          const isSsr = !!transformOptions?.ssr;
          const extractCss = shouldExtractCssForRequest(state, isSsr);
          const compiled = compileFile(
            id,
            getEnvironmentCache(state, isSsr),
            getCompileOptionsForRequest(state, isSsr),
            code,
          );
          syncCollectedCssForFile({ ...state, extractCss }, id, compiled);

          const output = generateOutput(compiled, {
            isProduction: state.isProduction,
            isDev: state.server !== null,
            ssr: isSsr,
            extractCss,
            filePath: id,
          });

          const result = await transformVirtualTypeScript(output, id);
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

import fs from "node:fs";
import type { Compiler as WebpackCompiler } from "webpack";
import { createUnplugin } from "unplugin";
import { createFilter } from "./filter.ts";
import { compileVueModule } from "./compiler.ts";
import {
  createVirtualStyleId,
  isVirtualStyleId,
  isVueFile,
  isVueStyleRequest,
  parseVueRequest,
} from "./request.ts";
import { generateOutput, wrapScopedPreprocessorStyle } from "./style.ts";
import { stripTypeScript } from "./strip-types.ts";
import type {
  CachedCompiledModule,
  CompiledModule,
  NormalizedVizeUnpluginOptions,
  VizeUnpluginOptions,
} from "./types.ts";

function normalizeOptions(rawOptions: VizeUnpluginOptions = {}): NormalizedVizeUnpluginOptions {
  const isProduction = rawOptions.isProduction ?? process.env.NODE_ENV === "production";
  return {
    include: rawOptions.include,
    exclude: rawOptions.exclude,
    isProduction,
    ssr: rawOptions.ssr ?? false,
    sourceMap: rawOptions.sourceMap ?? !isProduction,
    vapor: rawOptions.vapor ?? false,
    root: rawOptions.root ?? process.cwd(),
    debug: rawOptions.debug ?? false,
  };
}

function createVueDefineMap(isProduction: boolean): Record<string, string> {
  return {
    __VUE_OPTIONS_API__: JSON.stringify(true),
    __VUE_PROD_DEVTOOLS__: JSON.stringify(!isProduction),
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: JSON.stringify(!isProduction),
  };
}

function injectWebpackVueDefines(compiler: WebpackCompiler, isProduction: boolean): void {
  const { DefinePlugin } = compiler.webpack;
  const existingDefines = new Set<string>();

  for (const plugin of compiler.options.plugins ?? []) {
    const definitions = (plugin as { definitions?: Record<string, unknown> }).definitions;
    if (!definitions) {
      continue;
    }

    for (const key of Object.keys(definitions)) {
      existingDefines.add(key);
    }
  }

  const definitions = createVueDefineMap(isProduction);
  const missingDefinitions: Record<string, string> = {};

  for (const [key, value] of Object.entries(definitions)) {
    if (!existingDefines.has(key)) {
      missingDefinitions[key] = value;
    }
  }

  if (Object.keys(missingDefinitions).length > 0) {
    new DefinePlugin(missingDefinitions).apply(compiler);
  }
}

async function loadStyleBlock(
  id: string,
  options: NormalizedVizeUnpluginOptions,
  cache: Map<string, CachedCompiledModule>,
): Promise<string> {
  const request = parseVueRequest(id);
  const index = request.query.index ?? -1;
  if (index < 0) {
    return "";
  }

  let compiled: CompiledModule | undefined = cache.get(request.filename)?.compiled;

  if (!compiled && fs.existsSync(request.filename)) {
    const source = fs.readFileSync(request.filename, "utf8");
    compiled = compileVueModule(request.filename, source, options, cache).compiled;
  }

  const block = compiled?.styles[index];
  if (!block) {
    return "";
  }

  return wrapScopedPreprocessorStyle(block.content, request.query.scoped, block.lang);
}

export const vizeUnplugin = createUnplugin<VizeUnpluginOptions | undefined>((rawOptions = {}) => {
  const options = normalizeOptions(rawOptions);
  const filter = createFilter(options.include, options.exclude);
  const cache = new Map<string, CachedCompiledModule>();

  return {
    name: "unplugin-vize",

    resolveId(id) {
      if (isVueStyleRequest(id)) {
        return createVirtualStyleId(id);
      }
      return null;
    },

    loadInclude(id) {
      return isVirtualStyleId(id);
    },

    async load(id) {
      if (!isVirtualStyleId(id)) {
        return null;
      }

      return {
        code: await loadStyleBlock(id, options, cache),
        map: null,
      };
    },

    transformInclude(id) {
      const request = parseVueRequest(id);
      return !request.query.vue && isVueFile(request.filename) && filter(request.filename);
    },

    async transform(code, id) {
      if (!isVueFile(id) || !filter(id)) {
        return null;
      }

      const { compiled, warnings } = compileVueModule(id, code, options, cache);
      for (const warning of warnings) {
        this.warn(`[vize] ${warning}`);
      }

      const generated = generateOutput(compiled, {
        isProduction: options.isProduction,
        isDev: false,
        filePath: id,
      });

      const transformed = await stripTypeScript(id, generated, options.sourceMap);
      return {
        code: transformed.code,
        map: transformed.map,
      };
    },

    watchChange(id) {
      if (isVueFile(id)) {
        cache.delete(id);
      }
    },

    webpack(compiler) {
      injectWebpackVueDefines(compiler, options.isProduction);
    },

    esbuild: {
      onResolveFilter: /\.vue(?:$|\?)/,
      onLoadFilter: /\.vue(?:$|\?)/,
      loader(_code, id) {
        const request = parseVueRequest(id);
        if (request.query.type === "style") {
          return request.query.module !== false ? "local-css" : "css";
        }
        return "js";
      },
      config(buildOptions) {
        buildOptions.define = {
          ...createVueDefineMap(options.isProduction),
          ...buildOptions.define,
        };
      },
    },
  };
});

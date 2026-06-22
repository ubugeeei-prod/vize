import fs from "node:fs";
import { createRequire } from "node:module";
import type { Compiler as WebpackCompiler } from "webpack";
import { createUnplugin } from "unplugin";
import { createFilter } from "./filter.ts";
import { compileJsxModule, compileVueModule } from "./compiler.ts";
import {
  createVirtualStyleId,
  isJsxFile,
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
  VizeTemplateSyntax,
  VizeVueVersion,
  VizeUnpluginOptions,
} from "./types.ts";

const require = createRequire(import.meta.url);

type WebpackDefinePluginConstructor = new (definitions: Record<string, string>) => {
  apply(compiler: WebpackCompiler): void;
};

interface WebpackCompilerWithRuntime extends WebpackCompiler {
  webpack?: {
    DefinePlugin?: WebpackDefinePluginConstructor;
  };
}

function normalizeVueVersion(version: VizeUnpluginOptions["vueVersion"]): VizeVueVersion {
  return version ?? 3;
}

function isLegacyVueVersion(version: VizeVueVersion): boolean {
  return (
    version === "legacy" || version === 0.11 || version === 1 || version === 2 || version === "2.7"
  );
}

function normalizeTemplateSyntax(
  templateSyntax: VizeTemplateSyntax | undefined,
): VizeTemplateSyntax {
  return templateSyntax ?? "standard";
}

export function normalizeOptions(
  rawOptions: VizeUnpluginOptions = {},
): NormalizedVizeUnpluginOptions {
  const isProduction = rawOptions.isProduction ?? process.env.NODE_ENV === "production";
  const compatibility = rawOptions.compatibility ?? {};
  const vueVersion = normalizeVueVersion(rawOptions.vueVersion ?? compatibility.vueVersion);
  const mode =
    rawOptions.mode ?? (compatibility.scriptSetupInStandalone === true ? "function" : "module");
  const hostCompiler = compatibility.hostCompiler ?? isLegacyVueVersion(vueVersion);
  const templateSyntax = normalizeTemplateSyntax(rawOptions.templateSyntax);
  return {
    include: rawOptions.include,
    exclude: rawOptions.exclude,
    compatibility,
    isProduction,
    ssr: rawOptions.ssr ?? false,
    sourceMap: rawOptions.sourceMap ?? !isProduction,
    mode,
    vapor: rawOptions.vapor ?? false,
    jsxMode: rawOptions.jsxMode,
    customRenderer: rawOptions.customRenderer ?? false,
    templateSyntax,
    runtimeModuleName: rawOptions.runtimeModuleName ?? "vue",
    runtimeGlobalName: rawOptions.runtimeGlobalName ?? "Vue",
    vueVersion,
    hostCompiler,
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

function resolveWebpackDefinePlugin(
  compiler: WebpackCompilerWithRuntime,
  webpackVersion: 4 | 5 | undefined,
): WebpackDefinePluginConstructor | null {
  if (webpackVersion !== 4 && compiler.webpack?.DefinePlugin) {
    return compiler.webpack.DefinePlugin;
  }

  try {
    const hostWebpack = require("webpack") as {
      DefinePlugin?: WebpackDefinePluginConstructor;
    };
    return hostWebpack.DefinePlugin ?? null;
  } catch {
    return null;
  }
}

export function injectWebpackVueDefines(
  compiler: WebpackCompiler,
  isProduction: boolean,
  webpackVersion?: 4 | 5,
  definePluginConstructor?: WebpackDefinePluginConstructor,
): void {
  const DefinePlugin =
    definePluginConstructor ??
    resolveWebpackDefinePlugin(compiler as WebpackCompilerWithRuntime, webpackVersion);
  if (!DefinePlugin) {
    throw new Error(
      "[vize] Could not resolve webpack DefinePlugin. Install webpack in the host project or disable the Vize compiler with compatibility.hostCompiler.",
    );
  }

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
      if (options.hostCompiler) {
        return null;
      }
      if (isVueStyleRequest(id)) {
        return createVirtualStyleId(id);
      }
      return null;
    },

    loadInclude(id) {
      if (options.hostCompiler) {
        return false;
      }
      return isVirtualStyleId(id);
    },

    async load(id) {
      if (options.hostCompiler) {
        return null;
      }
      if (!isVirtualStyleId(id)) {
        return null;
      }

      return {
        code: await loadStyleBlock(id, options, cache),
        map: null,
      };
    },

    transformInclude(id) {
      if (options.hostCompiler) {
        return false;
      }
      // `.jsx`/`.tsx` modules route to the JSX compiler. They never carry the
      // `?vue` query, so a plain filename check plus the user filter is enough.
      if (isJsxFile(id)) {
        return filter(id);
      }
      // A `.vue` filename (from the path or a `vize-file` query) is required for a
      // match, so ids without a `.vue` substring can never be transformed. Skip
      // URLSearchParams parsing for the common plain-JS imports.
      if (!id.includes(".vue")) {
        return false;
      }
      const request = parseVueRequest(id);
      return !request.query.vue && isVueFile(request.filename) && filter(request.filename);
    },

    async transform(code, id) {
      if (options.hostCompiler) {
        return null;
      }

      if (isJsxFile(id)) {
        if (!filter(id)) {
          return null;
        }
        const { code: jsxCode, map: jsxMap, warnings } = compileJsxModule(id, code, options);
        for (const warning of warnings) {
          this.warn(`[vize] ${warning}`);
        }
        // HMR (deferred, #1533): the JSX compiler emits a render-function-only
        // module with no component object to attach a Vue HMR record to (unlike
        // the `.vue` path's `_sfc_main`), so a state-preserving boundary awaits
        // the JSX component-wrapper output. Source map + preamble plumbing land
        // now (`map` above, preamble inside `compileJsxModule`).
        return {
          code: jsxCode,
          map: jsxMap,
        };
      }

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
      if (!options.hostCompiler) {
        injectWebpackVueDefines(
          compiler,
          options.isProduction,
          options.compatibility.webpackVersion,
        );
      }
    },

    esbuild: {
      onResolveFilter: /\.(?:vue|[jt]sx)(?:$|\?)/,
      onLoadFilter: /\.(?:vue|[jt]sx)(?:$|\?)/,
      loader(_code, id) {
        const request = parseVueRequest(id);
        if (request.query.type === "style") {
          return request.query.module !== false ? "local-css" : "css";
        }
        return "js";
      },
      config(buildOptions) {
        if (options.hostCompiler) {
          return;
        }
        buildOptions.define = {
          ...createVueDefineMap(options.isProduction),
          ...buildOptions.define,
        };
      },
    },
  };
});

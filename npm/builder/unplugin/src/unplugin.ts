import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import type { Compiler as WebpackCompiler } from "webpack";
import { parseSfc } from "@vizejs/native";
import { createUnplugin } from "unplugin";
import { createFilter } from "./filter.ts";
import { compileJsxModule, compileVueModule } from "./compiler.ts";
import { listSfcSrcRequests, resolveSfcSrcRequests } from "../../shared/sfc-src.ts";
import { isJsxFile, isVueFile, isVueStyleRequest, parseVueRequest } from "./request.ts";
import { generateOutput } from "./style.ts";
import {
  isPreprocessorLang,
  preprocessStyle,
  scopeCss,
  transformStyleVars,
} from "./style-scope.ts";
import {
  findVirtualModules,
  registerWebpackStyleRule,
  webpackStylePath,
  webpackStyleRequest,
} from "./webpack-styles.ts";
import { stripTypeScript } from "./strip-types.ts";
import type {
  CachedCompiledModule,
  CompiledModule,
  HostCss,
  NormalizedVizeUnpluginOptions,
  StyleBlockInfo,
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
    experimentalInTagComments: rawOptions.experimentalInTagComments ?? false,
    experimentalPatternedTemplate: rawOptions.experimentalPatternedTemplate ?? false,
    experimentalSelfComponent: rawOptions.experimentalSelfComponent ?? false,
    experimentalStrictSlotChildren: rawOptions.experimentalStrictSlotChildren ?? false,
    experimentalServerScript: rawOptions.experimentalServerScript ?? false,
    jsxMode: rawOptions.jsxMode,
    jsxCompat: rawOptions.jsxCompat,
    customRenderer: rawOptions.customRenderer ?? false,
    customElements: rawOptions.customElements,
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
  context: unknown,
  id: string,
  options: NormalizedVizeUnpluginOptions,
  cache: Map<string, CachedCompiledModule>,
  jsxStyles: Map<string, StyleBlockInfo[]>,
): Promise<string> {
  const request = parseVueRequest(id);
  const index = request.query.index ?? -1;
  if (index < 0) {
    return "";
  }

  if (isJsxFile(request.filename)) {
    // The compiler scoped JSX CSS already.
    const styles =
      jsxStyles.get(request.filename) ??
      (fs.existsSync(request.filename)
        ? compileJsxModule(request.filename, fs.readFileSync(request.filename, "utf8"), options, {
            hostCss: "preprocessors",
          }).styles
        : []);
    return styles[index]?.content ?? "";
  }

  let compiled: CompiledModule | undefined = cache.get(request.filename)?.compiled;

  if (!compiled && fs.existsSync(request.filename)) {
    const source = fs.readFileSync(request.filename, "utf8");
    const resolutions = await resolveSrcRequests(context, request.filename, source);
    compiled = compileVueModule(request.filename, source, options, cache, resolutions).compiled;
  }

  const block = compiled?.styles[index];
  if (!compiled || !block) {
    return "";
  }
  const source = transformStyleVars(
    block.content,
    compiled.scopeId,
    options.isProduction ? request.filename : undefined,
  );

  // Vue order: preprocess, then scope. The id names what this module holds
  // (see styleRequestId). Webpack never loads these ids; its style files go
  // through the host's loaders instead (webpack-styles.ts).
  const css =
    block.lang && isPreprocessorLang(block.lang) && request.query.lang === "css"
      ? await preprocessStyle(source, block.lang, request.filename, options.root)
      : source;
  const scoped = request.query.scoped;
  return scoped ? scopeCss(css, scoped) : css;
}

/**
 * What the host does with an imported stylesheet. esbuild without `bundle`
 * never follows an import, and its CSS loaders compile no preprocessor.
 */
function hostCss(context: unknown, framework: string): HostCss {
  if (framework !== "esbuild") {
    return "preprocessors";
  }
  const native = (
    context as { getNativeBuildContext?: () => unknown }
  ).getNativeBuildContext?.() as { build?: { initialOptions?: { bundle?: boolean } } } | undefined;
  return native?.build?.initialOptions?.bundle === true ? "css" : "none";
}

const CSS_PIPELINE_HINT =
  "[vize] <style> blocks are stylesheet modules for the bundler's CSS pipeline, and this build did not bundle one. " +
  "tsdown needs @tsdown/css; Rollup needs a CSS plugin such as rollup-plugin-postcss; " +
  "Rolldown alone does not bundle CSS, so build through tsdown or Vite.";

interface HostBuildError {
  id?: unknown;
  code?: unknown;
  errors?: HostBuildError[];
}

/** A Rollup or Rolldown build error raised on a style module with no CSS pipeline. */
function isStylePipelineError(error: HostBuildError): boolean {
  return [error, ...(error.errors ?? [])].some(
    (item) =>
      (typeof item.id === "string" && isVueStyleRequest(item.id)) ||
      // Rolldown names no module for a CSS module it cannot bundle.
      item.code === "UNSUPPORTED_FEATURE",
  );
}

type HostResolve = (request: string, importer: string) => Promise<string | null | undefined>;
type RollupLikeResolve = (
  source: string,
  importer: string,
  options: { skipSelf: boolean },
) => Promise<{ id: string; external?: boolean | string } | null | undefined>;
type LoaderResolve = (context: string, request: string) => Promise<string>;
type EsbuildResolve = (
  path: string,
  options: { kind: "import-statement"; resolveDir: string; importer: string },
) => Promise<{ path: string; external: boolean; errors: unknown[] }>;

/** The bundler's own module resolver, or null for a host that offers none. */
function hostResolver(context: unknown): HostResolve | null {
  const ctx = context as { resolve?: RollupLikeResolve; getNativeBuildContext?: () => unknown };
  if (typeof ctx.resolve === "function") {
    const resolve = ctx.resolve;
    return async (request, importer) => {
      const result = await resolve.call(context, request, importer, { skipSelf: true });
      // An external module is not bundled, so there is no file to inline.
      return result && !result.external ? result.id : null;
    };
  }
  const native = ctx.getNativeBuildContext?.() as
    | { framework: string; loaderContext?: { getResolve: () => LoaderResolve } }
    | { framework: "esbuild"; build: { resolve: EsbuildResolve } }
    | undefined;
  if (native && "loaderContext" in native && native.loaderContext) {
    const resolve = native.loaderContext.getResolve();
    return (request, importer) => resolve(path.dirname(importer), request).catch(() => null);
  }
  if (native && "build" in native) {
    const { build } = native;
    return async (request, importer) => {
      const result = await build.resolve(request, {
        kind: "import-statement",
        resolveDir: path.dirname(importer),
        importer,
      });
      return result.errors.length > 0 || result.external ? null : result.path;
    };
  }
  return null;
}

/** Resolve the SFC's `src` module requests (packages, aliases) with the bundler. */
async function resolveSrcRequests(
  context: unknown,
  importer: string,
  source: string,
): Promise<Map<string, string> | undefined> {
  const requests = listSfcSrcRequests(parseSfc, importer, source);
  const resolve = requests.length > 0 ? hostResolver(context) : null;
  return resolve ? resolveSfcSrcRequests(importer, requests, resolve) : undefined;
}

type VizeUnpluginInput = VizeUnpluginOptions | undefined;

export const vizeUnplugin = createUnplugin<VizeUnpluginInput>((rawOptions = {}, meta) => {
  const options = normalizeOptions(rawOptions);
  const filter = createFilter(options.include, options.exclude);
  const cache = new Map<string, CachedCompiledModule>();
  const jsxStyles = new Map<string, StyleBlockInfo[]>();
  const webpackStyleFiles = meta.framework === "webpack";
  let virtualModules: ReturnType<typeof findVirtualModules>;
  /** A webpack style block is a virtual file the host's rules match (webpack-styles.ts). */
  const writeWebpackStyle = (
    id: string,
    block: StyleBlockInfo,
    scoped: string | null,
    content: string,
  ): string => {
    const stylePath = webpackStylePath(id, block, scoped);
    if (!virtualModules) {
      throw new Error("[vize] webpack style blocks need the plugin's virtual modules.");
    }
    virtualModules.writeModule(stylePath, content);
    return webpackStyleRequest(stylePath, block);
  };
  let loadedStyles = false;
  const rollupHooks = {
    buildStart() {
      loadedStyles = false;
    },
    buildEnd(this: { warn(message: string): void }, error?: HostBuildError) {
      if (error && loadedStyles && isStylePipelineError(error)) {
        this.warn(CSS_PIPELINE_HINT);
      }
    },
  };

  return {
    name: "unplugin-vize",

    resolveId(id) {
      if (options.hostCompiler) {
        return null;
      }
      // Claim the request so the bundler does not look for it on disk.
      return isVueStyleRequest(id) ? id : null;
    },

    loadInclude(id) {
      if (options.hostCompiler) {
        return false;
      }
      return !webpackStyleFiles && isVueStyleRequest(id);
    },

    async load(id) {
      if (options.hostCompiler) {
        return null;
      }
      if (!isVueStyleRequest(id)) {
        return null;
      }

      loadedStyles = true;
      return {
        code: await loadStyleBlock(this, id, options, cache, jsxStyles),
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
        const {
          code: jsxCode,
          map: jsxMap,
          warnings,
          styles,
        } = compileJsxModule(id, code, options, {
          hostCss: hostCss(this, meta.framework),
          styleRequest: webpackStyleFiles
            ? (block, scoped) => writeWebpackStyle(id, block, scoped, block.content)
            : undefined,
        });
        jsxStyles.set(id, styles);
        for (const warning of warnings) {
          this.warn(`[vize] ${warning}`);
        }
        // HMR (deferred, #1533): the JSX compiler emits a render-function-only
        // module with no component object to attach a Vue HMR record to (unlike
        // the `.vue` path's `_sfc_main`), so a state-preserving boundary awaits
        // the JSX component-wrapper output.
        return {
          code: jsxCode,
          map: jsxMap,
        };
      }

      if (!isVueFile(id) || !filter(id)) {
        return null;
      }

      const resolutions = await resolveSrcRequests(this, id, code);
      const { compiled, warnings, dependencies } = compileVueModule(
        id,
        code,
        options,
        cache,
        resolutions,
      );
      for (const warning of warnings) {
        this.warn(`[vize] ${warning}`);
      }
      for (const dependency of dependencies) {
        this.addWatchFile(dependency);
      }

      const generated = generateOutput(compiled, {
        isProduction: options.isProduction,
        isDev: false,
        hostCss: hostCss(this, meta.framework),
        filePath: id,
        styleRequest: webpackStyleFiles
          ? (block, scoped) =>
              writeWebpackStyle(
                id,
                block,
                scoped,
                transformStyleVars(
                  block.content,
                  compiled.scopeId,
                  options.isProduction ? id : undefined,
                ),
              )
          : undefined,
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
      jsxStyles.delete(id);
    },

    rollup: rollupHooks,
    rolldown: rollupHooks,

    webpack(compiler) {
      if (!options.hostCompiler) {
        virtualModules = findVirtualModules(compiler);
        registerWebpackStyleRule(compiler);
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

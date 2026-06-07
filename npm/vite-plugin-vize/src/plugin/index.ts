/** Main Vize Vite plugin implementation. */

import type { Plugin, ResolvedConfig, ViteDevServer } from "vite";

import type { VizeOptions, ConfigEnv, ResolvedVizeConfig } from "../types.ts";
import { createFilter } from "../utils/index.ts";
import { toBrowserImportPrefix } from "../virtual.ts";
import { shouldApplyDefineInVirtualModule, createLogger } from "../transform.ts";
import { loadConfig, resolveConfigExport, vizeConfigStore } from "../config.ts";
import {
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  DEFAULT_PRECOMPILE_IGNORE_PATTERNS,
  clearBuildCaches,
  type VizePluginState,
  compileAll,
  normalizePrecompileBatchSize,
} from "./state.ts";
import { resolveIdHook } from "./resolve.ts";
import { loadHook, transformHook } from "./load.ts";
import {
  handleHotUpdateHook,
  handleGenerateBundleHook,
  resolveComponentsCssFileName,
} from "./hmr.ts";
import {
  createPostTransformPlugin,
  createStylePostTransformPlugin,
  createVueCompatPlugin,
} from "./compat.ts";
import { patchUnoCssBridge } from "./unocss.ts";
import { patchQuasarBridge } from "./quasar.ts";
import { patchCssModuleGenerateScopedName } from "./css-modules.ts";
import { installVirtualAssetMiddleware } from "./dev-middleware.ts";
import {
  createLegacyVueCompatibilityPlugin,
  isLegacyVueCompatibilityMode,
  isLegacyVueVersion,
} from "./vue-version.ts";

export type { VizePluginState } from "./state.ts";

function aliasSortKey(find: string | RegExp): number {
  return typeof find === "string" ? find.length : find.source.length;
}

function mergeSharedConfig(
  baseConfig: ResolvedVizeConfig | null,
  overrideConfig: ResolvedVizeConfig | null,
): ResolvedVizeConfig | null {
  if (!baseConfig) return overrideConfig;
  if (!overrideConfig) return baseConfig;

  return {
    ...baseConfig,
    ...overrideConfig,
    compiler: {
      ...baseConfig.compiler,
      ...overrideConfig.compiler,
    },
    vite: {
      ...baseConfig.vite,
      ...overrideConfig.vite,
    },
    linter: {
      ...baseConfig.linter,
      ...overrideConfig.linter,
    },
    typeChecker: {
      ...baseConfig.typeChecker,
      ...overrideConfig.typeChecker,
    },
    formatter: {
      ...baseConfig.formatter,
      ...overrideConfig.formatter,
    },
    languageServer: {
      ...baseConfig.languageServer,
      ...overrideConfig.languageServer,
    },
    musea: {
      ...baseConfig.musea,
      ...overrideConfig.musea,
    },
    globalTypes: {
      ...baseConfig.globalTypes,
      ...overrideConfig.globalTypes,
    },
    entries: [...baseConfig.entries, ...overrideConfig.entries],
  };
}

function shouldExtractCssForBuild(
  state: Pick<VizePluginState, "extractCss" | "isProduction">,
  context: { environment?: { name?: string } },
): boolean {
  if (!state.isProduction) {
    return false;
  }

  const environmentName = context.environment?.name;
  if (environmentName === "client" || environmentName === "browser") {
    return true;
  }
  if (environmentName === "ssr" || environmentName === "server") {
    return false;
  }

  return state.extractCss;
}

function resolveCompatibilityOptions(
  options: VizeOptions,
  compilerConfig: ResolvedVizeConfig["compiler"] = {},
): NonNullable<VizeOptions["compatibility"]> {
  const compatibility = {
    ...compilerConfig.compatibility,
    ...options.compatibility,
  };
  const vueVersion = options.vueVersion ?? compatibility.vueVersion ?? 3;

  if (compatibility.hostCompiler === undefined && isLegacyVueVersion(vueVersion)) {
    compatibility.hostCompiler = true;
  }

  return compatibility;
}

export function vize(options: VizeOptions = {}): Plugin[] {
  if (isLegacyVueCompatibilityMode(options)) {
    return [createLegacyVueCompatibilityPlugin(options)];
  }

  const state: VizePluginState = {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    viteResolveCache: new Map(),
    isProduction: false,
    root: "",
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: null,
    precompileBatchSize: DEFAULT_PRECOMPILE_BATCH_SIZE,
    ignorePatterns: [],
    mergedOptions: options,
    initialized: false,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: createLogger(options.debug ?? false),
  };

  const mainPlugin: Plugin = {
    name: "vite-plugin-vize",
    enforce: "pre",

    config(userConfig, env) {
      patchCssModuleGenerateScopedName(userConfig);

      return {
        // Vue 3 ESM bundler build requires these compile-time feature flags.
        // @vitejs/plugin-vue normally provides them; vize must do so as its replacement.
        define: {
          __VUE_OPTIONS_API__: true,
          __VUE_PROD_DEVTOOLS__: env.command === "serve",
          __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: false,
        },
        optimizeDeps: {
          exclude: ["virtual:vize-styles"],
        },
      };
    },

    async configResolved(resolvedConfig: ResolvedConfig) {
      state.root = options.root ?? resolvedConfig.root;
      state.isProduction = options.isProduction ?? resolvedConfig.isProduction;

      const isSsrBuild = !!resolvedConfig.build?.ssr;
      const currentBase =
        resolvedConfig.command === "serve"
          ? (options.devUrlBase ?? resolvedConfig.base ?? "/")
          : (resolvedConfig.base ?? "/");
      if (isSsrBuild) {
        state.serverViteBase = currentBase;
      } else {
        state.clientViteBase = currentBase;
      }
      state.extractCss = state.isProduction && !isSsrBuild;
      state.componentsCssFileName = resolveComponentsCssFileName(resolvedConfig.build.assetsDir);

      // Capture Vite define values for applying to virtual modules. Vite's
      // built-in define plugin may not process \0-prefixed virtual modules, so
      // the transform hook mirrors the environment-sensitive replacements that
      // are safe to inline.
      // IMPORTANT: Nuxt shares the same plugin instance for client and server builds,
      // each calling configResolved with environment-specific defines. We must store
      // them separately to avoid the server's `document: "undefined"` leaking into
      // client transforms, or the client's `import.meta.server: false` into server ones.
      const isSsr = !!resolvedConfig.build?.ssr;
      const envDefine: Record<string, string> = {};
      if (resolvedConfig.define) {
        for (const [key, value] of Object.entries(resolvedConfig.define)) {
          if (!shouldApplyDefineInVirtualModule(key)) continue;
          if (typeof value === "string") {
            envDefine[key] = value;
          } else {
            envDefine[key] = JSON.stringify(value);
          }
        }
      }
      if (isSsr) {
        state.serverViteDefine = envDefine;
      } else {
        state.clientViteDefine = envDefine;
      }

      const configEnv: ConfigEnv = {
        mode: resolvedConfig.mode,
        command: resolvedConfig.command === "build" ? "build" : "serve",
        isSsrBuild: !!resolvedConfig.build?.ssr,
      };

      let fileConfig: ResolvedVizeConfig | null = null;
      if (options.configMode !== false) {
        try {
          fileConfig = await loadConfig(state.root, {
            mode: options.configMode ?? "root",
            configFile: options.configFile,
            env: configEnv,
          });
          if (fileConfig) {
            state.logger.log("Loaded config from vize.config file");
          }
        } catch (error) {
          state.logger.warn(
            `Failed to load vize config from ${options.configFile ?? state.root}:`,
            error,
          );
        }
      }

      let inlineConfig: ResolvedVizeConfig | null = null;
      if (options.config) {
        try {
          inlineConfig = await resolveConfigExport(options.config, configEnv);
          state.logger.log("Loaded inline vize config from plugin options");
        } catch (error) {
          state.logger.warn("Failed to resolve inline vize config:", error);
        }
      }

      const sharedConfig = mergeSharedConfig(fileConfig, inlineConfig);
      if (sharedConfig) {
        vizeConfigStore.set(state.root, sharedConfig);
      }

      const viteConfig = sharedConfig?.vite ?? {};
      const compilerConfig = sharedConfig?.compiler ?? {};
      const compatibility = resolveCompatibilityOptions(options, compilerConfig);
      const vueVersion = options.vueVersion ?? compatibility.vueVersion ?? 3;
      const mode =
        options.mode ??
        compilerConfig.mode ??
        (compatibility.scriptSetupInStandalone === true ? "function" : "module");
      const templateSyntax = options.templateSyntax ?? compilerConfig.templateSyntax ?? "standard";

      state.mergedOptions = {
        ...options,
        ssr: options.ssr ?? compilerConfig.ssr ?? false,
        sourceMap: options.sourceMap ?? compilerConfig.sourceMap,
        vapor: options.vapor ?? compilerConfig.vapor ?? false,
        customRenderer: options.customRenderer ?? compilerConfig.customRenderer ?? false,
        templateSyntax,
        compatibility,
        vueVersion,
        mode,
        runtimeModuleName: options.runtimeModuleName ?? compilerConfig.runtimeModuleName ?? "vue",
        runtimeGlobalName: options.runtimeGlobalName ?? compilerConfig.runtimeGlobalName ?? "Vue",
        include: options.include ?? viteConfig.include,
        exclude: options.exclude ?? viteConfig.exclude,
        scanPatterns: options.scanPatterns ?? viteConfig.scanPatterns,
        precompileBatchSize: options.precompileBatchSize ?? viteConfig.precompileBatchSize,
        ignorePatterns: options.ignorePatterns ?? viteConfig.ignorePatterns,
      };

      state.dynamicImportAliasRules = [];
      for (const alias of resolvedConfig.resolve.alias) {
        if (typeof alias.find !== "string" || typeof alias.replacement !== "string") {
          continue;
        }
        const fromPrefix = alias.find.endsWith("/") ? alias.find : `${alias.find}/`;
        const replacement = toBrowserImportPrefix(alias.replacement);
        const toPrefix = replacement.endsWith("/") ? replacement : `${replacement}/`;
        state.dynamicImportAliasRules.push({ fromPrefix, toPrefix });
      }
      // Prefer longer alias keys first (e.g. "@@" before "@")
      state.dynamicImportAliasRules.sort((a, b) => b.fromPrefix.length - a.fromPrefix.length);

      // Build CSS alias rules for @import resolution (use filesystem paths, not browser paths)
      state.cssAliasRules = [];
      for (const alias of resolvedConfig.resolve.alias) {
        if (
          !(typeof alias.find === "string" || alias.find instanceof RegExp) ||
          typeof alias.replacement !== "string"
        ) {
          continue;
        }
        state.cssAliasRules.push({
          find: alias.find,
          replacement: alias.replacement,
        });
      }
      // Prefer longer alias keys first
      state.cssAliasRules.sort((a, b) => aliasSortKey(b.find) - aliasSortKey(a.find));

      if (isLegacyVueCompatibilityMode(state.mergedOptions)) {
        state.filter = () => false;
        state.scanPatterns = [];
      } else {
        state.filter = createFilter(state.mergedOptions.include, state.mergedOptions.exclude);
        state.scanPatterns = state.mergedOptions.scanPatterns ?? ["**/*.vue"];
      }
      state.precompileBatchSize = normalizePrecompileBatchSize(
        state.mergedOptions.precompileBatchSize,
      );
      state.ignorePatterns = state.mergedOptions.ignorePatterns ?? [
        ...DEFAULT_PRECOMPILE_IGNORE_PATTERNS,
      ];
      patchUnoCssBridge(
        resolvedConfig.plugins as Array<{
          name?: string;
          transform?: Function;
        }>,
      );
      patchQuasarBridge(
        resolvedConfig.plugins as Array<{
          name?: string;
          transform?: Function;
        }>,
      );
      state.initialized = true;
    },

    configureServer(devServer: ViteDevServer) {
      state.server = devServer;
      installVirtualAssetMiddleware(devServer, state);
    },

    async buildStart() {
      state.viteResolveCache?.clear();
      if (!state.scanPatterns || state.scanPatterns.length === 0) {
        // Running in standalone rolldown context (e.g., ox-content OG image)
        // where configResolved is not called, or a framework integration has
        // opted into on-demand compilation. Skip pre-compilation.
        return;
      }
      await compileAll({ ...state, extractCss: shouldExtractCssForBuild(state, this) });
      state.logger.log("Cache keys:", [...state.cache.keys()].slice(0, 3));
    },

    resolveId(id, importer, options) {
      return resolveIdHook(this, state, id, importer, options);
    },

    load(id, loadOptions) {
      return loadHook(state, id, loadOptions);
    },

    async transform(code, id, transformOptions) {
      return transformHook(state, code, id, transformOptions);
    },

    async handleHotUpdate(ctx) {
      return handleHotUpdateHook(state, ctx);
    },

    generateBundle(_, bundle) {
      handleGenerateBundleHook(
        { ...state, extractCss: shouldExtractCssForBuild(state, this) },
        this.emitFile.bind(this),
        bundle,
      );
    },

    closeBundle() {
      if (state.server === null) {
        clearBuildCaches(state);
      }
    },
  };

  return [
    createVueCompatPlugin(state),
    mainPlugin,
    createStylePostTransformPlugin(),
    createPostTransformPlugin(state),
  ];
}

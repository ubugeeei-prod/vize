/** Main Vize Vite plugin implementation. */

import type { Plugin, ResolvedConfig, ViteDevServer } from "vite";

import type { VizeOptions, ConfigEnv } from "../types.ts";
import { createFilter } from "../utils/index.ts";
import { toBrowserImportPrefix } from "../virtual.ts";
import { shouldApplyDefineInVirtualModule, createLogger } from "../transform.ts";
import {
  DEFAULT_PRECOMPILE_BATCH_SIZE,
  DEFAULT_PRECOMPILE_IGNORE_PATTERNS,
  clearBuildCaches,
  type VizePluginState,
  normalizePrecompileBatchSize,
} from "./state.ts";
import { CompiledModuleCache } from "./compiled-module-cache.ts";
import { compileAll } from "./precompile-run.ts";
import { resolveIdHook } from "./resolve.ts";
import { loadHook, transformHook } from "./load.ts";
import {
  handleGenerateBundleHook,
  handleHotUpdateHook,
  resolveComponentsCssFileName,
} from "./hmr.ts";
import { handleHotUpdateEnvironmentHook } from "./hot-update-environment.ts";
import {
  createPostTransformPlugin,
  createStylePostTransformPlugin,
  createVueCompatPlugin,
} from "./compat.ts";
import { patchUnoCssBridge } from "./unocss.ts";
import { patchQuasarBridge } from "./quasar.ts";
import { patchCssModuleGenerateScopedName } from "./css-modules.ts";
import { installDevMiddleware } from "./dev-middleware.ts";
import { resolveExperimentalCompilerOptions } from "./experimentals.ts";
import { createLegacyVueCompatibilityPlugin, isLegacyVueCompatibilityMode } from "./vue-version.ts";
import { resolveSharedConfig } from "./shared-config.ts";
import * as configBridge from "./config-lifecycle.ts";
import { resolveVueFeatureDefines } from "./vue-feature-defines.ts";
import {
  aliasSortKey,
  resolveCompatibilityOptions,
  shouldExtractCssForBuild,
} from "./index-helpers.ts";

export type { VizePluginState } from "./state.ts";

export function vize(options: VizeOptions = {}): Plugin[] {
  if (isLegacyVueCompatibilityMode(options)) {
    return [createLegacyVueCompatibilityPlugin(options)];
  }

  const state: VizePluginState = {
    // Indexed maps: hot updates need the SFCs owning a changed `src` dependency
    // in constant time rather than by scanning both caches. See
    // `compiled-module-cache.ts`.
    cache: new CompiledModuleCache(),
    ssrCache: new CompiledModuleCache(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    viteResolveCache: new Map(),
    isProduction: false,
    viteBuildSourcemap: false,
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

    config(userConfig) {
      patchCssModuleGenerateScopedName(userConfig);

      return {
        // Vue 3 ESM bundler flags normally injected by @vitejs/plugin-vue.
        define: resolveVueFeatureDefines(options.features, userConfig.define),
        optimizeDeps: {
          exclude: ["virtual:vize-styles"],
        },
      };
    },

    async configResolved(resolvedConfig: ResolvedConfig) {
      state.root = options.root ?? resolvedConfig.root;
      state.isProduction = options.isProduction ?? resolvedConfig.isProduction;
      state.viteBuildSourcemap = !!resolvedConfig.build?.sourcemap;

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

      const sharedConfigPromise = resolveSharedConfig(options, state.root, configEnv, state.logger);
      // Vite runs configResolved hooks in parallel. Register the pending lookup
      // before yielding so companion plugins can await this exact config.
      const sharedConfig = await configBridge.register(
        resolvedConfig,
        state.root,
        sharedConfigPromise,
      );

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
        ...resolveExperimentalCompilerOptions(options, compilerConfig, sharedConfig?.experimentals),
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
      configBridge.configureServerCleanup(devServer);
      installDevMiddleware(devServer, state);
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
      return loadHook(state, id, {
        ...loadOptions,
        addWatchFile: this.addWatchFile.bind(this),
      });
    },

    async transform(code, id, transformOptions) {
      return transformHook(state, code, id, transformOptions);
    },

    async hotUpdate(options) {
      return handleHotUpdateEnvironmentHook(state, this.environment, options);
    },

    shouldTransformCachedModule({ id }: { id?: string }) {
      return id?.includes(".vue") ? true : undefined;
    },

    // Vite 7.3+ prefers `hotUpdate` when both hooks exist. Keep this deprecated
    // shim for plugin-vue drop-in compatibility and older Vite integrations.
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
        configBridge.unregisterBuild(this);
        clearBuildCaches(state);
      }
    },
  };

  return [
    createVueCompatPlugin(state, options),
    mainPlugin,
    createStylePostTransformPlugin(),
    createPostTransformPlugin(state),
  ];
}

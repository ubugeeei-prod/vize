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
import { processFilesystemEpoch } from "./filesystem-epoch.ts";
import { compileAll, PrecompileDiagnosticsError } from "./precompile-run.ts";
import { precompileInputOwners } from "./precompile-owner.ts";
import { resolveIdHook } from "./resolve.ts";
import { LOAD_ID_FILTER, TRANSFORM_ID_FILTER, loadHook, transformHook } from "./load.ts";
import { getSfcModuleRequest, requestResolvedSfc, transformSfcModule } from "./sfc-module.ts";
import { resetDemandCompiles, setCompileInputOwners } from "./demand-compile.ts";
import {
  handleGenerateBundleHook,
  handleHotUpdateHook,
  resolveComponentsCssFileName,
} from "./hmr.ts";
import { handleHotUpdateEnvironmentHook } from "./hot-update-environment.ts";
import { createManifestPlugin } from "./manifest.ts";
import {
  createPostTransformPlugin,
  createStylePostTransformPlugin,
  createVueCompatPlugin,
} from "./compat.ts";
import { patchUnoCssBridge } from "./unocss.ts";
import { patchQuasarBridge } from "./quasar.ts";
import { patchVoidIslandTransform } from "./void-islands.ts";
import { patchCssModuleGenerateScopedName } from "./css-modules.ts";
import { installDevMiddleware } from "./dev-middleware.ts";
import { mergeCompilerOptions, type VizeCompilerDefaults } from "./compiler-config.ts";
import { createFileCompilerOptions } from "./compiler-scopes.ts";
import { registerPluginPlan } from "../compiler.ts";
import {
  createLegacyVueCompatibilityPlugin,
  isLegacyVueCompatibilityMode,
  removeHostVueSfcCompilers,
} from "./vue-version.ts";
import { resolveSharedConfig } from "./shared-config.ts";
import * as configBridge from "./config-lifecycle.ts";
import { resolveVueFeatureDefines } from "./vue-feature-defines.ts";
import { resetTypeProjectWatches } from "./type-project-watches.ts";
import {
  aliasSortKey,
  resolveBuildStartEnvironment,
  resolveBuildStartSsr,
  resolveHookSsr,
  shouldExtractCssForBuild,
} from "./index-helpers.ts";

export type { VizePluginState } from "./state.ts";

/** Framework-derived defaults yield to project, entry, and explicit plugin options. */
export function vize(
  options: VizeOptions = {},
  compilerDefaults: VizeCompilerDefaults = {},
  /** @internal Only the Nuxt constructor supplies its host configuration context. */
  hostContext?: "nuxt",
): Plugin[] {
  if (isLegacyVueCompatibilityMode(options)) {
    return [createLegacyVueCompatibilityPlugin(options)];
  }

  let precompileConfig: ResolvedConfig | undefined;
  // The environments that began a build since this instance's config was resolved.
  const startedEnvironments = new Set<string>();
  const state: VizePluginState = {
    // Indexed maps: hot updates need the SFCs owning a changed `src` dependency
    // in constant time rather than by scanning both caches. See
    // `compiled-module-cache.ts`. Both resolve path identities in the process
    // epoch, which the hooks below end on every sign of a filesystem change.
    cache: new CompiledModuleCache(processFilesystemEpoch),
    ssrCache: new CompiledModuleCache(processFilesystemEpoch),
    filesystemEpoch: processFilesystemEpoch,
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    ssrPrecompileMetadata: new Map(),
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
    mergedOptions:
      compilerDefaults.whitespace === undefined
        ? options
        : { ...options, whitespace: options.whitespace ?? compilerDefaults.whitespace },
    initialized: false,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: createLogger(options.debug ?? false),
  };

  // Compiles at @vitejs/plugin-vue's position: after host transforms of the SFC text.
  const sfcPlugin: Plugin = {
    name: "vite-plugin-vize:sfc",
    transform: {
      // Every SFC main module id names its `.vue` file; the host skips the rest.
      filter: { id: /\.vue/ },
      async handler(code, id, transformOptions) {
        const sfc = getSfcModuleRequest(state, id);
        if (!sfc) return null;
        return transformSfcModule(state, code, sfc, {
          ssr: resolveHookSsr(this, transformOptions),
          addWatchFile: this.addWatchFile.bind(this),
          resolve: (request, importer) => this.resolve(request, importer, { skipSelf: true }),
        });
      },
    },
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
      // A build or a dev server begins: nothing observed earlier is assumed.
      processFilesystemEpoch.advance();
      startedEnvironments.clear();
      precompileConfig = resolvedConfig;
      state.root = options.root ?? resolvedConfig.root;
      state.viteCacheDir = resolvedConfig.cacheDir;
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

      const sharedConfigPromise = resolveSharedConfig(
        options,
        state.root,
        configEnv,
        state.logger,
        resolvedConfig,
        hostContext,
      );
      // Vite runs configResolved hooks in parallel. Register the pending lookup
      // before yielding so companion plugins can await this exact config.
      const sharedConfig = await configBridge.register(
        resolvedConfig,
        state.root,
        sharedConfigPromise,
      );

      state.mergedOptions = mergeCompilerOptions(options, sharedConfig, compilerDefaults);
      state.pluginPlan = registerPluginPlan(sharedConfig);
      const replaced = removeHostVueSfcCompilers(
        resolvedConfig.plugins as Plugin[],
        state.mergedOptions,
        sfcPlugin,
      );
      if (replaced > 0) {
        state.logger.log(`Replaced ${replaced} @vitejs/plugin-vue instance(s) with Vize.`);
      }
      state.fileCompilerOptions = createFileCompilerOptions(
        sharedConfig,
        state.root,
        options,
        compilerDefaults,
      );
      state.compilerScopeIdentity = sharedConfig?.entries;

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
        // The startup scan is opt-in: without configured patterns nothing is
        // compiled before the module graph asks for it.
        state.scanPatterns = state.mergedOptions.scanPatterns ?? [];
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
      patchVoidIslandTransform(resolvedConfig.plugins);
      state.initialized = true;
    },

    configureServer(devServer: ViteDevServer) {
      state.server = devServer;
      // Vite forwards file events to `watchChange`, but not directory events,
      // and a remounted workspace link is a directory event.
      devServer.watcher.on("all", () => processFilesystemEpoch.advance());
      configBridge.configureServerCleanup(devServer);
      installDevMiddleware(devServer, state);
    },

    async buildStart() {
      const environment = resolveBuildStartEnvironment(this, precompileConfig);
      // The environments configured together build one snapshot of the sources.
      // A build with no Vite config resolution (standalone Rolldown), or of an
      // environment that already built, starts after a gap nothing reported.
      if (!state.initialized || startedEnvironments.has(environment)) {
        processFilesystemEpoch.advance();
        startedEnvironments.clear();
      }
      startedEnvironments.add(environment);
      resetTypeProjectWatches(state.cache);
      state.viteResolveCache?.clear();
      const inputOwners = precompileInputOwners(this, precompileConfig);
      setCompileInputOwners(state, inputOwners);
      if (!state.scanPatterns || state.scanPatterns.length === 0) {
        // No startup scan was asked for: each environment compiles the SFCs its
        // module graph requests, in batches (`demand-compile.ts`). Also a
        // standalone rolldown context (e.g., ox-content OG image) where
        // configResolved is not called.
        return;
      }
      try {
        await compileAll(
          { ...state, extractCss: shouldExtractCssForBuild(state, this) },
          inputOwners,
          (request, importer) => this.resolve(request, importer, { skipSelf: true }),
          this.addWatchFile.bind(this),
          resolveBuildStartSsr(this, precompileConfig),
        );
      } catch (error) {
        if (!(error instanceof PrecompileDiagnosticsError)) throw error;
        state.logger.warn(`${error.message}\nStartup scan diagnostics are deferred until import.`);
      }
      state.logger.log("Cache keys:", [...state.cache.keys()].slice(0, 3));
    },

    // No filter: an import from an SFC is resolved against the SFC's real path
    // whatever its specifier is, and a hook filter cannot see the importer.
    resolveId(id, importer, options) {
      const ssr = resolveHookSsr(this, options);
      const hookOptions: { ssr: boolean; scan?: boolean } = { ...options, ssr };
      const resolving = resolveIdHook(this, state, id, importer, hookOptions);
      // A dependency scan reads SFCs; it does not ask this environment to compile them.
      if (hookOptions.scan || !id.includes(".vue")) return resolving;
      return resolving.then((resolved) => {
        requestResolvedSfc(state, resolved, ssr);
        return resolved;
      });
    },

    load: {
      filter: { id: LOAD_ID_FILTER },
      handler(id, loadOptions) {
        return loadHook(state, id, {
          ssr: resolveHookSsr(this, loadOptions),
          addWatchFile: (file) => this.addWatchFile(file),
        });
      },
    },

    transform: {
      filter: { id: TRANSFORM_ID_FILTER },
      async handler(code, id, transformOptions) {
        return transformHook(state, code, id, {
          ...transformOptions,
          ssr: resolveHookSsr(this, transformOptions),
        });
      },
    },

    watchChange() {
      processFilesystemEpoch.advance();
    },

    async hotUpdate(options) {
      processFilesystemEpoch.advance();
      return handleHotUpdateEnvironmentHook(state, this.environment, options);
    },

    shouldTransformCachedModule({ id }: { id?: string }) {
      return id?.includes(".vue") ? true : undefined;
    },

    // Vite 7.3+ prefers `hotUpdate` when both hooks exist. Keep this deprecated
    // shim for plugin-vue drop-in compatibility and older Vite integrations.
    async handleHotUpdate(ctx) {
      processFilesystemEpoch.advance();
      const container =
        ctx.server.environments?.client?.pluginContainer ?? ctx.server.pluginContainer;
      return handleHotUpdateHook(state, ctx, {
        resolve: (request, importer) => container.resolveId(request, importer),
      });
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
        resetDemandCompiles(state);
      }
    },
  };

  return [
    createVueCompatPlugin(state, options),
    mainPlugin,
    sfcPlugin,
    createStylePostTransformPlugin(),
    createPostTransformPlugin(state),
    // Nuxt restores source identities in its own client manifest.
    ...(hostContext === "nuxt" ? [] : [createManifestPlugin()]),
  ];
}

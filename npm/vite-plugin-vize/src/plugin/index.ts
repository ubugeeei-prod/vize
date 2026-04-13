/**
 * Main Vize Vite plugin implementation.
 *
 * Contains the `vize()` factory function that creates the Vite plugin array.
 * Hook implementations are split across sub-modules:
 * - state.ts: VizePluginState type + compileAll batch compilation
 * - resolve.ts: resolveId hook + resolveVuePath
 * - load.ts: load + transform hooks
 * - hmr.ts: handleHotUpdate + generateBundle hooks
 * - compat.ts: vueCompatPlugin + postTransformPlugin
 */

import type { Plugin, ResolvedConfig, ViteDevServer } from "vite";
import fs from "node:fs";

import type { VizeOptions, ConfigEnv } from "../types.js";
import { createFilter } from "../utils/index.js";
import { toBrowserImportPrefix } from "../virtual.js";
import { isBuiltinDefine, createLogger } from "../transform.js";
import { loadConfig, vizeConfigStore } from "../config.js";
import { type VizePluginState, compileAll } from "./state.js";
import { resolveIdHook } from "./resolve.js";
import { loadHook, transformHook } from "./load.js";
import { handleHotUpdateHook, handleGenerateBundleHook } from "./hmr.js";
import { createVueCompatPlugin, createPostTransformPlugin } from "./compat.js";

export type { VizePluginState } from "./state.js";

export function vize(options: VizeOptions = {}): Plugin[] {
  const state: VizePluginState = {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root: "",
    clientViteBase: "/",
    serverViteBase: "/",
    server: null,
    filter: () => true,
    scanPatterns: null,
    ignorePatterns: [],
    mergedOptions: options,
    initialized: false,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    clientViteDefine: {},
    serverViteDefine: {},
    logger: createLogger(options.debug ?? false),
  };

  const mainPlugin: Plugin = {
    name: "vite-plugin-vize",
    enforce: "pre",

    config(userConfig, env) {
      // Wrap custom generateScopedName to clean up virtual module filenames.
      // Must be done here (not configResolved) because the resolved config is frozen.
      // When Vize delegates CSS module processing to Vite, the virtual module ID
      // (e.g., \0/abs/path/Comp.vue?vue&type=style&...module.scss) is passed as
      // the "filename" parameter. Strip the \0 prefix and appended suffixes so
      // that user-defined generateScopedName receives the real .vue file path.
      const cssModules = userConfig.css?.modules;
      if (cssModules && typeof cssModules.generateScopedName === "function") {
        const origFn = cssModules.generateScopedName;
        cssModules.generateScopedName = function (name: string, filename: string, css: string) {
          let clean = filename;
          // Vite's postcss-modules resolves the virtual module ID against root,
          // producing paths like: /project/root/\0/abs/path/Comp.vue?...module.scss
          // The \0 (NUL byte) may be in the middle, not at the start.
          // Extract the real path after the NUL marker.
          const nulIdx = clean.indexOf("\0");
          if (nulIdx >= 0) {
            clean = clean.slice(nulIdx + 1);
          }
          // Remove .module.{lang} and .{lang} suffixes appended by resolveId
          clean = clean.replace(/\.module\.\w+$/, "").replace(/\.\w+$/, "");
          // Extract just the file path (before query params)
          if (clean.includes("?")) {
            clean = clean.split("?")[0];
          }
          return origFn.call(this, name, clean, css);
        };
      }

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
      state.extractCss = state.isProduction;

      // Capture custom Vite define values for applying to virtual modules.
      // Vite's built-in define plugin may not process \0-prefixed virtual modules,
      // so we apply replacements ourselves in the transform hook.
      // IMPORTANT: Nuxt shares the same plugin instance for client and server builds,
      // each calling configResolved with environment-specific defines. We must store
      // them separately to avoid the server's `document: "undefined"` leaking into
      // client transforms, or the client's `import.meta.server: false` into server ones.
      const isSsr = !!resolvedConfig.build?.ssr;
      const envDefine: Record<string, string> = {};
      if (resolvedConfig.define) {
        for (const [key, value] of Object.entries(resolvedConfig.define)) {
          if (isBuiltinDefine(key)) continue;
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

      let fileConfig = null;
      if (options.configMode !== false) {
        try {
          fileConfig = await loadConfig(state.root, {
            mode: options.configMode ?? "root",
            configFile: options.configFile,
            env: configEnv,
          });
          if (fileConfig) {
            state.logger.log("Loaded config from vize.config file");
            vizeConfigStore.set(state.root, fileConfig);
          }
        } catch (error) {
          state.logger.warn(
            `Failed to load vize config from ${options.configFile ?? state.root}:`,
            error,
          );
        }
      }

      const viteConfig = fileConfig?.vite ?? {};
      const compilerConfig = fileConfig?.compiler ?? {};

      state.mergedOptions = {
        ...options,
        ssr: options.ssr ?? compilerConfig.ssr ?? false,
        sourceMap: options.sourceMap ?? compilerConfig.sourceMap,
        vapor: options.vapor ?? compilerConfig.vapor ?? false,
        include: options.include ?? viteConfig.include,
        exclude: options.exclude ?? viteConfig.exclude,
        scanPatterns: options.scanPatterns ?? viteConfig.scanPatterns,
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
        if (typeof alias.find !== "string" || typeof alias.replacement !== "string") {
          continue;
        }
        state.cssAliasRules.push({ find: alias.find, replacement: alias.replacement });
      }
      // Prefer longer alias keys first
      state.cssAliasRules.sort((a, b) => b.find.length - a.find.length);

      state.filter = createFilter(state.mergedOptions.include, state.mergedOptions.exclude);
      state.scanPatterns = state.mergedOptions.scanPatterns ?? ["**/*.vue"];
      state.ignorePatterns = state.mergedOptions.ignorePatterns ?? [
        "node_modules/**",
        "dist/**",
        ".git/**",
      ];
      state.initialized = true;
    },

    configureServer(devServer: ViteDevServer) {
      state.server = devServer;

      // Rewrite __x00__ URLs from virtual module dynamic imports.
      // When compiled .vue files contain dynamic imports (e.g., template literal imports
      // for SVGs), the browser resolves them relative to the virtual module URL which
      // contains \0 (encoded as __x00__). Vite's plugin container short-circuits
      // resolveId for \0-prefixed IDs, so we intercept at the middleware level and
      // rewrite to /@fs/ so Vite serves the actual file.
      devServer.middlewares.use((req, _res, next) => {
        if (req.url && req.url.includes("__x00__")) {
          const [urlPath, queryPart] = req.url.split("?");
          // e.g., /@id/__x00__/Users/.../help.svg?import -> /@fs/Users/.../help.svg?import
          let cleanedPath = urlPath.replace(/__x00__/g, "");
          // After removing __x00__, /@id//Users/... has double slash -- normalize to /@fs/
          cleanedPath = cleanedPath.replace(/^\/@id\/\//, "/@fs/");

          // Do not rewrite vize virtual Vue modules (e.g. /@id/__x00__/.../App.vue.ts),
          // they must go through plugin load() and are not real files on disk.
          if (cleanedPath.startsWith("/@fs/")) {
            const fsPath = cleanedPath.slice(4); // strip '/@fs'
            if (
              fsPath.startsWith("/") &&
              fs.existsSync(fsPath) &&
              fs.statSync(fsPath).isFile() &&
              !fsPath.endsWith(".vue.ts")
            ) {
              const cleaned = queryPart ? `${cleanedPath}?${queryPart}` : cleanedPath;
              if (cleaned !== req.url) {
                state.logger.log(`middleware: rewriting ${req.url} -> ${cleaned}`);
                req.url = cleaned;
              }
            }
          }
        }
        next();
      });
    },

    async buildStart() {
      if (!state.scanPatterns) {
        // Running in standalone rolldown context (e.g., ox-content OG image)
        // where configResolved is not called. Skip pre-compilation.
        return;
      }
      await compileAll(state);
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

    generateBundle() {
      handleGenerateBundleHook(state, this.emitFile.bind(this));
    },
  };

  return [createVueCompatPlugin(state), mainPlugin, createPostTransformPlugin(state)];
}

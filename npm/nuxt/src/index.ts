import {
  addServerPlugin,
  addVitePlugin,
  createResolver,
  defineNuxtModule,
  getNuxtVersion,
  isNuxt2,
} from "@nuxt/kit";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";
import { createNuxtComponentResolver, injectNuxtComponentImports } from "./components";
import { injectNuxtI18nHelpers } from "./i18n";
import { appendMuseaArtComponentIgnore } from "./musea-components";
import type { VizeNuxtCompilerOptions, VizeNuxtOptions } from "./options";
import {
  resolveNuxtBridgeOptions,
  resolveNuxtCompilerOptions,
  resolveNuxtDevOptions,
  resolveNuxtMuseaOptions,
  resolveNuxtUnoCssOptions,
} from "./options";
import {
  buildNuxtDevAssetBase,
  isVizeGeneratedVueModuleId,
  isVizeJsxModuleId,
  isVizeVirtualVueModuleId,
  normalizeVizeVirtualVueModuleId,
  preserveExplicitVueImportsFromNuxtAutoImports,
  preserveExplicitVueImportsFromVizeModuleSource,
  stabilizeNuxtInjectedKeysForVizeVirtualModule,
} from "./utils";
import { appendOriginalVueSourceForUnoCss } from "./unocss";

type ViteTransformResult = string | { code?: string; map?: unknown } | null | undefined;
const VIZE_NUXT_AUTO_IMPORT_PATCHED = "__vizeNuxtAutoImportPatched";
const VUE_RUNTIME_DEDUPE = [
  "vue",
  "@vue/reactivity",
  "@vue/runtime-core",
  "@vue/runtime-dom",
  "@vue/shared",
];
const VUE_CLIENT_RUNTIME_IMPORT = "vue/dist/vue.runtime.esm-bundler.js";
type VitePluginWithTransform = {
  name?: string;
  transform?: unknown;
  [VIZE_NUXT_AUTO_IMPORT_PATCHED]?: boolean;
};
type NuxtWithBuilderOptions = {
  options: {
    app?: {
      baseURL?: string;
      buildAssetsDir?: string;
    };
    builder?: string;
    build?: {
      publicPath?: string;
    };
    router?: {
      base?: string;
    };
    vite?: { plugins?: unknown[]; resolve?: { dedupe?: string[] } };
    nitro?: { virtual?: Record<string, string> };
  };
};

function getDetectedNuxtMajor(nuxt: unknown): 2 | 3 | 4 | null {
  try {
    const version = getNuxtVersion(nuxt as never);
    const major = Number.parseInt(version.split(".")[0] ?? "", 10);
    return major === 2 || major === 3 || major === 4 ? major : null;
  } catch {
    return null;
  }
}

function hasNuxtViteCompilerSupport(nuxt: NuxtWithBuilderOptions): boolean {
  const builder = nuxt.options.builder;
  if (typeof builder === "string") {
    return builder === "vite" || builder.includes("vite-builder");
  }

  if (nuxt.options.vite) {
    return true;
  }

  try {
    return !isNuxt2(nuxt as never);
  } catch {
    return true;
  }
}

function getNuxtAppBaseURL(nuxt: NuxtWithBuilderOptions): string | undefined {
  return nuxt.options.app?.baseURL ?? nuxt.options.router?.base;
}

function getNuxtBuildAssetsDir(nuxt: NuxtWithBuilderOptions): string | undefined {
  return nuxt.options.app?.buildAssetsDir ?? nuxt.options.build?.publicPath;
}

function shouldUseVizeCompiler(
  compilerOptions: false | VizeNuxtCompilerOptions,
): compilerOptions is VizeNuxtCompilerOptions {
  return (
    compilerOptions !== false &&
    compilerOptions.compatibility?.hostCompiler !== true &&
    (compilerOptions.vueVersion ?? 3) === 3
  );
}

function dedupeVueRuntimePackages(vite: NonNullable<NuxtWithBuilderOptions["options"]["vite"]>) {
  vite.resolve ||= {};
  const dedupe = new Set(vite.resolve.dedupe ?? []);
  for (const packageName of VUE_RUNTIME_DEDUPE) {
    dedupe.add(packageName);
  }
  vite.resolve.dedupe = [...dedupe];
}

function isViteSsrTransform(args: unknown[]): boolean {
  const options = args[0];
  return (
    typeof options === "object" &&
    options !== null &&
    "ssr" in options &&
    (options as { ssr?: boolean }).ssr === true
  );
}

function rewriteBareVueImportsToClientRuntime(code: string): string {
  return code
    .replace(/(\bfrom\s*)(["'])vue\2/g, (_, prefix: string, quote: string) => {
      return `${prefix}${quote}${VUE_CLIENT_RUNTIME_IMPORT}${quote}`;
    })
    .replace(/(\bimport\s*)(["'])vue\2/g, (_, prefix: string, quote: string) => {
      return `${prefix}${quote}${VUE_CLIENT_RUNTIME_IMPORT}${quote}`;
    });
}

function normalizeNuxtKeyedTransformResult(
  id: string,
  result: ViteTransformResult,
): ViteTransformResult {
  if (!isVizeVirtualVueModuleId(id) || result == null) {
    return result;
  }
  if (typeof result === "string") {
    return normalizeNuxtInjectedKeysForVizeVirtualModule(result, id);
  }
  if (typeof result.code !== "string") {
    return result;
  }
  const code = normalizeNuxtInjectedKeysForVizeVirtualModule(result.code, id);
  return code === result.code ? result : { ...result, code };
}

function patchNuxtKeyedFunctionsPlugin(plugin: { transform?: unknown }): void {
  if (typeof plugin.transform === "function") {
    const original = plugin.transform;
    plugin.transform = async function (
      this: unknown,
      code: string,
      id: string,
      ...args: unknown[]
    ) {
      const result = (await original.call(this, code, id, ...args)) as ViteTransformResult;
      return normalizeNuxtKeyedTransformResult(id, result);
    };
    return;
  }

  const transform = plugin.transform as { handler?: unknown } | undefined;
  if (!transform || typeof transform.handler !== "function") {
    return;
  }

  const original = transform.handler;
  transform.handler = async function (this: unknown, code: string, id: string, ...args: unknown[]) {
    const result = (await original.call(this, code, id, ...args)) as ViteTransformResult;
    return normalizeNuxtKeyedTransformResult(id, result);
  };
}

function normalizeNuxtAutoImportTransformResult(
  code: string,
  id: string,
  result: ViteTransformResult,
  rewriteVueRuntimeImports: boolean,
): ViteTransformResult {
  if (!isVizeGeneratedVueModuleId(id) || result == null) {
    return result;
  }
  if (typeof result === "string") {
    const normalized = preserveExplicitVueImportsFromNuxtAutoImports(code, result);
    const restored = preserveExplicitVueImportsFromVizeModuleSource(id, normalized);
    return rewriteVueRuntimeImports ? rewriteBareVueImportsToClientRuntime(restored) : restored;
  }
  if (typeof result.code !== "string") {
    return result;
  }
  let normalized = preserveExplicitVueImportsFromVizeModuleSource(
    id,
    preserveExplicitVueImportsFromNuxtAutoImports(code, result.code),
  );
  if (rewriteVueRuntimeImports) {
    normalized = rewriteBareVueImportsToClientRuntime(normalized);
  }
  return normalized === result.code ? result : { ...result, code: normalized };
}

function patchNuxtAutoImportTransformPlugin(
  plugin: VitePluginWithTransform | undefined,
  isBuild: boolean,
): void {
  if (!plugin) {
    return;
  }
  if (plugin[VIZE_NUXT_AUTO_IMPORT_PATCHED]) {
    return;
  }

  if (typeof plugin.transform === "function") {
    const original = plugin.transform;
    plugin.transform = async function (
      this: unknown,
      code: string,
      id: string,
      ...args: unknown[]
    ) {
      const result = (await original.call(this, code, id, ...args)) as ViteTransformResult;
      return normalizeNuxtAutoImportTransformResult(
        code,
        id,
        result,
        isBuild && !isViteSsrTransform(args),
      );
    };
    plugin[VIZE_NUXT_AUTO_IMPORT_PATCHED] = true;
    return;
  }

  const transform = plugin.transform as { handler?: unknown } | undefined;
  if (!transform || typeof transform.handler !== "function") {
    return;
  }

  const original = transform.handler;
  transform.handler = async function (this: unknown, code: string, id: string, ...args: unknown[]) {
    const result = (await original.call(this, code, id, ...args)) as ViteTransformResult;
    return normalizeNuxtAutoImportTransformResult(
      code,
      id,
      result,
      isBuild && !isViteSsrTransform(args),
    );
  };
  plugin[VIZE_NUXT_AUTO_IMPORT_PATCHED] = true;
}

export default defineNuxtModule<VizeNuxtOptions>({
  meta: {
    name: "@vizejs/nuxt",
    configKey: "vize",
  },
  defaults: {
    musea: false,
    nuxtMusea: {
      route: { path: "/" },
    },
  },
  setup(options, nuxt) {
    const resolver = createResolver(import.meta.url);
    const detectedNuxtMajor = options.compatibility?.nuxtVersion ?? getDetectedNuxtMajor(nuxt) ?? 3;
    const vueVersion = options.compatibility?.vueVersion ?? (detectedNuxtMajor === 2 ? 2 : 3);
    const nuxtWithBuilderOptions = nuxt as NuxtWithBuilderOptions;
    const supportsViteCompiler = hasNuxtViteCompilerSupport(nuxtWithBuilderOptions);
    const appBaseURL = getNuxtAppBaseURL(nuxtWithBuilderOptions);
    const buildAssetsDir = getNuxtBuildAssetsDir(nuxtWithBuilderOptions);
    const bridgeOptions = resolveNuxtBridgeOptions(options.bridge);
    const devOptions = resolveNuxtDevOptions(options.dev);
    const museaOptions = resolveNuxtMuseaOptions(options.musea);
    const unocssOptions = resolveNuxtUnoCssOptions(options.unocss);

    if (museaOptions !== false) nuxt.hook("components:dirs", appendMuseaArtComponentIgnore);

    // Compiler
    const compilerOptions = resolveNuxtCompilerOptions(
      nuxt.options.rootDir,
      appBaseURL,
      buildAssetsDir,
      options.compiler,
      {
        supportsViteCompiler,
        vueVersion,
      },
    );
    const usesVizeCompiler = shouldUseVizeCompiler(compilerOptions);
    if (compilerOptions !== false) {
      nuxt.options.vite ||= {};
      nuxt.options.vite.plugins = nuxt.options.vite.plugins || [];
      nuxt.options.vite.plugins.push(vize(compilerOptions));
    }

    let isNuxtBuild = false;
    let isViteBuild = false;
    if (usesVizeCompiler) {
      nuxt.hook("build:before", () => {
        if (nuxt.options.dev !== false) {
          return;
        }
        isNuxtBuild = true;
        nuxt.options.vite ||= {};
        dedupeVueRuntimePackages(nuxt.options.vite);
      });
    }

    if (usesVizeCompiler) {
      if (nuxt.options.dev && devOptions.stylesheetLinks) {
        const devAssetBase =
          compilerOptions.devUrlBase ?? buildNuxtDevAssetBase(appBaseURL, buildAssetsDir);
        nuxt.options.nitro ||= {};
        nuxt.options.nitro.virtual ||= {};
        if (nuxt.options.nitro.virtual) {
          nuxt.options.nitro.virtual["#vizejs/nuxt/dev-stylesheet-links-config"] =
            `export const devAssetBase = ${JSON.stringify(devAssetBase)};`;
          addServerPlugin(resolver.resolve("./runtime/server/dev-stylesheet-links"));
        }
      }

      // Remove Nuxt's built-in @vitejs/plugin-vue when vize is active.
      // Both plugins handle .vue files; if both are active, @vitejs/plugin-vue
      // may try to read vize's \0-prefixed virtual module IDs via fs.readFileSync,
      // causing "path must not contain null bytes" / ENOENT errors.
      //
      // Nuxt adds @vitejs/plugin-vue AFTER vite:extendConfig but BEFORE
      // vite:configResolved. For the environment API path, the hook receives
      // a shallow copy of the config, so we must MUTATE the plugins array
      // in-place (splice) rather than replacing it (filter), so the change
      // propagates to the original config used by createServer().
      nuxt.hook(
        "vite:configResolved",
        (config: { command?: string; plugins: VitePluginWithTransform[] }) => {
          isViteBuild = config.command === "build" || isNuxtBuild || nuxt.options.dev === false;
          for (let i = config.plugins.length - 1; i >= 0; i--) {
            const p = config.plugins[i];
            const name = p && typeof p === "object" && "name" in p ? p.name : "";
            if (name === "vite:vue") {
              config.plugins.splice(i, 1);
            } else if (
              bridgeOptions.stableInjectedKeys &&
              name === "nuxt:compiler:keyed-functions"
            ) {
              patchNuxtKeyedFunctionsPlugin(p);
            }
            if (bridgeOptions.autoImports) {
              patchNuxtAutoImportTransformPlugin(p, isViteBuild);
            }
          }
        },
      );
    }

    // ─── Bridge: Apply Nuxt transforms to vize virtual modules ────────────
    // Nuxt's auto-import (unimport) and component loader (LoaderPlugin) use
    // unplugin-utils/createFilter which hard-excludes \0-prefixed module IDs.
    // Since vize uses \0-prefixed virtual IDs (Rollup convention), those
    // transforms never run on vize-compiled modules. This bridge plugin
    // fills the gap by applying the same transforms in a post-processing step.

    // Capture unimport context for composable auto-imports (useRoute, ref, computed, etc.)
    let unimportCtx: {
      injectImports: (
        code: string,
        id?: string,
      ) => Promise<{ code: string; s: unknown; imports: unknown[] }>;
    } | null = null;
    if (usesVizeCompiler && bridgeOptions.autoImports) {
      nuxt.hook("imports:context", (ctx: unknown) => {
        unimportCtx = ctx as typeof unimportCtx;
      });
    }

    const nuxtComponentResolver =
      usesVizeCompiler && bridgeOptions.components
        ? createNuxtComponentResolver({
            buildDir: nuxt.options.buildDir,
            moduleNames: nuxt.options.modules.filter(
              (moduleName): moduleName is string => typeof moduleName === "string",
            ),
            rootDir: nuxt.options.rootDir,
          })
        : null;

    // Capture component registry for component auto-imports (NuxtPage, NuxtLayout, etc.)
    if (nuxtComponentResolver) {
      nuxt.hook("components:extend", (comps: unknown) => {
        nuxtComponentResolver.register(
          comps as Array<{
            pascalName: string;
            kebabName: string;
            name: string;
            filePath: string;
            export: string;
            mode?: "client" | "server";
          }>,
        );
      });
    }

    const shouldAddNuxtTransformBridge =
      usesVizeCompiler &&
      (bridgeOptions.autoImports ||
        bridgeOptions.components ||
        bridgeOptions.i18n ||
        bridgeOptions.stableInjectedKeys);

    if (shouldAddNuxtTransformBridge) {
      addVitePlugin({
        name: "vizejs:nuxt-transform-bridge",
        enforce: "post" as const,
        async transform(code: string, id: string, ...args: unknown[]) {
          // Only process Vize-compiled component modules. In dev, Vite can call
          // transform hooks with the plugin-visible `.vue.ts?vue&vize` ID
          // rather than Rollup's internal `\0` virtual ID. Raw `.jsx`/`.tsx`
          // Vue components are compiled in place (no `.vue.ts` virtual id), so
          // they are matched separately and still receive Nuxt's auto-import,
          // component, and i18n bridging.
          if (!isVizeGeneratedVueModuleId(id) && !isVizeJsxModuleId(id)) return;

          let result = code;
          let changed = false;

          // 1. Component auto-imports: replace _resolveComponent("Name") with direct imports
          // Nuxt's LoaderPlugin normally does this, but skips \0-prefixed IDs.
          if (nuxtComponentResolver) {
            const nextComponentResult = injectNuxtComponentImports(result, (name) => {
              return nuxtComponentResolver.resolve(name);
            });
            if (nextComponentResult !== result) {
              result = nextComponentResult;
              changed = true;
            }
          }

          // 2. i18n function injection: inject useI18n() for $t, $rt, $d, $n, $tm, $te
          // @nuxtjs/i18n's TransformI18nFunctionPlugin skips \0-prefixed IDs.
          // Must inject inside the setup() function body, not at module top level.
          // Use negative lookbehind to exclude `_ctx.$t(` and `this.$t(` (property access),
          // which are Vue template globals and don't need useI18n injection.
          if (bridgeOptions.i18n) {
            const nextResult = injectNuxtI18nHelpers(result);
            if (nextResult !== result) {
              result = nextResult;
              changed = true;
            }
          }

          // 3. Composable auto-imports: inject useRoute, ref, computed, useI18n, etc.
          // Nuxt's unimport TransformPlugin normally does this, but skips \0-prefixed IDs.
          // Runs after i18n injection so unimport picks up the `useI18n` reference.
          if (unimportCtx) {
            try {
              const beforeUnimport = result;
              const injected = await unimportCtx.injectImports(result, id);
              if (injected.imports && injected.imports.length > 0) {
                result = preserveExplicitVueImportsFromNuxtAutoImports(
                  beforeUnimport,
                  injected.code,
                );
                changed = true;
              }
            } catch {
              // Ignore errors — auto-imports might not be needed for all modules
            }
          }

          if (bridgeOptions.autoImports) {
            const nextResult = preserveExplicitVueImportsFromVizeModuleSource(id, result);
            if (nextResult !== result) {
              result = nextResult;
              changed = true;
            }
          }

          if (bridgeOptions.stableInjectedKeys) {
            const stableKeyResult = stabilizeNuxtInjectedKeysForVizeVirtualModule(result, id);
            if (stableKeyResult !== result) {
              result = stableKeyResult;
              changed = true;
            }
          }

          if (isViteBuild && !isViteSsrTransform(args)) {
            const clientRuntimeResult = rewriteBareVueImportsToClientRuntime(result);
            if (clientRuntimeResult !== result) {
              result = clientRuntimeResult;
              changed = true;
            }
          }

          if (changed) {
            return { code: result, map: null };
          }
        },
      });
    }

    // ─── UnoCSS bridge: patch filter to accept vize virtual modules ────────
    // UnoCSS's Vite plugin uses createFilter from unplugin-utils which
    // hard-excludes \0-prefixed module IDs. Additionally, UnoCSS's pipeline
    // filter uses /\.(vue|...)($|\?)/ which rejects `.vue.ts` suffixes.
    //
    // Attributify support: UnoCSS's attributify extractor expects HTML-style
    // attributes (e.g. `flex="~ col gap1"`) but Vize compiles templates to
    // JS render functions where these become object properties (e.g.
    // `{ flex: "~ col gap1" }`). To support attributify, we also feed the
    // original .vue source to UnoCSS's extractor alongside the compiled JS.
    if (usesVizeCompiler && unocssOptions !== false) {
      addVitePlugin({
        name: "vizejs:unocss-bridge",
        configResolved(config: { plugins: Array<{ name: string; transform?: Function }> }) {
          for (const plugin of config.plugins) {
            if (plugin.name?.startsWith("unocss:") && typeof plugin.transform === "function") {
              const origTransform = plugin.transform;
              // Only enrich with original .vue source for the global mode plugin
              // (unocss:global:*) which does extraction only (returns null).
              // Other plugins like unocss:transformers modify the code and would
              // propagate the appended .vue source into the transform pipeline,
              // causing parse errors in downstream transforms (e.g. transformWithOxc).
              const isExtractionOnly = plugin.name.startsWith("unocss:global");
              plugin.transform = function (code: string, id: string, ...args: unknown[]) {
                if (isVizeVirtualVueModuleId(id)) {
                  // Strip \0 prefix AND .ts suffix so UnoCSS's filter accepts it.
                  // UnoCSS's defaultPipelineInclude is /\.(vue|...)($|\?)/ which
                  // requires .vue at end-of-string or before ?, not .vue.ts.
                  const normalizedId = normalizeVizeVirtualVueModuleId(id);

                  // For extraction-only plugins, append original .vue source so
                  // UnoCSS's attributify extractor can find HTML-style attribute
                  // patterns (flex="~ col gap1" etc.) that don't survive
                  // template-to-render-function compilation.
                  let effectiveCode = code;
                  if (isExtractionOnly && unocssOptions.originalSource !== false) {
                    effectiveCode = appendOriginalVueSourceForUnoCss(code, normalizedId, {
                      maxBytes: unocssOptions.originalSource.maxBytes,
                    });
                  }

                  return origTransform.call(this, effectiveCode, normalizedId, ...args);
                }
                return origTransform.call(this, code, id, ...args);
              };
            }
          }
        },
      });
    }

    // Musea gallery (without nuxtMusea mock layer)
    // In Nuxt context, real composables/components are already available
    // via Nuxt's own Vite plugins. Adding nuxtMusea globally would shadow
    // Nuxt's #imports resolution and break the app.
    if (museaOptions !== false && supportsViteCompiler) {
      const museaBasePath =
        "basePath" in museaOptions
          ? ((museaOptions as Record<string, unknown>).basePath as string)
          : "/__musea__";
      nuxt.options.vite ||= {};
      nuxt.options.vite.plugins = nuxt.options.vite.plugins || [];
      nuxt.options.vite.plugins.push(...musea(museaOptions));

      // Print Musea Gallery URL after dev server starts
      nuxt.hook("listen", (_server: unknown, listener: { url: string }) => {
        const url = listener.url?.replace(/\/$/, "") || "http://localhost:3000";
        console.log(
          `  \x1b[36m➜\x1b[0m  \x1b[1mMusea Gallery:\x1b[0m \x1b[36m${url}${museaBasePath}\x1b[0m`,
        );
      });
    }
  },
});

// Re-export types for convenience
export type { MuseaOptions } from "@vizejs/vite-plugin-musea";
export type {
  NuxtMuseaOptions,
  VizeNuxtBridgeOptions,
  VizeNuxtCompilerCompatibilityOptions,
  VizeNuxtCompatibilityOptions,
  VizeNuxtCompilerOptions,
  VizeNuxtDevOptions,
  VizeNuxtMajorVersion,
  VizeNuxtOptions,
  VizeNuxtUnoCssOptions,
  VizeNuxtVueVersion,
} from "./options";

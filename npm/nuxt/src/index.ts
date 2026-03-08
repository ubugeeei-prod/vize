/**
 * @vizejs/nuxt - Nuxt module for Vize
 *
 * Provides:
 * - Compiler: Vue SFC compilation via Vite plugin
 * - Musea: Component gallery with Nuxt mock support
 * - Linter: `vize lint` CLI command (via `vize` bin)
 * - Type Checker: `vize check` CLI command (via `vize` bin)
 */

import fs from "node:fs";
import { defineNuxtModule, addVitePlugin } from "@nuxt/kit";
import vize from "@vizejs/vite-plugin";
import { musea } from "@vizejs/vite-plugin-musea";
import type { MuseaOptions } from "@vizejs/vite-plugin-musea";
import type { NuxtMuseaOptions } from "@vizejs/musea-nuxt";
import { createNuxtComponentResolver, injectNuxtComponentImports } from "./components";
import { injectNuxtI18nHelpers } from "./i18n";

export interface VizeNuxtOptions {
  /**
   * Enable/disable the Vize compiler (Vue SFC compilation via Vite plugin).
   * Set to `false` to use Vue's default SFC compiler instead.
   * @default true
   */
  compiler?: boolean;

  /**
   * Musea gallery options.
   * Set to `false` to disable musea.
   */
  musea?: MuseaOptions | false;

  /**
   * Nuxt mock options for musea gallery.
   * NOTE: In Nuxt context, nuxtMusea mocks are NOT added as a global Vite plugin
   * because they would intercept `#imports` resolution and break Nuxt's internals.
   * Real Nuxt composables are available via Nuxt's own plugin pipeline.
   */
  nuxtMusea?: NuxtMuseaOptions;
}

export default defineNuxtModule<VizeNuxtOptions>({
  meta: {
    name: "@vizejs/nuxt",
    configKey: "vize",
  },
  defaults: {
    musea: {
      include: ["**/*.art.vue"],
      inlineArt: false,
    },
    nuxtMusea: {
      route: { path: "/" },
    },
  },
  setup(options, nuxt) {
    nuxt.options.vite.plugins = nuxt.options.vite.plugins || [];

    // Compiler
    if (options.compiler !== false) {
      nuxt.options.vite.plugins.push(vize());

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
      nuxt.hook("vite:configResolved", (config: { plugins: Array<{ name?: string }> }) => {
        for (let i = config.plugins.length - 1; i >= 0; i--) {
          const p = config.plugins[i];
          const name = p && typeof p === "object" && "name" in p ? p.name : "";
          if (name === "vite:vue") {
            config.plugins.splice(i, 1);
          }
        }
      });
    }

    // ─── Bridge: Apply Nuxt transforms to vize virtual modules ────────────
    // Nuxt's auto-import (unimport) and component loader (LoaderPlugin) use
    // unplugin-utils/createFilter which hard-excludes \0-prefixed module IDs.
    // Since vize uses \0-prefixed virtual IDs (Rollup convention), those
    // transforms never run on vize-compiled modules. This bridge plugin
    // fills the gap by applying the same transforms in a post-processing step.

    // Capture unimport context for composable auto-imports (useRoute, ref, computed, etc.)
    let unimportCtx: {
      injectImports: (code: string, id?: string) => Promise<{ code: string; s: unknown; imports: unknown[] }>;
    } | null = null;
    nuxt.hook("imports:context", (ctx: unknown) => {
      unimportCtx = ctx as typeof unimportCtx;
    });

    const nuxtComponentResolver = createNuxtComponentResolver({
      buildDir: nuxt.options.buildDir,
      moduleNames: nuxt.options.modules.filter((moduleName): moduleName is string => typeof moduleName === "string"),
      rootDir: nuxt.options.rootDir,
    });

    // Capture component registry for component auto-imports (NuxtPage, NuxtLayout, etc.)
    nuxt.hook("components:extend", (comps: unknown) => {
      nuxtComponentResolver.register(comps as Array<{
        pascalName: string;
        kebabName: string;
        name: string;
        filePath: string;
        export: string;
      }>);
    });

    addVitePlugin({
      name: "vizejs:nuxt-transform-bridge",
      enforce: "post" as const,
      async transform(code: string, id: string) {
        // Only process vize virtual modules
        if (!id.startsWith("\0") || !id.endsWith(".vue.ts")) return;

        let result = code;
        let changed = false;

        // 1. Component auto-imports: replace _resolveComponent("Name") with direct imports
        // Nuxt's LoaderPlugin normally does this, but skips \0-prefixed IDs.
        const nextComponentResult = injectNuxtComponentImports(result, (name) => {
          return nuxtComponentResolver.resolve(name);
        });
        if (nextComponentResult !== result) {
          result = nextComponentResult;
          changed = true;
        }

        // 2. i18n function injection: inject useI18n() for $t, $rt, $d, $n, $tm, $te
        // @nuxtjs/i18n's TransformI18nFunctionPlugin skips \0-prefixed IDs.
        // Must inject inside the setup() function body, not at module top level.
        // Use negative lookbehind to exclude `_ctx.$t(` and `this.$t(` (property access),
        // which are Vue template globals and don't need useI18n injection.
        const nextResult = injectNuxtI18nHelpers(result);
        if (nextResult !== result) {
          result = nextResult;
          changed = true;
        }

        // 3. Composable auto-imports: inject useRoute, ref, computed, useI18n, etc.
        // Nuxt's unimport TransformPlugin normally does this, but skips \0-prefixed IDs.
        // Runs after i18n injection so unimport picks up the `useI18n` reference.
        if (unimportCtx) {
          try {
            const injected = await unimportCtx.injectImports(result, id);
            if (injected.imports && injected.imports.length > 0) {
              result = injected.code;
              changed = true;
            }
          } catch {
            // Ignore errors — auto-imports might not be needed for all modules
          }
        }

        if (changed) {
          return { code: result, map: null };
        }
      },
    });

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
              if (id.startsWith("\0") && id.endsWith(".vue.ts")) {
                // Strip \0 prefix AND .ts suffix so UnoCSS's filter accepts it.
                // UnoCSS's defaultPipelineInclude is /\.(vue|...)($|\?)/ which
                // requires .vue at end-of-string or before ?, not .vue.ts.
                const normalizedId = id.slice(1).replace(/\.ts$/, "");

                // For extraction-only plugins, append original .vue source so
                // UnoCSS's attributify extractor can find HTML-style attribute
                // patterns (flex="~ col gap1" etc.) that don't survive
                // template-to-render-function compilation.
                let effectiveCode = code;
                if (isExtractionOnly) {
                  try {
                    const originalSource = fs.readFileSync(normalizedId, "utf-8");
                    effectiveCode = code + "\n" + originalSource;
                  } catch {
                    // File may not exist (virtual components, etc.)
                  }
                }

                return origTransform.call(this, effectiveCode, normalizedId, ...args);
              }
              return origTransform.call(this, code, id, ...args);
            };
          }
        }
      },
    });

    // Musea gallery (without nuxtMusea mock layer)
    // In Nuxt context, real composables/components are already available
    // via Nuxt's own Vite plugins. Adding nuxtMusea globally would shadow
    // Nuxt's #imports resolution and break the app.
    if (options.musea !== false) {
      const museaBasePath = options.musea && typeof options.musea === "object" && "basePath" in options.musea
        ? (options.musea as Record<string, unknown>).basePath as string
        : "/__musea__";
      nuxt.options.vite.plugins.push(...musea(options.musea || {}));

      // Print Musea Gallery URL after dev server starts
      nuxt.hook("listen", (_server: unknown, listener: { url: string }) => {
        const url = listener.url?.replace(/\/$/, "") || "http://localhost:3000";
        console.log(`  \x1b[36m➜\x1b[0m  \x1b[1mMusea Gallery:\x1b[0m \x1b[36m${url}${museaBasePath}\x1b[0m`);
      });
    }
  },
});

// Re-export types for convenience
export type { MuseaOptions } from "@vizejs/vite-plugin-musea";
export type { NuxtMuseaOptions } from "@vizejs/musea-nuxt";

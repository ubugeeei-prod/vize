/**
 * Main Musea Vite plugin implementation.
 *
 * Contains the `musea()` factory function that creates the Vite plugin,
 * including dev server middleware, config resolution, and build-start scanning.
 *
 * Virtual module hooks (resolveId / load / handleHotUpdate) are extracted into
 * `virtual.ts`.
 *
 * Middleware and API route logic is extracted into:
 * - `server-middleware.ts` -- gallery SPA, preview, art module serving
 * - `api-routes.ts` -- REST API endpoints for gallery UI
 */

import type { Plugin, ViteDevServer, ResolvedConfig } from "vite";
import fs from "node:fs";
import path from "node:path";
import { vizeConfigStore } from "@vizejs/vite-plugin";

import type { MuseaOptions, ArtFileInfo, ArtMetadata } from "../types/index.js";

import { loadNative } from "../native-loader.js";
import { extractScriptSetupContent } from "../art-module.js";
import { shouldProcess, scanArtFiles, generateStorybookFiles, buildThemeConfig } from "../utils.js";
import { registerMiddleware } from "../server-middleware.js";
import { createApiMiddleware } from "../api-routes/index.js";
import {
  createResolveId,
  createLoad,
  createHandleHotUpdate,
  type VirtualModuleState,
} from "./virtual.js";

function extractArtTagAttributes(source: string): Record<string, string | true> {
  const artTagMatch = source.match(/<art\b([\s\S]*?)>/i);
  if (!artTagMatch) return {};

  const attributes: Record<string, string | true> = {};
  const attrPattern = /([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'))?/g;

  for (const match of artTagMatch[1].matchAll(attrPattern)) {
    const name = match[1];
    if (!name || name === "/") continue;
    attributes[name] = match[2] ?? match[3] ?? true;
  }

  return attributes;
}

function parseActionEvents(value: string | true | undefined): string[] | undefined {
  if (typeof value !== "string") return undefined;

  const events = value
    .split(",")
    .map((eventName) => eventName.trim().toLowerCase())
    .filter(Boolean);

  return events.length > 0 ? [...new Set(events)] : undefined;
}

function extractCustomArtMetadata(source: string): Pick<ArtMetadata, "actionEvents"> {
  const attrs = extractArtTagAttributes(source);
  const actionEvents = new Set(parseActionEvents(attrs["action-events"]) ?? []);
  const captureMousemove = attrs["capture-mousemove"];

  if (captureMousemove === true || captureMousemove === "true") {
    actionEvents.add("mousemove");
  }

  return {
    actionEvents: actionEvents.size > 0 ? [...actionEvents] : undefined,
  };
}

/**
 * Create Musea Vite plugin.
 */
export function musea(options: MuseaOptions = {}): Plugin[] {
  let include = options.include ?? ["**/*.art.vue"];
  let exclude = options.exclude ?? ["node_modules/**", "dist/**"];
  let basePath = options.basePath ?? "/__musea__";
  let storybookCompat = options.storybookCompat ?? false;
  const storybookOutDir = options.storybookOutDir ?? ".storybook/stories";
  let inlineArt = options.inlineArt ?? false;
  const tokensPath = options.tokensPath;
  const themeConfig = buildThemeConfig(options.theme);
  const previewCss = options.previewCss ?? [];
  const previewSetup = options.previewSetup;

  let config: ResolvedConfig;
  let server: ViteDevServer | null = null;
  const artFiles = new Map<string, ArtFileInfo>();
  let resolvedPreviewCss: string[] = [];
  let resolvedPreviewSetup: string | null = null;

  // Shared state for virtual module hooks
  const virtualState: VirtualModuleState = {
    basePath,
    get inlineArt() {
      return inlineArt;
    },
    artFiles,
    resolvedPreviewCss,
    resolvedPreviewSetup,
    getConfigRoot: () => config.root,
    getServer: () => server,
    processArtFile,
  };

  // Create virtual module hooks
  const resolveId = createResolveId(virtualState);
  const load = createLoad(virtualState);
  const handleHotUpdate = createHandleHotUpdate(virtualState);

  // Main plugin
  const mainPlugin: Plugin = {
    name: "vite-plugin-musea",
    enforce: "pre",

    config() {
      // Add Vue alias for runtime template compilation
      // This is needed because variant templates are compiled at runtime
      return {
        resolve: {
          alias: {
            vue: "vue/dist/vue.esm-bundler.js",
          },
        },
      };
    },

    configResolved(resolvedConfig) {
      config = resolvedConfig;

      // Merge musea config from vize.config.ts (plugin args > config file > defaults)
      const vizeConfig = vizeConfigStore.get(resolvedConfig.root);
      if (vizeConfig?.musea) {
        const mc = vizeConfig.musea;
        // Only apply config file values when plugin options were not explicitly set
        if (!options.include && mc.include) include = mc.include;
        if (!options.exclude && mc.exclude) exclude = mc.exclude;
        if (!options.basePath && mc.basePath) basePath = mc.basePath;
        if (options.storybookCompat === undefined && mc.storybookCompat !== undefined)
          storybookCompat = mc.storybookCompat;
        if (options.inlineArt === undefined && mc.inlineArt !== undefined) inlineArt = mc.inlineArt;
      }

      // Update virtualState.basePath in case it changed from config resolution
      virtualState.basePath = basePath;

      // Resolve previewCss paths to absolute paths
      resolvedPreviewCss = previewCss.map((cssPath) =>
        path.isAbsolute(cssPath) ? cssPath : path.resolve(resolvedConfig.root, cssPath),
      );

      // Resolve previewSetup path
      if (previewSetup) {
        resolvedPreviewSetup = path.isAbsolute(previewSetup)
          ? previewSetup
          : path.resolve(resolvedConfig.root, previewSetup);
      }

      // Update shared state references after resolution
      virtualState.resolvedPreviewCss = resolvedPreviewCss;
      virtualState.resolvedPreviewSetup = resolvedPreviewSetup;
    },

    configureServer(devServer) {
      server = devServer;

      // Register gallery SPA, preview, and art module middleware
      registerMiddleware(devServer, {
        basePath,
        themeConfig,
        artFiles,
        resolvedPreviewCss,
        resolvedPreviewSetup,
      });

      // Register API endpoints
      devServer.middlewares.use(
        `${basePath}/api`,
        createApiMiddleware({
          config,
          artFiles,
          tokensPath,
          basePath,
          resolvedPreviewCss,
          resolvedPreviewSetup,
          processArtFile,
          getDevServerPort: () => devServer.config.server.port || 5173,
        }),
      );

      // Watch for Art file changes
      devServer.watcher.on("change", async (file) => {
        if (file.endsWith(".art.vue") && shouldProcess(file, include, exclude, config.root)) {
          await processArtFile(file);
          console.log(`[musea] Reloaded: ${path.relative(config.root, file)}`);
        }
        // Inline art: re-check .vue files on change
        if (inlineArt && file.endsWith(".vue") && !file.endsWith(".art.vue")) {
          const hadArt = artFiles.has(file);
          const source = await fs.promises.readFile(file, "utf-8");
          if (source.includes("<art")) {
            await processArtFile(file);
            console.log(`[musea] Reloaded inline art: ${path.relative(config.root, file)}`);
          } else if (hadArt) {
            artFiles.delete(file);
            console.log(`[musea] Removed inline art: ${path.relative(config.root, file)}`);
          }
        }
      });

      devServer.watcher.on("add", async (file) => {
        if (file.endsWith(".art.vue") && shouldProcess(file, include, exclude, config.root)) {
          await processArtFile(file);
          console.log(`[musea] Added: ${path.relative(config.root, file)}`);
        }
        // Inline art: check new .vue files
        if (inlineArt && file.endsWith(".vue") && !file.endsWith(".art.vue")) {
          const source = await fs.promises.readFile(file, "utf-8");
          if (source.includes("<art")) {
            await processArtFile(file);
            console.log(`[musea] Added inline art: ${path.relative(config.root, file)}`);
          }
        }
      });

      devServer.watcher.on("unlink", (file) => {
        if (artFiles.has(file)) {
          artFiles.delete(file);
          console.log(`[musea] Removed: ${path.relative(config.root, file)}`);
        }
      });

      // Print Musea gallery URL after server starts
      return () => {
        devServer.httpServer?.once("listening", () => {
          const address = devServer.httpServer?.address();
          if (address && typeof address === "object") {
            const protocol = devServer.config.server.https ? "https" : "http";
            const rawHost = address.address;
            // Normalize IPv6/IPv4 localhost addresses to "localhost"
            const host =
              rawHost === "::" ||
              rawHost === "::1" ||
              rawHost === "0.0.0.0" ||
              rawHost === "127.0.0.1"
                ? "localhost"
                : rawHost;
            const port = address.port;
            const url = `${protocol}://${host}:${port}${basePath}`;

            console.log();
            console.log(`  \x1b[36m➜\x1b[0m  \x1b[1mMusea Gallery:\x1b[0m \x1b[36m${url}\x1b[0m`);
          }
        });
      };
    },

    async buildStart() {
      // Scan for Art files
      console.log(`[musea] config.root: ${config.root}, include: ${JSON.stringify(include)}`);
      const files = await scanArtFiles(config.root, include, exclude, inlineArt);

      console.log(`[musea] Found ${files.length} art files`);

      for (const file of files) {
        await processArtFile(file);
      }

      // Generate Storybook CSF if enabled
      if (storybookCompat) {
        await generateStorybookFiles(artFiles, config.root, storybookOutDir);
      }
    },

    resolveId,
    load,
    handleHotUpdate,
  };

  // Helper functions scoped to this plugin instance

  async function processArtFile(filePath: string): Promise<void> {
    try {
      const source = await fs.promises.readFile(filePath, "utf-8");
      const binding = loadNative();
      const parsed = binding.parseArt(source, { filename: filePath });
      const customMetadata = extractCustomArtMetadata(source);

      // Skip files with no variants (e.g. .vue files without <art> block)
      if (!parsed.variants || parsed.variants.length === 0) return;

      const isInline = !filePath.endsWith(".art.vue");

      const info: ArtFileInfo = {
        path: filePath,
        metadata: {
          title: parsed.metadata.title || (isInline ? path.basename(filePath, ".vue") : ""),
          description: parsed.metadata.description,
          component: isInline ? undefined : parsed.metadata.component,
          category: parsed.metadata.category,
          tags: parsed.metadata.tags,
          status: parsed.metadata.status as "draft" | "ready" | "deprecated",
          order: parsed.metadata.order,
          actionEvents: customMetadata.actionEvents ?? parsed.metadata.actionEvents,
        },
        variants: parsed.variants.map((v) => ({
          name: v.name,
          template: v.template,
          isDefault: v.isDefault,
          skipVrt: v.skipVrt,
        })),
        hasScriptSetup: isInline ? false : parsed.hasScriptSetup,
        scriptSetupContent:
          !isInline && parsed.hasScriptSetup ? extractScriptSetupContent(source) : undefined,
        hasScript: parsed.hasScript,
        styleCount: parsed.styleCount,
        isInline,
        componentPath: isInline ? filePath : undefined,
      };

      artFiles.set(filePath, info);
    } catch (e) {
      console.error(`[musea] Failed to process ${filePath}:`, e);
    }
  }

  return [mainPlugin];
}

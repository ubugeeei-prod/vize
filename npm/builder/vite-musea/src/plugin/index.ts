import type { Plugin, ViteDevServer, ResolvedConfig } from "vite";
import { transformWithEsbuild } from "vite";
import fs from "node:fs";
import path from "node:path";
import type { MuseaOptions, ArtFileInfo } from "../types/index.js";

import {
  shouldProcess,
  scanArtFiles,
  generateStorybookFiles,
  buildThemeConfig,
  resolveScanRoots,
} from "../utils.js";
import { registerMiddleware } from "../server-middleware.js";
import { createApiMiddleware } from "../api-routes/index.js";
import { createDevSessionToken } from "../security.js";
import {
  createResolveId,
  createLoad,
  createHandleHotUpdate,
  type VirtualModuleState,
} from "./virtual.js";
import { shouldApplyMuseaPlugin } from "./apply.js";
import { watchMuseaArtFiles } from "./watch.js";
import { createVueRuntimeCompilerAlias } from "./vue-alias.js";
import {
  applyMuseaStaticBuildInput,
  emitStaticGallery,
  isMuseaStaticBuild,
  loadStaticRuntimeModule,
  museaStaticBuildConfig,
  resolveStaticRuntimeId,
  shouldEnableMuseaStaticBuild,
  type StaticBuildInput,
} from "../static-export.js";
import { resolveMuseaSharedConfig } from "./config.js";
import { processMuseaArtFile } from "./art-processing.js";

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
  const devSessionToken = createDevSessionToken();

  let config: ResolvedConfig;
  let server: ViteDevServer | null = null;
  const artFiles = new Map<string, ArtFileInfo>();
  let resolvedPreviewCss: string[] = [];
  let resolvedPreviewSetup: string | null = null;
  let scanRoots: string[] = [];
  let staticBuildEnabled = isMuseaStaticBuild();

  const virtualState: VirtualModuleState = {
    basePath,
    get inlineArt() {
      return inlineArt;
    },
    artFiles,
    resolvedPreviewCss,
    resolvedPreviewSetup,
    getConfigRoot: () => config.root,
    getScanRoots: () => scanRoots,
    getServer: () => server,
    processArtFile,
  };

  const virtualResolveId = createResolveId(virtualState);
  const virtualLoad = createLoad(virtualState);
  const handleHotUpdate = createHandleHotUpdate(virtualState);

  const mainPlugin: Plugin = {
    name: "vite-plugin-musea",
    enforce: "pre",
    apply(_config, env) {
      return shouldApplyMuseaPlugin(env);
    },

    config(userConfig, env) {
      staticBuildEnabled = shouldEnableMuseaStaticBuild(env.command);
      const staticBuildConfig = staticBuildEnabled
        ? museaStaticBuildConfig(userConfig.build?.rollupOptions?.input as StaticBuildInput)
        : {};
      return { resolve: { alias: [createVueRuntimeCompilerAlias()] }, ...staticBuildConfig };
    },

    options: applyMuseaStaticBuildInput,
    async configResolved(resolvedConfig) {
      config = resolvedConfig;

      const vizeConfig = await resolveMuseaSharedConfig(resolvedConfig);
      if (vizeConfig?.musea) {
        const mc = vizeConfig.musea;
        if (!options.include && mc.include) include = mc.include;
        if (!options.exclude && mc.exclude) exclude = mc.exclude;
        if (!options.basePath && mc.basePath) basePath = mc.basePath;
        if (options.storybookCompat === undefined && mc.storybookCompat !== undefined)
          storybookCompat = mc.storybookCompat;
        if (options.inlineArt === undefined && mc.inlineArt !== undefined) inlineArt = mc.inlineArt;
      }

      virtualState.basePath = basePath;

      resolvedPreviewCss = previewCss.map((cssPath) =>
        path.isAbsolute(cssPath) ? cssPath : path.resolve(resolvedConfig.root, cssPath),
      );

      if (previewSetup) {
        resolvedPreviewSetup = path.isAbsolute(previewSetup)
          ? previewSetup
          : path.resolve(resolvedConfig.root, previewSetup);
      }

      virtualState.resolvedPreviewCss = resolvedPreviewCss;
      virtualState.resolvedPreviewSetup = resolvedPreviewSetup;
      scanRoots = resolveScanRoots(resolvedConfig.root, include);
    },

    configureServer(devServer) {
      server = devServer;

      registerMiddleware(devServer, {
        basePath,
        devSessionToken,
        themeConfig,
        artFiles,
        scanRoots,
        resolvedPreviewCss,
        resolvedPreviewSetup,
      });

      devServer.middlewares.use(
        `${basePath}/api`,
        createApiMiddleware({
          config,
          artFiles,
          scanRoots,
          tokensPath,
          basePath,
          resolvedPreviewCss,
          resolvedPreviewSetup,
          devSessionToken,
          processArtFile,
          getDevServerPort: () => devServer.config.server.port || 5173,
        }),
      );

      devServer.watcher.on("change", async (file) => {
        if (file.endsWith(".art.vue") && shouldProcess(file, include, exclude, config.root)) {
          await processArtFile(file);
          console.log(`[musea] Reloaded: ${path.relative(config.root, file)}`);
        }
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

      return () => {
        devServer.httpServer?.once("listening", () => {
          const address = devServer.httpServer?.address();
          if (address && typeof address === "object") {
            const protocol = devServer.config.server.https ? "https" : "http";
            const rawHost = address.address;
            const host = ["::", "::1", "0.0.0.0", "127.0.0.1"].includes(rawHost)
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
      console.log(`[musea] config.root: ${config.root}, include: ${JSON.stringify(include)}`);
      const files = await scanArtFiles(config.root, include, exclude, inlineArt);

      console.log(`[musea] Found ${files.length} art files`);

      if (server) {
        watchMuseaArtFiles(server.watcher, files);
      }

      for (const file of files) {
        await processArtFile(file);
      }

      if (storybookCompat) {
        await generateStorybookFiles(artFiles, config.root, storybookOutDir);
      }
    },

    resolveId(id) {
      return resolveStaticRuntimeId(id) ?? virtualResolveId(id);
    },
    load(id) {
      return loadStaticRuntimeModule(id, artFiles) ?? virtualLoad(id);
    },
    async generateBundle(_options, bundle) {
      if (!staticBuildEnabled) return;
      await emitStaticGallery((asset) => void this.emitFile(asset), bundle, {
        config,
        artFiles,
        scanRoots,
        tokensPath,
        basePath,
        resolvedPreviewCss,
        resolvedPreviewSetup,
        devSessionToken,
        themeConfig,
      });
    },
    async transform(code, id) {
      if (!id.includes("?musea-virtual")) {
        return null;
      }

      if (!id.includes("musea-art:") && !id.includes("\0musea:")) {
        return null;
      }

      const safeId = id
        .replaceAll("\0", "")
        .replace(/[^\w./-]+/g, "_")
        .replace(/_+/g, "_");
      const loaderId = path.join(config.root, `.musea-${safeId}.ts`);

      return transformWithEsbuild(code, loaderId, {
        loader: "ts",
        format: "esm",
        sourcemap: config.command === "serve",
        target: "esnext",
      });
    },
    handleHotUpdate,
  };

  async function processArtFile(filePath: string): Promise<void> {
    const info = await processMuseaArtFile(filePath, {
      root: config.root,
      command: config.command,
    });
    if (info) artFiles.set(filePath, info);
  }

  return [mainPlugin];
}

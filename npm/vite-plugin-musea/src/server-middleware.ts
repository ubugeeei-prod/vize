/**
 * Musea dev server middleware handlers.
 *
 * Extracted from the main plugin to keep file sizes manageable.
 * Provides middleware for the gallery SPA, static assets, preview rendering,
 * and art module serving.
 */

import type { ViteDevServer } from "vite";
import { createRequire } from "node:module";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import type { ArtFileInfo } from "./types/index.js";
import { generateGalleryHtml } from "./gallery/index.js";
import { generatePreviewModule, generatePreviewHtml } from "./preview/index.js";
import { generateArtModule } from "./art-module.js";
import { toPascalCase } from "./utils.js";

const moduleDir = path.dirname(fileURLToPath(import.meta.url));

function resolveGalleryDistDir(): string {
  return path.resolve(moduleDir, "gallery");
}

function resolveGallerySourceDir(): string {
  return path.resolve(moduleDir, "../gallery");
}

function toViteFsPath(filePath: string): string {
  return encodeURI(`/@fs${filePath.split(path.sep).join("/")}`);
}

async function tryLoadSourceGalleryHtml(
  devServer: ViteDevServer,
  url: string,
  basePath: string,
  themeConfig?: { default: string; custom?: Record<string, unknown> },
): Promise<string | null> {
  const gallerySourceDir = resolveGallerySourceDir();
  const indexHtmlPath = path.join(gallerySourceDir, "index.html");

  try {
    await fs.promises.access(indexHtmlPath);
  } catch {
    return null;
  }

  const sourceEntryPath = toViteFsPath(path.join(gallerySourceDir, "main.ts"));
  const themeScript = themeConfig
    ? `window.__MUSEA_THEME_CONFIG__=${JSON.stringify(themeConfig)};`
    : "";

  let html = await fs.promises.readFile(indexHtmlPath, "utf-8");
  html = html.replace('src="./main.ts"', `src="${sourceEntryPath}"`);
  html = html.replace(
    "</head>",
    `<script>window.__MUSEA_BASE_PATH__='${basePath}';${themeScript}</script></head>`,
  );

  return devServer.transformIndexHtml(url, html);
}

/** Dependencies injected from the plugin closure. */
export interface MiddlewareContext {
  basePath: string;
  themeConfig: { default: string; custom?: Record<string, unknown> } | undefined;
  artFiles: Map<string, ArtFileInfo>;
  resolvedPreviewCss: string[];
  resolvedPreviewSetup: string | null;
}

/**
 * Register all Musea middleware on the given dev server.
 *
 * This sets up:
 * - Gallery SPA route (serves built SPA or inline HTML fallback)
 * - Gallery static assets (/assets/)
 * - axe-core vendor script
 * - Preview module route
 * - VRT preview route
 * - Art module route
 */
export function registerMiddleware(devServer: ViteDevServer, ctx: MiddlewareContext): void {
  const { basePath, themeConfig, artFiles } = ctx;

  // --- Gallery SPA route ---
  devServer.middlewares.use(basePath, async (req, res, next) => {
    const url = req.url || "/";

    if (
      url === "/" ||
      url === "/index.html" ||
      url.startsWith("/tokens") ||
      url.startsWith("/component/") ||
      url.startsWith("/tests")
    ) {
      const galleryDistDir = resolveGalleryDistDir();
      const indexHtmlPath = path.join(galleryDistDir, "index.html");

      try {
        await fs.promises.access(indexHtmlPath);
        let html = await fs.promises.readFile(indexHtmlPath, "utf-8");
        const themeScript = themeConfig
          ? `window.__MUSEA_THEME_CONFIG__=${JSON.stringify(themeConfig)};`
          : "";
        html = html.replace(
          "</head>",
          `<script>window.__MUSEA_BASE_PATH__='${basePath}';${themeScript}</script></head>`,
        );
        res.setHeader("Content-Type", "text/html");
        res.end(html);
        return;
      } catch {
        const sourceHtml = await tryLoadSourceGalleryHtml(devServer, url, basePath, themeConfig);
        if (sourceHtml) {
          res.setHeader("Content-Type", "text/html");
          res.end(sourceHtml);
          return;
        }

        const html = generateGalleryHtml(basePath, themeConfig);
        res.setHeader("Content-Type", "text/html");
        res.end(html);
        return;
      }
    }

    // Serve gallery static assets (JS, CSS) from built SPA
    if (url.startsWith("/assets/")) {
      const galleryDistDir = resolveGalleryDistDir();
      const filePath = path.join(galleryDistDir, url);
      try {
        const stat = await fs.promises.stat(filePath);
        if (stat.isFile()) {
          const content = await fs.promises.readFile(filePath);
          const ext = path.extname(filePath);
          const mimeTypes: Record<string, string> = {
            ".js": "application/javascript",
            ".css": "text/css",
            ".svg": "image/svg+xml",
            ".png": "image/png",
            ".ico": "image/x-icon",
            ".woff2": "font/woff2",
            ".woff": "font/woff",
          };
          res.setHeader("Content-Type", mimeTypes[ext] || "application/octet-stream");
          res.setHeader("Cache-Control", "public, max-age=31536000, immutable");
          res.end(content);
          return;
        }
      } catch {
        // File not found, fall through
      }
    }

    next();
  });

  // --- axe-core vendor script ---
  devServer.middlewares.use(`${basePath}/vendor/axe-core.min.js`, async (_req, res, _next) => {
    try {
      const require = createRequire(import.meta.url);
      const axeCorePath = require.resolve("axe-core/axe.min.js");
      const content = await fs.promises.readFile(axeCorePath, "utf-8");
      res.setHeader("Content-Type", "application/javascript");
      res.setHeader("Cache-Control", "public, max-age=86400");
      res.end(content);
    } catch {
      res.statusCode = 404;
      res.end("axe-core not installed");
    }
  });

  // --- Preview module route ---
  devServer.middlewares.use(`${basePath}/preview-module`, async (req, res, _next) => {
    const url = new URL(req.url || "", `http://localhost`);
    const artPath = url.searchParams.get("art");
    const variantName = url.searchParams.get("variant");

    if (!artPath || !variantName) {
      res.statusCode = 400;
      res.end("Missing art or variant parameter");
      return;
    }

    const art = artFiles.get(artPath);
    if (!art) {
      res.statusCode = 404;
      res.end("Art not found");
      return;
    }

    const variant = art.variants.find((v) => v.name === variantName);
    if (!variant) {
      res.statusCode = 404;
      res.end("Variant not found");
      return;
    }

    const variantComponentName = toPascalCase(variant.name);
    const moduleCode = generatePreviewModule(
      art,
      variantComponentName,
      variant.name,
      ctx.resolvedPreviewCss,
      ctx.resolvedPreviewSetup,
    );

    try {
      const result = await devServer.transformRequest(
        `virtual:musea-preview:${artPath}:${variantName}`,
      );
      if (result) {
        res.setHeader("Content-Type", "application/javascript");
        res.setHeader("Cache-Control", "no-cache");
        res.end(result.code);
        return;
      }
    } catch {
      // Fall through to manual response
    }

    res.setHeader("Content-Type", "application/javascript");
    res.setHeader("Cache-Control", "no-cache");
    res.end(moduleCode);
  });

  // --- VRT preview route ---
  devServer.middlewares.use(`${basePath}/preview`, async (req, res, _next) => {
    const url = new URL(req.url || "", `http://localhost`);
    const artPath = url.searchParams.get("art");
    const variantName = url.searchParams.get("variant");

    if (!artPath || !variantName) {
      res.statusCode = 400;
      res.end("Missing art or variant parameter");
      return;
    }

    const art = artFiles.get(artPath);
    if (!art) {
      res.statusCode = 404;
      res.end("Art not found");
      return;
    }

    const variant = art.variants.find((v) => v.name === variantName);
    if (!variant) {
      res.statusCode = 404;
      res.end("Variant not found");
      return;
    }

    const config = devServer.config;
    const html = generatePreviewHtml(art, variant, basePath, config.base);
    res.setHeader("Content-Type", "text/html");
    res.end(html);
  });

  // --- Art module route ---
  devServer.middlewares.use(`${basePath}/art`, async (req, res, next) => {
    const url = new URL(req.url || "", "http://localhost");
    const artPath = decodeURIComponent(url.pathname.slice(1));

    if (!artPath) {
      next();
      return;
    }

    const art = artFiles.get(artPath);
    if (!art) {
      res.statusCode = 404;
      res.end("Art not found: " + artPath);
      return;
    }

    try {
      const virtualId = `virtual:musea-art:${artPath}`;
      const result = await devServer.transformRequest(virtualId);
      if (result) {
        res.setHeader("Content-Type", "application/javascript");
        res.setHeader("Cache-Control", "no-cache");
        res.end(result.code);
      } else {
        const moduleCode = generateArtModule(art, artPath);
        res.setHeader("Content-Type", "application/javascript");
        res.end(moduleCode);
      }
    } catch (err) {
      console.error("[musea] Failed to transform art module:", err);
      const moduleCode = generateArtModule(art, artPath);
      res.setHeader("Content-Type", "application/javascript");
      res.end(moduleCode);
    }
  });
}

import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { generateGalleryBody, generateGalleryScript } from "./gallery/template.js";
import { serializeScriptValue } from "./security.js";
import {
  createStaticGalleryPayload,
  joinUrlPath,
  staticPreviewId,
  type StaticGalleryDataContext,
  type StaticGalleryPayload,
} from "./static-data.js";
import type { ArtFileInfo } from "./types/index.js";
import { escapeHtml } from "./utils.js";

export const MUSEA_STATIC_BUILD_ENV = "VIZE_MUSEA_STATIC_BUILD";
export const VIRTUAL_STATIC_RUNTIME = "virtual:musea-static-runtime";

const RESOLVED_STATIC_RUNTIME = "\0musea-static-runtime";
const STATIC_RUNTIME_INPUT_NAME = "musea-static-runtime";
const STATIC_USER_INPUT_NAME = "musea-static-entry";
const moduleDir = path.dirname(fileURLToPath(import.meta.url));

export type StaticBuildInput = string | readonly string[] | Record<string, string> | undefined;
type OutputBundle = Record<string, OutputChunk | { type: string; fileName: string }>;
type OutputChunk = {
  type: "chunk";
  name: string;
  fileName: string;
  facadeModuleId: string | null;
};

export interface StaticEmitContext extends StaticGalleryDataContext {
  themeConfig: { default: string; custom?: Record<string, unknown> } | undefined;
}

export function isMuseaStaticBuild(): boolean {
  return process.env[MUSEA_STATIC_BUILD_ENV] === "1";
}

export function museaStaticBuildInput(input: StaticBuildInput): Record<string, string> {
  const entries: Record<string, string> = {};

  if (typeof input === "string") {
    entries[STATIC_USER_INPUT_NAME] = input;
  } else if (Array.isArray(input)) {
    input.forEach((value, index) => {
      entries[`${STATIC_USER_INPUT_NAME}-${index + 1}`] = value;
    });
  } else if (input) {
    Object.assign(entries, input);
  }

  entries[STATIC_RUNTIME_INPUT_NAME] = VIRTUAL_STATIC_RUNTIME;
  return entries;
}

export function applyMuseaStaticBuildInput(options: { input?: unknown }): null {
  if (!isMuseaStaticBuild()) return null;
  options.input = museaStaticBuildInput(options.input as StaticBuildInput);
  return null;
}

export function museaStaticBuildConfig(input?: StaticBuildInput): {
  build: { rollupOptions: { input: Record<string, string> } };
} {
  return {
    build: {
      rollupOptions: {
        input: museaStaticBuildInput(input),
      },
    },
  };
}

export function resolveStaticRuntimeId(id: string): string | null {
  return id === VIRTUAL_STATIC_RUNTIME ? RESOLVED_STATIC_RUNTIME : null;
}

export function loadStaticRuntimeModule(
  id: string,
  artFiles: Map<string, ArtFileInfo>,
): string | null {
  if (id !== RESOLVED_STATIC_RUNTIME) return null;

  const entries = Array.from(artFiles.values()).flatMap((art) =>
    art.variants.map((variant) => {
      const key = staticPreviewId(art.path, variant.name);
      const moduleId = `virtual:musea-preview:${art.path}:${variant.name}`;
      return `${JSON.stringify(key)}: () => import(${JSON.stringify(moduleId)})`;
    }),
  );

  return `
const loaders = { ${entries.join(",\n")} };

export async function loadMuseaPreview(id) {
  const load = loaders[id];
  if (!load) throw new Error("Musea preview not found: " + id);
  await load();
}

if (typeof window !== "undefined") {
  window.__MUSEA_LOAD_PREVIEW__ = loadMuseaPreview;
}
`;
}

export async function emitStaticGallery(
  emitFile: (asset: { type: "asset"; fileName: string; source: string | Uint8Array }) => void,
  bundle: OutputBundle,
  ctx: StaticEmitContext,
): Promise<void> {
  const runtimeFileName = findRuntimeFileName(bundle);
  if (!runtimeFileName) {
    throw new Error("musea static build could not find its generated runtime entry");
  }

  const payload = await createStaticGalleryPayload(ctx);
  const staticRoot = staticRootFromBasePath(ctx.basePath);
  await emitGalleryShell(emitFile, staticRoot, ctx, payload);
  emitFile({
    type: "asset",
    fileName: joinFileName(staticRoot, "api", "static.json"),
    source: JSON.stringify(payload, null, 2),
  });
  emitFile({
    type: "asset",
    fileName: joinFileName(staticRoot, "api", "arts"),
    source: JSON.stringify(payload.arts, null, 2),
  });
  await emitAxeVendor(emitFile, staticRoot);
  emitPreviewHtml(emitFile, staticRoot, runtimeFileName, ctx.basePath, payload, ctx.artFiles);
  emitRootRedirect(emitFile, staticRoot, ctx.basePath);
}

function findRuntimeFileName(bundle: OutputBundle): string | null {
  for (const item of Object.values(bundle)) {
    if (item.type === "chunk" && item.name === "musea-static-runtime") {
      return item.fileName;
    }
  }
  return null;
}

async function emitGalleryShell(
  emitFile: (asset: { type: "asset"; fileName: string; source: string | Uint8Array }) => void,
  staticRoot: string,
  ctx: StaticEmitContext,
  payload: StaticGalleryPayload,
): Promise<void> {
  const galleryDir = resolveGalleryDistDir();
  if (!galleryDir) {
    const html = injectStaticGlobals(generateStaticFallbackGalleryHtml(ctx.basePath), ctx, payload);
    emitFile({ type: "asset", fileName: joinFileName(staticRoot, "index.html"), source: html });
    return;
  }

  for (const filePath of await collectFiles(galleryDir)) {
    const relative = path.relative(galleryDir, filePath).split(path.sep).join("/");
    const target = joinFileName(staticRoot, relative);
    const content = await fs.promises.readFile(filePath);
    if (relative === "index.html") {
      const html = injectStaticGlobals(content.toString("utf-8"), ctx, payload);
      emitFile({ type: "asset", fileName: target, source: rewriteGalleryBase(html, ctx.basePath) });
    } else {
      emitFile({ type: "asset", fileName: target, source: content });
    }
  }

  if (payload.arts.length === 0) {
    return;
  }
}

function generateStaticFallbackGalleryHtml(basePath: string): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Musea - Component Gallery</title>
  <style>html,body{min-height:100%;margin:0}body{font-family:system-ui,sans-serif}</style>
</head>
<body>${generateGalleryBody(basePath)}

  <script type="module">${generateGalleryScript(basePath)}
  </script>
</body>
</html>`;
}

function emitPreviewHtml(
  emitFile: (asset: { type: "asset"; fileName: string; source: string }) => void,
  staticRoot: string,
  runtimeFileName: string,
  basePath: string,
  payload: StaticGalleryPayload,
  artFiles: Map<string, ArtFileInfo>,
): void {
  const previewDir = joinFileName(staticRoot, "preview");
  const runtimeUrl = relativeUrl(previewDir, runtimeFileName);

  for (const art of artFiles.values()) {
    for (const variant of art.variants) {
      const id = staticPreviewId(art.path, variant.name);
      const html = generateStaticPreviewHtml(art, variant.name, id, basePath, runtimeUrl);
      emitFile({ type: "asset", fileName: joinFileName(previewDir, `${id}.html`), source: html });
    }
  }

  void payload;
}

function generateStaticPreviewHtml(
  art: ArtFileInfo,
  variantName: string,
  previewId: string,
  basePath: string,
  runtimeUrl: string,
): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(art.metadata.title)} - ${escapeHtml(variantName)}</title>
  <script>window.__MUSEA_BASE_PATH__=${serializeScriptValue(basePath)};</script>
  <style>html,body{width:100%;height:100%;margin:0}body{font-family:system-ui,sans-serif;background:#fff;padding:1rem;overflow:auto;box-sizing:border-box}.musea-variant{min-height:calc(100vh - 2rem)}.musea-error{color:#dc2626;background:#fef2f2;border:1px solid #fecaca;border-radius:8px;padding:1rem;font-size:.875rem}.musea-loading{color:#6b7280;font-size:.875rem}.musea-bg-checkerboard{background-image:linear-gradient(45deg,#ccc 25%,transparent 25%),linear-gradient(-45deg,#ccc 25%,transparent 25%),linear-gradient(45deg,transparent 75%,#ccc 75%),linear-gradient(-45deg,transparent 75%,#ccc 75%)!important;background-size:20px 20px!important;background-position:0 0,0 10px,10px -10px,-10px 0!important}</style>
</head>
<body>
  <div id="app" class="musea-variant" data-art="${escapeHtml(art.path)}" data-variant="${escapeHtml(variantName)}">
    <div class="musea-loading">Loading component...</div>
  </div>
  <script type="module">
    import ${JSON.stringify(runtimeUrl)};
    Promise.resolve().then(() => {
      const loadMuseaPreview = window.__MUSEA_LOAD_PREVIEW__;
      if (typeof loadMuseaPreview !== "function") {
        throw new Error("Musea preview runtime failed to load");
      }
      return loadMuseaPreview(${JSON.stringify(previewId)});
    }).catch((error) => {
      const el = document.getElementById("app");
      if (el) el.textContent = error instanceof Error ? error.message : String(error);
    });
  </script>
</body>
</html>`;
}

function injectStaticGlobals(
  html: string,
  ctx: StaticEmitContext,
  payload: StaticGalleryPayload,
): string {
  const themeScript = ctx.themeConfig
    ? `window.__MUSEA_THEME_CONFIG__=${serializeScriptValue(ctx.themeConfig)};`
    : "";
  const script = `<script>window.__MUSEA_BASE_PATH__=${serializeScriptValue(ctx.basePath)};window.__MUSEA_STATIC__=true;window.__MUSEA_STATIC_PREVIEWS__=${serializeScriptValue(payload.previews)};${themeScript}</script>`;
  return html.includes("</head>")
    ? html.replace("</head>", `${script}</head>`)
    : `${script}${html}`;
}

function rewriteGalleryBase(html: string, basePath: string): string {
  return html.replaceAll("/__musea__/", `${basePath.replace(/\/?$/, "/")}`);
}

function emitRootRedirect(
  emitFile: (asset: { type: "asset"; fileName: string; source: string }) => void,
  staticRoot: string,
  basePath: string,
): void {
  if (!staticRoot) return;
  const target = joinUrlPath(basePath, "");
  emitFile({
    type: "asset",
    fileName: "index.html",
    source: `<!doctype html><meta charset="utf-8"><meta http-equiv="refresh" content="0; url=${target}"><script>location.replace(${JSON.stringify(target)}+location.search+location.hash)</script>`,
  });
}

async function emitAxeVendor(
  emitFile: (asset: { type: "asset"; fileName: string; source: string | Uint8Array }) => void,
  staticRoot: string,
): Promise<void> {
  try {
    const require = createRequire(import.meta.url);
    const axePath = require.resolve("axe-core/axe.min.js");
    const source = await fs.promises.readFile(axePath);
    emitFile({
      type: "asset",
      fileName: joinFileName(staticRoot, "vendor/axe-core.min.js"),
      source,
    });
  } catch {
    // axe-core is an optional peer; static a11y keeps the same best-effort behavior as dev.
  }
}

function resolveGalleryDistDir(): string | null {
  const candidates = [path.join(moduleDir, "gallery"), path.resolve(moduleDir, "../dist/gallery")];
  return candidates.find((candidate) => fs.existsSync(path.join(candidate, "index.html"))) ?? null;
}

async function collectFiles(dir: string): Promise<string[]> {
  const entries = await fs.promises.readdir(dir, { withFileTypes: true });
  const files: string[] = [];
  for (const entry of entries) {
    const filePath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectFiles(filePath)));
    } else {
      files.push(filePath);
    }
  }
  return files;
}

function staticRootFromBasePath(basePath: string): string {
  return basePath.replace(/^\/+|\/+$/g, "");
}

function joinFileName(...parts: string[]): string {
  return parts.filter(Boolean).join("/");
}

function relativeUrl(fromDir: string, toFile: string): string {
  const relative = path.posix.relative(fromDir || ".", toFile);
  return relative.startsWith(".") ? relative : `./${relative}`;
}

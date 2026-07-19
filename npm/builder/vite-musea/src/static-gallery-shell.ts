import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { generateGalleryBody, generateGalleryScript } from "./gallery/template.js";
import { generateGalleryGlobalsScript } from "./gallery/globals.js";
import { rewriteGalleryBase, rewriteGalleryTextAssetBase } from "./static-base.js";
import type { StaticGalleryPayload } from "./static-data.js";
import type { StaticEmitContext } from "./static-export.js";

const moduleDir = path.dirname(fileURLToPath(import.meta.url));

export function joinFileName(...parts: string[]): string {
  return parts.filter(Boolean).join("/");
}

export async function emitGalleryShell(
  emitFile: (asset: { type: "asset"; fileName: string; source: string | Uint8Array }) => void,
  staticRoot: string,
  ctx: StaticEmitContext,
  payload: StaticGalleryPayload,
  galleryDistDir: string | null = resolveGalleryDistDir(),
): Promise<void> {
  if (!galleryDistDir) {
    const html = injectStaticGlobals(generateStaticFallbackGalleryHtml(ctx.basePath), ctx, payload);
    emitFile({ type: "asset", fileName: joinFileName(staticRoot, "index.html"), source: html });
    return;
  }

  for (const filePath of await collectFiles(galleryDistDir)) {
    const relative = path.relative(galleryDistDir, filePath).split(path.sep).join("/");
    const target = joinFileName(staticRoot, relative);
    const content = await fs.promises.readFile(filePath);
    if (relative === "index.html") {
      // The base rewrite targets the prebuilt, base-agnostic gallery shell;
      // the injected globals already carry fully prefixed preview URLs, so
      // rewriting after injection doubled the prefix for subpath base paths
      // (#3109). Rewrite first, then inject.
      const html = rewriteGalleryBase(content.toString("utf-8"), ctx.basePath);
      emitFile({
        type: "asset",
        fileName: target,
        source: injectStaticGlobals(html, ctx, payload),
      });
    } else {
      const source = rewriteGalleryTextAssetBase(content, relative, ctx.basePath);
      emitFile({ type: "asset", fileName: target, source });
    }
  }
}

export function injectStaticGlobals(
  html: string,
  ctx: StaticEmitContext,
  payload: StaticGalleryPayload,
): string {
  const script = `<script>${generateGalleryGlobalsScript({
    basePath: ctx.basePath,
    staticPreviews: payload.previews,
    themeConfig: ctx.themeConfig,
    tokenPreviewConfig: ctx.tokenPreviewConfig,
  })}</script>`;
  return html.includes("</head>")
    ? html.replace("</head>", `${script}</head>`)
    : `${script}${html}`;
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

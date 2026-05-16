/**
 * Individual route handler functions for the Musea gallery API.
 *
 * Extracted from api-routes.ts to keep file sizes manageable.
 * These handle GET /api/arts/:path/... sub-routes.
 */

import fs from "node:fs";

import type { ApiRoutesContext, SendJson, SendError } from "./index.js";
import { allowedSourceRoots, resolveComponentSourcePath } from "../component-source.js";
import { loadNative, analyzeSfcFallback } from "../native-loader.js";

export { handleArtPalette } from "./handler-palette.js";

/** GET /api/arts/:path/source */
export async function handleArtSource(
  ctx: ApiRoutesContext,
  match: RegExpMatchArray,
  sendJson: SendJson,
  sendError: SendError,
): Promise<void> {
  const artPath = decodeURIComponent(match[1]);
  const art = ctx.artFiles.get(artPath);
  if (!art) {
    sendError("Art not found", 404);
    return;
  }

  try {
    const source = await fs.promises.readFile(artPath, "utf-8");
    sendJson({ source, path: artPath });
  } catch (e) {
    sendError(e instanceof Error ? e.message : String(e));
  }
}

/** GET /api/arts/:path/analysis */
export async function handleArtAnalysis(
  ctx: ApiRoutesContext,
  match: RegExpMatchArray,
  sendJson: SendJson,
  sendError: SendError,
): Promise<void> {
  const artPath = decodeURIComponent(match[1]);
  const art = ctx.artFiles.get(artPath);
  if (!art) {
    sendError("Art not found", 404);
    return;
  }

  try {
    const resolvedComponentPath = resolveComponentSourcePath(
      art,
      artPath,
      allowedSourceRoots(ctx.config.root, ctx.scanRoots),
    );

    if (resolvedComponentPath) {
      const source = await fs.promises.readFile(resolvedComponentPath, "utf-8");
      const binding = loadNative();
      if (binding.analyzeSfc) {
        const analysis = binding.analyzeSfc(source, {
          filename: resolvedComponentPath,
        });
        sendJson(analysis);
      } else {
        const analysis = analyzeSfcFallback(source, {
          filename: resolvedComponentPath,
        });
        sendJson(analysis);
      }
    } else {
      sendJson({ props: [], emits: [] });
    }
  } catch (e) {
    sendError(e instanceof Error ? e.message : String(e));
  }
}

/** GET /api/arts/:path/docs */
export async function handleArtDocs(
  ctx: ApiRoutesContext,
  match: RegExpMatchArray,
  sendJson: SendJson,
  sendError: SendError,
): Promise<void> {
  const artPath = decodeURIComponent(match[1]);
  const art = ctx.artFiles.get(artPath);
  if (!art) {
    sendError("Art not found", 404);
    return;
  }

  try {
    const source = await fs.promises.readFile(artPath, "utf-8");
    const binding = loadNative();
    if (binding.generateArtDoc) {
      const doc = binding.generateArtDoc(source, {
        filename: artPath,
      });
      // Replace Self with component name and format indentation
      let markdown = doc.markdown || "";
      const componentName = art.metadata.title || "Component";
      markdown = markdown
        .replace(/<Self(\s|>|\/)/g, `<${componentName}$1`)
        .replace(/<\/Self>/g, `</${componentName}>`);
      // Fix indentation in code blocks
      markdown = markdown.replace(
        /```(\w*)\n([\s\S]*?)```/g,
        (_match: string, lang: string, code: string) => {
          const lines = code.split("\n");
          let minIndent = Infinity;
          for (const line of lines) {
            if (line.trim()) {
              const indent = line.match(/^(\s*)/)?.[1].length || 0;
              minIndent = Math.min(minIndent, indent);
            }
          }
          if (minIndent === Infinity) minIndent = 0;
          let formatted: string;
          if (minIndent > 0) {
            formatted = lines.map((line: string) => line.slice(minIndent)).join("\n");
          } else {
            let restIndent = Infinity;
            for (let i = 1; i < lines.length; i++) {
              if (lines[i].trim()) {
                const indent = lines[i].match(/^(\s*)/)?.[1].length || 0;
                restIndent = Math.min(restIndent, indent);
              }
            }
            if (restIndent === Infinity || restIndent === 0) {
              formatted = lines.join("\n");
            } else {
              formatted = lines
                .map((line: string, i: number) => (i === 0 ? line : line.slice(restIndent)))
                .join("\n");
            }
          }
          return "```" + lang + "\n" + formatted + "```";
        },
      );
      sendJson({ ...doc, markdown });
    } else {
      sendJson({
        markdown: "",
        title: art.metadata.title,
        variant_count: art.variants.length,
      });
    }
  } catch (e) {
    sendError(e instanceof Error ? e.message : String(e));
  }
}

/** GET /api/arts/:path/variants/:name/a11y */
export function handleArtA11y(
  ctx: ApiRoutesContext,
  match: RegExpMatchArray,
  sendJson: SendJson,
  sendError: SendError,
): void {
  const artPath = decodeURIComponent(match[1]);
  const _variantName = decodeURIComponent(match[2]);
  const art = ctx.artFiles.get(artPath);
  if (!art) {
    sendError("Art not found", 404);
    return;
  }

  // Return empty a11y results (populated after VRT --a11y run)
  sendJson({ violations: [], passes: 0, incomplete: 0 });
}

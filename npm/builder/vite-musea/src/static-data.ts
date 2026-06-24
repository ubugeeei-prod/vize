import crypto from "node:crypto";
import fs from "node:fs";
import type { ResolvedConfig } from "vite";

import { handleTokensGet, handleTokensUsage } from "./api-tokens.js";
import {
  handleArtAnalysis,
  handleArtA11y,
  handleArtDocs,
  handleArtPalette,
  handleArtSource,
} from "./api-routes/handlers.js";
import type { ApiRoutesContext, SendError, SendJson } from "./api-routes/index.js";
import type { ArtFileInfo } from "./types/index.js";

export interface StaticGalleryPayload {
  arts: ArtFileInfo[];
  previews: Record<string, Record<string, string>>;
  details: Record<string, StaticArtDetails>;
  tokens: unknown;
  tokenUsage: unknown;
}

export interface StaticArtDetails {
  source: unknown;
  palette: unknown;
  analysis: unknown;
  docs: unknown;
  a11y: Record<string, unknown>;
}

export interface StaticGalleryDataContext {
  config: ResolvedConfig;
  artFiles: Map<string, ArtFileInfo>;
  scanRoots: string[];
  tokensPath: string | undefined;
  /**
   * Optional outer project root used as an additional allowed boundary for
   * `tokensPath` resolution during static builds.
   */
  projectRoot?: string;
  basePath: string;
  resolvedPreviewCss: string[];
  resolvedPreviewSetup: string | null;
  devSessionToken: string;
}

export function staticPreviewId(artPath: string, variantName: string): string {
  return crypto
    .createHash("sha256")
    .update(artPath)
    .update("\0")
    .update(variantName)
    .digest("hex")
    .slice(0, 20);
}

export function joinUrlPath(basePath: string, ...segments: string[]): string {
  const normalizedBase = basePath === "/" ? "" : basePath.replace(/\/+$/, "");
  const suffix = segments
    .map((segment) => segment.replace(/^\/+|\/+$/g, ""))
    .filter(Boolean)
    .join("/");
  return `${normalizedBase}/${suffix}`;
}

export async function createStaticGalleryPayload(
  ctx: StaticGalleryDataContext,
): Promise<StaticGalleryPayload> {
  const apiCtx = createApiContext(ctx);
  const arts = Array.from(ctx.artFiles.values());
  const previews: StaticGalleryPayload["previews"] = {};
  const details: StaticGalleryPayload["details"] = {};

  for (const art of arts) {
    previews[art.path] = {};
    const a11y: Record<string, unknown> = {};

    for (const variant of art.variants) {
      const id = staticPreviewId(art.path, variant.name);
      previews[art.path][variant.name] = joinUrlPath(ctx.basePath, "preview", `${id}.html`);
      a11y[variant.name] = await captureJson((sendJson, sendError) => {
        handleArtA11y(apiCtx, artVariantMatch(art.path, variant.name), sendJson, sendError);
      }, emptyA11y());
    }

    details[art.path] = {
      source: await captureJson(
        (sendJson, sendError) => {
          return handleArtSource(apiCtx, artMatch(art.path), sendJson, sendError);
        },
        { source: "", path: art.path },
      ),
      palette: await captureJson((sendJson, sendError) => {
        return handleArtPalette(apiCtx, artMatch(art.path), sendJson, sendError);
      }, emptyPalette(art)),
      analysis: await captureJson(
        (sendJson, sendError) => {
          return handleArtAnalysis(apiCtx, artMatch(art.path), sendJson, sendError);
        },
        { props: [], emits: [] },
      ),
      docs: await captureJson((sendJson, sendError) => {
        return handleArtDocs(apiCtx, artMatch(art.path), sendJson, sendError);
      }, emptyDocs(art)),
      a11y,
    };
  }

  return {
    arts,
    previews,
    details,
    tokens: await captureJson((sendJson) => handleTokensGet(apiCtx, sendJson), emptyTokens()),
    tokenUsage: await captureJson((sendJson) => handleTokensUsage(apiCtx, sendJson), {}),
  };
}

function createApiContext(ctx: StaticGalleryDataContext): ApiRoutesContext {
  return {
    config: ctx.config,
    artFiles: ctx.artFiles,
    scanRoots: ctx.scanRoots,
    tokensPath: ctx.tokensPath,
    projectRoot: ctx.projectRoot,
    basePath: ctx.basePath,
    resolvedPreviewCss: ctx.resolvedPreviewCss,
    resolvedPreviewSetup: ctx.resolvedPreviewSetup,
    devSessionToken: ctx.devSessionToken,
    processArtFile: async (filePath: string) => {
      await fs.promises.access(filePath);
    },
    getDevServerPort: () => 5173,
  };
}

async function captureJson<T>(
  run: (sendJson: SendJson, sendError: SendError) => void | Promise<void>,
  fallback: T,
): Promise<unknown> {
  let captured: unknown = fallback;
  const sendJson: SendJson = (data: unknown) => {
    captured = data;
  };
  const sendError: SendError = (message: string, status = 500) => {
    captured = { error: message, status };
  };
  await run(sendJson, sendError);
  return captured;
}

function artMatch(artPath: string): RegExpMatchArray {
  return ["", encodeURIComponent(artPath)] as unknown as RegExpMatchArray;
}

function artVariantMatch(artPath: string, variantName: string): RegExpMatchArray {
  return [
    "",
    encodeURIComponent(artPath),
    encodeURIComponent(variantName),
  ] as unknown as RegExpMatchArray;
}

function emptyPalette(art: ArtFileInfo): unknown {
  return {
    title: art.metadata.title,
    controls: [],
    groups: [],
    json: "{}",
    typescript: "",
  };
}

function emptyDocs(art: ArtFileInfo): unknown {
  return {
    markdown: "",
    title: art.metadata.title,
    variant_count: art.variants.length,
  };
}

function emptyA11y(): unknown {
  return { violations: [], passes: 0, incomplete: 0 };
}

function emptyTokens(): unknown {
  return {
    categories: [],
    tokenMap: {},
    meta: { filePath: "", tokenCount: 0, primitiveCount: 0, semanticCount: 0 },
  };
}

import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { resolveSnapshotIdentity } from "./vrt/snapshot-identity.js";
import type { ResolvedConfig } from "vite";

import { handleTokensGet, handleTokensUsage } from "./api-tokens.js";
import {
  handleArtAnalysis,
  handleArtDocs,
  handleArtPalette,
  handleArtSource,
} from "./api-routes/handlers.js";
import { sortedArts } from "./art-order.js";
import type { ApiRoutesContext, SendError, SendJson } from "./api-routes/index.js";
import type { ArtFileInfo } from "./types/index.js";

export interface StaticGalleryPayload {
  arts: ArtFileInfo[];
  snapshotIdentityVersion: 1;
  snapshotIdentities: Record<string, string>;
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
  const sourceArts = sortedArts(ctx.artFiles.values());
  const arts: ArtFileInfo[] = [];
  const previews: StaticGalleryPayload["previews"] = Object.create(null);
  const details: StaticGalleryPayload["details"] = {};
  const snapshotIdentities: Record<string, string> = Object.create(null);
  const identityRoot = resolveStaticIdentityRoot(
    ctx.config.root,
    sourceArts.map((art) => art.path),
    ctx.projectRoot,
  );
  const publicPaths = new Map<string, string>();
  for (const art of sourceArts) {
    publicPaths.set(art.path, resolveStaticArtIdentity(art.path, identityRoot));
  }

  for (const sourceArt of sourceArts) {
    const publicPath = publicPaths.get(sourceArt.path)!;
    const art = publicArtInfo(sourceArt, publicPath, identityRoot);
    arts.push(art);
    snapshotIdentities[publicPath] = publicPath;
    previews[publicPath] = Object.create(null);

    for (const variant of art.variants) {
      const id = staticPreviewId(publicPath, variant.name);
      previews[publicPath][variant.name] = joinUrlPath(ctx.basePath, "preview", `${id}.html`);
    }

    details[publicPath] = {
      source: withPublicSourcePath(
        await captureJson(
          (sendJson, sendError) => {
            return handleArtSource(apiCtx, artMatch(sourceArt.path), sendJson, sendError);
          },
          { source: "", path: sourceArt.path },
        ),
        publicPath,
      ),
      palette: await captureJson((sendJson, sendError) => {
        return handleArtPalette(apiCtx, artMatch(sourceArt.path), sendJson, sendError);
      }, emptyPalette(art)),
      analysis: await captureJson(
        (sendJson, sendError) => {
          return handleArtAnalysis(apiCtx, artMatch(sourceArt.path), sendJson, sendError);
        },
        { props: [], emits: [] },
      ),
      docs: await captureJson((sendJson, sendError) => {
        return handleArtDocs(apiCtx, artMatch(sourceArt.path), sendJson, sendError);
      }, emptyDocs(art)),
    };
  }

  const tokens = await captureJson((sendJson) => handleTokensGet(apiCtx, sendJson), emptyTokens());
  const tokenUsage = await captureJson((sendJson) => handleTokensUsage(apiCtx, sendJson), {});
  return {
    arts,
    snapshotIdentityVersion: 1,
    snapshotIdentities,
    previews,
    details,
    tokens: withPublicTokenPath(tokens, identityRoot),
    tokenUsage: withPublicUsagePaths(tokenUsage, publicPaths),
  };
}

export function resolveStaticIdentityRoot(
  viteRoot: string,
  artPaths: Iterable<string>,
  projectRoot?: string,
): string {
  if (projectRoot) return path.resolve(projectRoot);
  let root = path.resolve(viteRoot);
  for (const artPath of artPaths) {
    const directory = path.dirname(path.resolve(artPath));
    while (!isPathInside(root, directory)) {
      const parent = path.dirname(root);
      if (parent === root) {
        throw new Error(
          `Art is outside the Vite root on another filesystem: ${artPath}. Configure projectRoot.`,
        );
      }
      root = parent;
    }
  }
  return root;
}

export function resolveStaticArtIdentity(artPath: string, identityRoot: string): string {
  return resolveSnapshotIdentity(artPath, identityRoot);
}

function publicArtInfo(art: ArtFileInfo, publicPath: string, identityRoot: string): ArtFileInfo {
  return {
    ...art,
    path: publicPath,
    ...(art.componentPath
      ? { componentPath: publicSourceIdentity(art.componentPath, identityRoot) }
      : {}),
  };
}

function withPublicSourcePath(value: unknown, publicPath: string): unknown {
  if (!isRecord(value) || typeof value.path !== "string") return value;
  return { ...value, path: publicPath };
}

function withPublicTokenPath(value: unknown, identityRoot: string): unknown {
  if (!isRecord(value) || !isRecord(value.meta) || typeof value.meta.filePath !== "string")
    return value;
  if (!path.isAbsolute(value.meta.filePath)) return value;
  return {
    ...value,
    meta: {
      ...value.meta,
      filePath: publicSourceIdentity(value.meta.filePath, identityRoot),
    },
  };
}

function withPublicUsagePaths(value: unknown, publicPaths: Map<string, string>): unknown {
  if (!isRecord(value)) return value;
  return Object.fromEntries(
    Object.entries(value).map(([token, entries]) => [
      token,
      Array.isArray(entries)
        ? entries.map((entry) =>
            isRecord(entry) && typeof entry.artPath === "string"
              ? {
                  ...entry,
                  artPath: publicPaths.get(entry.artPath) ?? path.basename(entry.artPath),
                }
              : entry,
          )
        : entries,
    ]),
  );
}

function publicSourceIdentity(sourcePath: string, identityRoot: string): string {
  const relative = path.relative(identityRoot, path.resolve(sourcePath));
  if (
    relative === "" ||
    relative === ".." ||
    relative.startsWith(`..${path.sep}`) ||
    path.isAbsolute(relative)
  ) {
    return path.basename(sourcePath);
  }
  return relative.split(path.sep).join("/");
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isPathInside(root: string, candidate: string): boolean {
  const relative = path.relative(root, candidate);
  return (
    relative === "" ||
    (relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative))
  );
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

function emptyTokens(): unknown {
  return {
    categories: [],
    tokenMap: {},
    meta: { filePath: "", tokenCount: 0, primitiveCount: 0, semanticCount: 0 },
  };
}

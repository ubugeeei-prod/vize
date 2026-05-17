/**
 * Musea gallery API route handlers.
 *
 * Extracted from the main plugin to keep file sizes manageable.
 * Provides REST API endpoints consumed by the gallery UI:
 * - GET/POST/PUT/DELETE /api/tokens  (delegated to api-tokens.ts)
 * - GET /api/arts, /api/arts/:path, /api/arts/:path/source, etc.
 * - POST /api/preview-with-props
 * - POST /api/generate
 * - POST /api/run-vrt
 */

import type { IncomingMessage, ServerResponse } from "node:http";
import type { ResolvedConfig } from "vite";
import fs from "node:fs";

import type { ArtFileInfo } from "../types/index.js";
import {
  handleTokensUsage,
  handleTokensGet,
  handleTokensCreate,
  handleTokensUpdate,
  handleTokensDelete,
} from "../api-tokens.js";
import {
  handleArtSource,
  handleArtPalette,
  handleArtAnalysis,
  handleArtDocs,
  handleArtA11y,
} from "./handlers.js";
import { handlePreviewWithProps, handleGenerate, handleRunVrt } from "./post-handlers.js";
import {
  collectRequestBody,
  decodeUrlComponent,
  DEFAULT_API_BODY_LIMIT_BYTES,
  HttpError,
  resolveInside,
  validateDevApiRequest,
} from "../security.js";

/** Dependencies injected from the plugin closure. */
export interface ApiRoutesContext {
  config: ResolvedConfig;
  artFiles: Map<string, ArtFileInfo>;
  scanRoots: string[];
  tokensPath: string | undefined;
  basePath: string;
  resolvedPreviewCss: string[];
  resolvedPreviewSetup: string | null;
  devSessionToken: string;
  apiBodyLimit?: number;
  processArtFile: (filePath: string) => Promise<void>;
  /** Reference to the dev server for VRT port resolution */
  getDevServerPort: () => number;
}

export type SendJson = (data: unknown, status?: number) => void;
export type SendError = (message: string, status?: number) => void;
export type ReadBody = () => Promise<string>;

type NextFn = () => void;

/**
 * Create the API middleware handler for the Musea gallery.
 *
 * Returns a Connect-compatible middleware function that handles all
 * `/api/...` sub-routes under the configured basePath.
 */
export function createApiMiddleware(ctx: ApiRoutesContext) {
  return async (req: IncomingMessage, res: ServerResponse, next: NextFn) => {
    const sendJson: SendJson = (data: unknown, status = 200) => {
      res.statusCode = status;
      res.setHeader("Content-Type", "application/json");
      res.end(JSON.stringify(data));
    };

    const sendError: SendError = (message: string, status = 500) => {
      sendJson({ error: message }, status);
    };

    const readBody: ReadBody = () =>
      collectRequestBody(req, ctx.apiBodyLimit ?? DEFAULT_API_BODY_LIMIT_BYTES);

    const url = req.url || "/";

    try {
      const requestError = validateDevApiRequest(req, ctx.devSessionToken);
      if (requestError) {
        sendError(requestError.message, requestError.status);
        return;
      }

      // --- GET /api/arts ---
      if (url === "/arts" && req.method === "GET") {
        sendJson(Array.from(ctx.artFiles.values()));
        return;
      }

      // --- Token routes (delegated to api-tokens.ts) ---
      if (url === "/tokens/usage" && req.method === "GET") {
        await handleTokensUsage(ctx, sendJson);
        return;
      }
      if (url === "/tokens" && req.method === "GET") {
        await handleTokensGet(ctx, sendJson);
        return;
      }
      if (url === "/tokens" && req.method === "POST") {
        await handleTokensCreate(ctx, readBody, sendJson, sendError);
        return;
      }
      if (url === "/tokens" && req.method === "PUT") {
        await handleTokensUpdate(ctx, readBody, sendJson, sendError);
        return;
      }
      if (url === "/tokens" && req.method === "DELETE") {
        await handleTokensDelete(ctx, readBody, sendJson, sendError);
        return;
      }

      // --- PUT /api/arts/:path/source (update art source) ---
      if (url?.startsWith("/arts/") && req.method === "PUT") {
        const rest = url.slice(6);
        const sourceMatch = rest.match(/^(.+)\/source$/);
        if (sourceMatch) {
          const artPath = decodeUrlComponent(sourceMatch[1], "art path");
          const art = ctx.artFiles.get(artPath);
          if (!art) {
            sendError("Art not found", 404);
            return;
          }

          const safeArtPath = resolveInside(ctx.config.root, artPath, "art path");
          const body = await readBody();
          const { source } = JSON.parse(body) as { source: string };
          if (typeof source !== "string") {
            sendError("Missing required field: source", 400);
            return;
          }
          await fs.promises.writeFile(safeArtPath, source, "utf-8");
          await ctx.processArtFile(safeArtPath);
          sendJson({ success: true });
          return;
        }
        next();
        return;
      }

      // --- GET /api/arts/:path/... sub-routes ---
      if (url?.startsWith("/arts/") && req.method === "GET") {
        const rest = url.slice(6);

        const sourceMatch = rest.match(/^(.+)\/source$/);
        const paletteMatch = rest.match(/^(.+)\/palette$/);
        const analysisMatch = rest.match(/^(.+)\/analysis$/);
        const docsMatch = rest.match(/^(.+)\/docs$/);
        const a11yMatch = rest.match(/^(.+)\/variants\/([^/]+)\/a11y$/);

        if (sourceMatch) {
          await handleArtSource(ctx, sourceMatch, sendJson, sendError);
          return;
        }

        if (paletteMatch) {
          await handleArtPalette(ctx, paletteMatch, sendJson, sendError);
          return;
        }

        if (analysisMatch) {
          await handleArtAnalysis(ctx, analysisMatch, sendJson, sendError);
          return;
        }

        if (docsMatch) {
          await handleArtDocs(ctx, docsMatch, sendJson, sendError);
          return;
        }

        if (a11yMatch) {
          handleArtA11y(ctx, a11yMatch, sendJson, sendError);
          return;
        }

        // GET /api/arts/:path (no sub-resource)
        const artPath = decodeUrlComponent(rest, "art path");
        const art = ctx.artFiles.get(artPath);
        if (art) {
          sendJson(art);
        } else {
          sendError("Art not found", 404);
        }
        return;
      }

      // --- POST routes (delegated to post-handlers.ts) ---
      if (req.method === "POST") {
        const body = await readBody();

        if (url === "/preview-with-props") {
          handlePreviewWithProps(ctx, body, res, sendJson, sendError);
          return;
        }

        if (url === "/generate") {
          await handleGenerate(ctx, body, sendJson, sendError);
          return;
        }

        if (url === "/run-vrt") {
          await handleRunVrt(ctx, body, sendJson, sendError);
          return;
        }
      }

      next();
    } catch (e) {
      if (e instanceof HttpError) {
        sendError(e.message, e.status);
        return;
      }
      sendError(e instanceof Error ? e.message : String(e));
    }
  };
}

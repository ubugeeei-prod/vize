import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import { Readable } from "node:stream";
import os from "node:os";
import path from "node:path";
import type { ResolvedConfig } from "vite";

import { createApiMiddleware, type ApiRoutesContext } from "./index.ts";
import type { ArtFileInfo } from "../types/art.ts";

interface CapturedResponse {
  body: string;
  headers: Record<string, string>;
  nextCalled: boolean;
  statusCode: number;
}

function createArt(pathname: string): ArtFileInfo {
  return {
    path: pathname,
    metadata: { title: "Escape", tags: [], status: "ready" },
    variants: [],
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
}

function createContext(root: string, artFiles = new Map<string, ArtFileInfo>()): ApiRoutesContext {
  return {
    config: { root } as ResolvedConfig,
    artFiles,
    scanRoots: [root],
    tokensPath: undefined,
    basePath: "/__musea__",
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
    devSessionToken: "test-session",
    processArtFile: async () => {},
    getDevServerPort: () => 5173,
  };
}

function authorizedJsonHeaders(ctx: ApiRoutesContext): IncomingMessage["headers"] {
  return {
    host: "localhost:5173",
    origin: "http://localhost:5173",
    "content-type": "application/json",
    "x-musea-session": ctx.devSessionToken,
  };
}

async function invokeApi(
  ctx: ApiRoutesContext,
  init: {
    body?: string;
    headers?: IncomingMessage["headers"];
    method: string;
    url: string;
  },
): Promise<CapturedResponse> {
  return await new Promise((resolve, reject) => {
    const captured: CapturedResponse = {
      body: "",
      headers: {},
      nextCalled: false,
      statusCode: 200,
    };

    const req = Readable.from(init.body === undefined ? [] : [init.body]) as IncomingMessage & {
      socket: { remoteAddress: string };
    };
    req.method = init.method;
    req.url = init.url;
    req.headers = init.headers ?? {};
    req.socket = { remoteAddress: "127.0.0.1" };

    const res = {
      get statusCode() {
        return captured.statusCode;
      },
      set statusCode(value: number) {
        captured.statusCode = value;
      },
      setHeader(name: string, value: number | string | string[]) {
        captured.headers[name.toLowerCase()] = Array.isArray(value)
          ? value.join(", ")
          : String(value);
      },
      end(chunk?: Buffer | string) {
        if (chunk) {
          captured.body += Buffer.isBuffer(chunk) ? chunk.toString("utf-8") : chunk;
        }
        resolve(captured);
      },
    } as ServerResponse;

    const next = () => {
      captured.nextCalled = true;
      resolve(captured);
    };

    Promise.resolve(createApiMiddleware(ctx)(req, res, next)).catch(reject);
  });
}

void test("createApiMiddleware refuses art source writes through a symlink", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-source-symlink-"));
  const artPath = "src/Escape.art.vue";
  const artFilePath = path.join(tempDir, artPath);
  const secretPath = path.join(tempDir, ".env");

  try {
    await fs.promises.mkdir(path.dirname(artFilePath), { recursive: true });
    await fs.promises.writeFile(secretPath, "SECRET=1\n");
    await fs.promises.symlink(secretPath, artFilePath);

    const ctx = createContext(tempDir, new Map([[artPath, createArt(artPath)]]));
    const response = await invokeApi(ctx, {
      method: "PUT",
      url: `/arts/${encodeURIComponent(artPath)}/source`,
      headers: authorizedJsonHeaders(ctx),
      body: JSON.stringify({ source: "overwritten" }),
    });

    assert.equal(response.statusCode, 400);
    assert.deepEqual(JSON.parse(response.body), { error: "art path must be a .art.vue file" });
    assert.equal(await fs.promises.readFile(secretPath, "utf-8"), "SECRET=1\n");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

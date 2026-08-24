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
    metadata: {
      title: "Escape",
      tags: [],
      status: "ready",
    },
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
    remoteAddress?: string;
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

    const req = Readable.from(init.body === undefined ? [] : [init.body]) as IncomingMessage;
    req.method = init.method;
    req.url = init.url;
    req.headers = init.headers ?? {};
    req.socket = { remoteAddress: init.remoteAddress ?? "127.0.0.1" } as IncomingMessage["socket"];

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

void test("createApiMiddleware rejects source writes from non-loopback clients", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-loopback-"));
  const artPath = path.join(tempDir, "src", "Button.art.vue");

  try {
    await fs.promises.mkdir(path.dirname(artPath), { recursive: true });
    await fs.promises.writeFile(artPath, "original", "utf-8");

    const ctx = createContext(tempDir, new Map([[artPath, createArt(artPath)]]));
    const response = await invokeApi(ctx, {
      method: "PUT",
      url: `/arts/${encodeURIComponent(artPath)}/source`,
      headers: authorizedJsonHeaders(ctx),
      remoteAddress: "192.168.1.20",
      body: JSON.stringify({ source: "escaped" }),
    });

    assert.equal(response.statusCode, 403);
    assert.match(response.body, /loopback/);
    assert.equal(await fs.promises.readFile(artPath, "utf-8"), "original");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("createApiMiddleware rejects source reads that follow a planted symlink", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-symlink-"));
  const root = path.join(tempDir, "root");
  const outside = path.join(tempDir, "outside");
  const artPath = path.join(root, "src", "Leak.art.vue");

  try {
    await fs.promises.mkdir(path.join(root, "src"), { recursive: true });
    await fs.promises.mkdir(outside);
    await fs.promises.writeFile(path.join(outside, "secret.txt"), "secret-from-outside", "utf-8");
    await fs.promises.symlink(path.join(outside, "secret.txt"), artPath);

    const ctx = createContext(root, new Map([[artPath, createArt(artPath)]]));
    const response = await invokeApi(ctx, {
      method: "GET",
      url: `/arts/${encodeURIComponent(artPath)}/source`,
    });

    assert.equal(response.statusCode, 400);
    assert.match(response.body, /escapes the allowed directory/);
    assert.doesNotMatch(response.body, /secret-from-outside/);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

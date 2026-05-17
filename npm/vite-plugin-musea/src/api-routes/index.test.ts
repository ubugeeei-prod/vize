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

    const req = Readable.from(init.body === undefined ? [] : [init.body]) as IncomingMessage;
    req.method = init.method;
    req.url = init.url;
    req.headers = init.headers ?? {};

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

void test("createApiMiddleware returns 400 for malformed encoded art paths", async () => {
  const response = await invokeApi(createContext(process.cwd()), {
    method: "GET",
    url: "/arts/%E0%A4%A/source",
  });

  assert.equal(response.statusCode, 400);
  assert.match(response.body, /art path is not valid URL encoding/);
});

void test("createApiMiddleware blocks source writes outside the configured root", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-security-"));
  const root = path.join(tempDir, "root");
  const outside = path.join(tempDir, "outside");
  const outsideArt = path.join(outside, "Escape.art.vue");

  try {
    await fs.promises.mkdir(root);
    await fs.promises.mkdir(outside);
    await fs.promises.writeFile(outsideArt, "original", "utf-8");

    const ctx = createContext(root, new Map([[outsideArt, createArt(outsideArt)]]));
    const response = await invokeApi(ctx, {
      method: "PUT",
      url: `/arts/${encodeURIComponent(outsideArt)}/source`,
      headers: {
        host: "localhost:5173",
        origin: "http://localhost:5173",
        "content-type": "application/json",
        "x-musea-session": ctx.devSessionToken,
      },
      body: JSON.stringify({ source: "escaped" }),
    });

    assert.equal(response.statusCode, 400);
    assert.match(response.body, /art path escapes the allowed directory/);
    assert.equal(await fs.promises.readFile(outsideArt, "utf-8"), "original");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

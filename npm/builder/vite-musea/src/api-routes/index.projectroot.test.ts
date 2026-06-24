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

void test("createApiMiddleware resolves tokensPath under the configured projectRoot", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-project-root-"));
  const projectRoot = path.join(tempDir, "project");
  const appRoot = path.join(projectRoot, "app");
  const designRoot = path.join(projectRoot, "design");
  const tokensDir = path.join(designRoot, "tokens");

  try {
    await fs.promises.mkdir(appRoot, { recursive: true });
    await fs.promises.mkdir(tokensDir, { recursive: true });
    await fs.promises.writeFile(
      path.join(tokensDir, "colors.tokens.json"),
      JSON.stringify({
        color: {
          primitive: {
            slate: {
              50: { value: "#f1f5f9" },
            },
          },
        },
      }),
      "utf-8",
    );

    const ctx = createContext(appRoot);
    // Default scanRoots stay at the Vite app root — no include widening — yet
    // `tokensPath` still resolves because the Nuxt-style project root is
    // exposed as an additional allowed boundary.
    ctx.scanRoots = [appRoot];
    ctx.tokensPath = "../design/tokens";
    ctx.projectRoot = projectRoot;

    const response = await invokeApi(ctx, {
      method: "GET",
      url: "/tokens",
    });

    assert.equal(response.statusCode, 200, response.body);
    const body = JSON.parse(response.body);
    assert.equal(body.error, undefined);
    assert.equal(body.meta.filePath, tokensDir);
    assert.equal(body.tokenMap["color.primitive.slate.50"].value, "#f1f5f9");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("createApiMiddleware still rejects tokensPath outside the projectRoot", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-api-project-root-deny-"));
  const projectRoot = path.join(tempDir, "project");
  const appRoot = path.join(projectRoot, "app");
  const outsideDir = path.join(tempDir, "outside-tokens");

  try {
    await fs.promises.mkdir(appRoot, { recursive: true });
    await fs.promises.mkdir(outsideDir, { recursive: true });
    await fs.promises.writeFile(
      path.join(outsideDir, "colors.tokens.json"),
      JSON.stringify({}),
      "utf-8",
    );

    const ctx = createContext(appRoot);
    ctx.scanRoots = [appRoot];
    ctx.tokensPath = "../../outside-tokens";
    ctx.projectRoot = projectRoot;

    const response = await invokeApi(ctx, {
      method: "GET",
      url: "/tokens",
    });

    assert.equal(response.statusCode, 200, response.body);
    const body = JSON.parse(response.body);
    assert.match(body.error ?? "", /tokensPath escapes the allowed directory/);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

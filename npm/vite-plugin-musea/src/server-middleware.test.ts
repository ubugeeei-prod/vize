import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import os from "node:os";
import path from "node:path";
import type { Connect, ViteDevServer } from "vite";

import {
  registerMiddleware,
  serveGalleryAsset,
  type MiddlewareContext,
} from "./server-middleware.ts";

interface CapturedResponse {
  body: string;
  headers: Record<string, string>;
  nextCalled: boolean;
  statusCode: number;
}

interface RegisteredMiddleware {
  route: string | null;
  handler: Connect.NextHandleFunction;
}

function createDevServer(root: string): {
  devServer: ViteDevServer;
  stack: RegisteredMiddleware[];
} {
  const stack: RegisteredMiddleware[] = [];
  const devServer = {
    config: { root, base: "/" },
    middlewares: {
      use(
        routeOrHandler: string | Connect.NextHandleFunction,
        maybeHandler?: Connect.NextHandleFunction,
      ) {
        if (typeof routeOrHandler === "string") {
          stack.push({
            route: routeOrHandler,
            handler: maybeHandler as Connect.NextHandleFunction,
          });
          return;
        }
        stack.push({ route: null, handler: routeOrHandler });
      },
    },
    transformIndexHtml: async (_url: string, html: string) => html,
    transformRequest: async () => null,
  } as unknown as ViteDevServer;

  return { devServer, stack };
}

function createContext(): MiddlewareContext {
  return {
    basePath: "/__musea__",
    devSessionToken: "test-session",
    themeConfig: undefined,
    artFiles: new Map(),
    scanRoots: [],
    resolvedPreviewCss: [],
    resolvedPreviewSetup: null,
  };
}

async function invokeMiddleware(
  handler: Connect.NextHandleFunction,
  url: string,
): Promise<CapturedResponse> {
  return await new Promise((resolve, reject) => {
    const captured: CapturedResponse = {
      body: "",
      headers: {},
      nextCalled: false,
      statusCode: 200,
    };
    const req = {
      method: "GET",
      url,
      headers: {},
    } as IncomingMessage;
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

    Promise.resolve(handler(req, res, next)).catch(reject);
  });
}

function createCapturedResponse(): {
  captured: CapturedResponse;
  res: ServerResponse;
} {
  const captured: CapturedResponse = {
    body: "",
    headers: {},
    nextCalled: false,
    statusCode: 200,
  };
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
    },
  } as ServerResponse;
  return { captured, res };
}

void test("gallery asset helper serves only contained assets with immutable cache headers", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-assets-"));

  try {
    await fs.promises.mkdir(path.join(tempDir, "assets"));
    await fs.promises.writeFile(path.join(tempDir, "assets", "app.js"), "export {};", "utf-8");

    const { captured, res } = createCapturedResponse();
    const handled = await serveGalleryAsset(tempDir, "/assets/app.js", res);

    assert.equal(handled, true);
    assert.equal(captured.statusCode, 200);
    assert.equal(captured.headers["content-type"], "application/javascript");
    assert.equal(captured.headers["cache-control"], "public, max-age=31536000, immutable");
    assert.equal(captured.body, "export {};");
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

void test("gallery asset middleware rejects encoded traversal", async () => {
  const { devServer, stack } = createDevServer(process.cwd());
  registerMiddleware(devServer, createContext());
  const galleryHandler = stack.find((entry) => entry.route === "/__musea__")?.handler;

  assert.ok(galleryHandler);
  const response = await invokeMiddleware(galleryHandler, "/assets/%2e%2e/package.json");

  assert.equal(response.statusCode, 400);
  assert.equal(response.nextCalled, false);
  assert.match(response.body, /asset path must not contain parent directory segments/);
});

void test("art module middleware returns 400 for malformed encoded art paths", async () => {
  const { devServer, stack } = createDevServer(process.cwd());
  registerMiddleware(devServer, createContext());
  const artHandler = stack.find((entry) => entry.route === "/__musea__/art")?.handler;

  assert.ok(artHandler);
  const response = await invokeMiddleware(artHandler, "/%E0%A4%A");

  assert.equal(response.statusCode, 400);
  assert.equal(response.nextCalled, false);
  assert.match(response.body, /art path is not valid URL encoding/);
});

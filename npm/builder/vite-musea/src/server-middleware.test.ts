import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import type { IncomingMessage, ServerResponse } from "node:http";
import os from "node:os";
import path from "node:path";
import { createServer as createViteServer, type Connect, type ViteDevServer } from "vite";

import {
  generateDevGlobalsScript,
  ignorePackedGalleryAssetTransforms,
  registerMiddleware,
  serveGalleryAsset,
  type MiddlewareContext,
} from "./server-middleware.ts";

void test("packed gallery assets bypass Vite transforms without hiding other modules", () => {
  const html = `<!doctype html>
<script type="module" crossorigin src="/gallery/assets/index-abc.js"></script>
<script type="module" src="/application.js"></script>
<script src="/gallery/assets/legacy.js"></script>`;

  const transformed = ignorePackedGalleryAssetTransforms(html, "/gallery/");

  assert.match(
    transformed,
    /<script vite-ignore type="module" crossorigin src="\/gallery\/assets\/index-abc\.js">/,
  );
  assert.match(transformed, /<script type="module" src="\/application\.js">/);
  assert.match(transformed, /<script src="\/gallery\/assets\/legacy\.js">/);
  assert.equal((transformed.match(/vite-ignore/gu) ?? []).length, 1);
});

void test("packed gallery keeps Vite development injection without pre-transforming assets", async () => {
  const server = await createViteServer({
    appType: "custom",
    configFile: false,
    logLevel: "silent",
    optimizeDeps: { noDiscovery: true },
    server: { middlewareMode: true },
  });
  const warmed: string[] = [];
  server.warmupRequest = async (url: string) => {
    warmed.push(url);
  };

  try {
    const transformed = await server.transformIndexHtml(
      "/gallery/",
      ignorePackedGalleryAssetTransforms(
        '<script type="module" src="/gallery/assets/index-abc.js"></script>',
        "/gallery",
      ),
    );

    assert.deepEqual(warmed, []);
    assert.doesNotMatch(transformed, /vite-ignore/u);
    assert.match(transformed, /\/@vite\/client/u);
    assert.match(transformed, /src="\/gallery\/assets\/index-abc\.js"/u);
  } finally {
    await server.close();
  }
});

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
  headers: IncomingMessage["headers"] = {},
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
      headers,
      socket: { remoteAddress: "127.0.0.1" },
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

void test("the gallery page carries the session token for local requests only", async () => {
  const { devServer, stack } = createDevServer(process.cwd());
  registerMiddleware(devServer, createContext());
  const galleryHandler = stack.find((entry) => entry.route === "/__musea__")?.handler;
  assert.ok(galleryHandler);
  const globals = async (headers: IncomingMessage["headers"]) => {
    const response = await invokeMiddleware(galleryHandler, "/", headers);
    assert.equal(response.statusCode, 200);
    assert.equal(response.headers["content-type"], "text/html");
    return /<script>(window\.__MUSEA_BASE_PATH__=[^<]*)<\/script>/.exec(response.body)?.[1];
  };

  assert.equal(
    await globals({ host: "localhost:5173" }),
    'window.__MUSEA_BASE_PATH__="/__musea__";window.__MUSEA_SESSION_TOKEN__="test-session";',
  );
  // The page still renders for other readers; their writes are refused by the API.
  for (const headers of [
    { host: "preview.tunnel.example" },
    { host: "localhost:5173", "x-forwarded-for": "203.0.113.7" },
    { host: "192.168.1.10:5173" },
  ]) {
    assert.equal(await globals(headers), 'window.__MUSEA_BASE_PATH__="/__musea__";');
  }
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

void test("dev gallery globals include token preview configuration", () => {
  const script = generateDevGlobalsScript("/__musea__", "session", undefined, {
    rules: [{ pathIncludes: "elevation.overlay", kind: "zIndex" }],
  });

  assert.match(
    script,
    /window\.__MUSEA_TOKEN_PREVIEWS__=\{"rules":\[\{"pathIncludes":"elevation\.overlay","kind":"zIndex"\}\]\};/,
  );
});

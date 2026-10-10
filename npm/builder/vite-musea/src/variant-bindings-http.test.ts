import assert from "node:assert/strict";
import test from "node:test";
import { createServer as createHttpServer } from "node:http";
import { createServer } from "vite";
import { registerMiddleware } from "./server-middleware.ts";
import type { ArtFileInfo } from "./types/art.ts";

void test("actual Vite HTTP previews bound duplicate-name errors and remain responsive", async (t) => {
  const art: ArtFileInfo = {
    path: "/repo/Host.art.vue",
    metadata: { title: "Invalid bindings", tags: [], status: "ready" },
    variants: [0, 1].map(() => ({
      name: "Default",
      template: "<button />",
      isDefault: false,
      skipVrt: false,
    })),
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
  const server = await createServer({
    configFile: false,
    logLevel: "silent",
    server: { middlewareMode: true, hmr: false },
    plugins: [
      {
        name: "musea-preview-error-contract",
        configureServer(server) {
          registerMiddleware(server, {
            basePath: "/gallery",
            devSessionToken: "unit",
            themeConfig: undefined,
            artFiles: new Map([[art.path, art]]),
            scanRoots: [],
            resolvedPreviewCss: [],
            resolvedPreviewSetup: null,
          });
        },
      },
    ],
  });
  const http = createHttpServer(server.middlewares);
  t.after(async () => {
    if (http.listening) {
      http.closeAllConnections();
      await new Promise<void>((resolve, reject) =>
        http.close((error) => (error ? reject(error) : resolve())),
      );
    }
    await server.close();
  });
  await new Promise<void>((resolve) => http.listen(0, "127.0.0.1", resolve));
  const address = http.address();
  assert.ok(address && typeof address !== "string");
  const origin = `http://127.0.0.1:${address.port}`;
  const parameters = new URLSearchParams({ art: art.path, variant: "Default" });
  for (let attempt = 0; attempt < 2; attempt++) {
    const response = await fetch(`${origin}/gallery/preview-module?${parameters.toString()}`, {
      signal: AbortSignal.timeout(2000),
    });
    assert.equal(response.status, 500);
    assert.ok((await response.text()).includes("Duplicate Musea variant name"));
  }
  const missing = await fetch(`${origin}/gallery/preview-module`, {
    signal: AbortSignal.timeout(2000),
  });
  assert.equal(missing.status, 400);
});

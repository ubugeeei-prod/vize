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
  const valid = { ...art, path: "/repo/Valid.art.vue", variants: art.variants.slice(0, 1) };
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
            artFiles: new Map([
              [art.path, art],
              [valid.path, valid],
            ]),
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
  const duplicateRoutes = [
    `/gallery/preview-module?${parameters.toString()}`,
    `/gallery/art/${encodeURIComponent(art.path)}`,
  ];
  for (let attempt = 0; attempt < 2; attempt++) {
    for (const route of duplicateRoutes) {
      const response = await fetch(`${origin}${route}`, { signal: AbortSignal.timeout(2000) });
      assert.equal(response.status, 500);
      assert.ok((await response.text()).includes("Duplicate Musea variant name"));
    }
  }
  const ordinary = await fetch(`${origin}/gallery/art/${encodeURIComponent(valid.path)}`, {
    signal: AbortSignal.timeout(2000),
  });
  assert.equal(ordinary.status, 200);
  assert.equal(ordinary.headers.get("content-type"), "application/javascript");
  assert.equal(ordinary.headers.get("cache-control"), null);
  assert.ok((await ordinary.text()).includes("export default Default"));
  const missing = await fetch(`${origin}/gallery/preview-module`, {
    signal: AbortSignal.timeout(2000),
  });
  assert.equal(missing.status, 400);
});

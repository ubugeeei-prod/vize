import assert from "node:assert/strict";
import crypto from "node:crypto";
import { cp, mkdir, mkdtemp, readFile, rm, stat, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { inspect } from "node:util";
import { build } from "vite";
import vize from "../../vite/src/index.ts";
import { musea } from "./plugin/index.ts";
import type { StaticGalleryPayload } from "./static-data.ts";

export const repository = fileURLToPath(new URL("../../../../", import.meta.url));
export const defaults = { brand: "default", scheme: "light", locale: "en" };
export const changed = { brand: "ocean", scheme: "dark", locale: "ja" };
export const fixtureRoot = path.join(
  repository,
  "tests/tooling/fixtures/musea/static-variant-name",
);
export const sha256 = (bytes: Uint8Array | string) =>
  crypto.createHash("sha256").update(bytes).digest("hex");

/** Use authored native inputs and the complete production plugin/build pipeline. */
export async function buildNativeStaticGallery(output: string) {
  const root = await mkdtemp(path.join(os.tmpdir(), "musea-native-static-"));
  await mkdir(output, { recursive: true });
  try {
    const gallery = path.join(repository, "npm/builder/vite-musea/dist/gallery/index.html");
    await stat(gallery); // Refuse the lightweight fallback shell in this native browser law.
    await cp(path.join(repository, "examples/vite-musea/src"), path.join(root, "src"), {
      recursive: true,
    });
    await cp(
      path.join(repository, "examples/vite-musea/musea.preview.ts"),
      path.join(root, "musea.preview.ts"),
    );
    await mkdir(path.join(root, "src/regression"));
    for (const file of ["Host.vue", "Host.art.vue"])
      await cp(path.join(fixtureRoot, file), path.join(root, "src/regression", file));
    await writeFile(
      path.join(root, "preview.setup.ts"),
      `import setup from './musea.preview.ts';
export default async function sourceSetup(app, context) {
  Reflect.set(window, '__publicDocumentToken', crypto.randomUUID());
  Reflect.set(window, '__publicSetupCalls', (Reflect.get(window, '__publicSetupCalls') || 0) + 1);
  Reflect.set(window, '__publicFirstGlobals', { ...context.globals.value });
  await new Promise(resolve => setTimeout(resolve, 120));
  setup(app, context);
}
`,
    );
    const inputs = await Promise.all(
      [
        "src/components/MuseaButton.vue",
        "src/regression/Host.vue",
        "src/regression/Host.art.vue",
        "musea.preview.ts",
        "preview.setup.ts",
      ].map(async (file) => {
        const bytes = await readFile(path.join(root, file));
        return { file, sha256: sha256(bytes), source: bytes.toString() };
      }),
    );
    await writeFile(path.join(output, "inputs.json"), JSON.stringify(inputs, null, 2));
    const directory = path.join(root, "dist");
    await build({
      root,
      configFile: false,
      base: "/built/",
      logLevel: "warn",
      cacheDir: path.join(root, ".vite-cache"),
      resolve: {
        alias: { vue: fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js")) },
      },
      plugins: [
        vize(),
        musea({
          include: ["src/**/*.vue"],
          inlineArt: true,
          basePath: "/gallery/",
          previewSetup: "preview.setup.ts",
          toolbar: [
            {
              id: "brand",
              title: "Brand",
              type: "select",
              options: ["default", "ocean"],
              default: "default",
            },
            {
              id: "scheme",
              title: "Component theme",
              type: "toggle",
              default: "light",
              options: [
                { value: "light", label: "Light" },
                { value: "dark", label: "Dark" },
              ],
            },
            { id: "locale", title: "Locale", type: "select", options: ["en", "ja"], default: "en" },
          ],
        }),
      ],
      build: { outDir: directory, emptyOutDir: true, minify: true },
    });
    const manifestBytes = await readFile(path.join(directory, "gallery/api/static.json"));
    const manifest = JSON.parse(manifestBytes.toString()) as StaticGalleryPayload;
    await writeFile(path.join(output, "static.json"), manifestBytes);
    return { root, directory, manifest };
  } catch (error) {
    await writeFile(
      path.join(output, "build-failure.json"),
      JSON.stringify(
        {
          message: String(error),
          stack: error instanceof Error ? error.stack : undefined,
          details: inspect(error, { depth: null, maxArrayLength: null, maxStringLength: null }),
        },
        null,
        2,
      ),
    );
    await rm(root, { recursive: true, force: true });
    throw error;
  }
}

/** Serve emitted bytes only; document navigation has the ordinary static SPA fallback. */
export async function hostNativeStaticGallery(directory: string) {
  const responses: { path: string; status: number; sha256?: string }[] = [];
  const mime: Record<string, string> = {
    ".html": "text/html",
    ".js": "text/javascript",
    ".css": "text/css",
    ".json": "application/json",
    ".ttf": "font/ttf",
  };
  const server = createServer(async (request, response) => {
    const url = new URL(request.url ?? "/", "http://127.0.0.1");
    try {
      if (!url.pathname.startsWith("/built/")) {
        response.writeHead(url.pathname === "/favicon.ico" ? 204 : 404).end();
        return;
      }
      const relative = decodeURIComponent(url.pathname.slice("/built/".length));
      let file = path.resolve(directory, relative);
      assert.ok(file.startsWith(`${directory}${path.sep}`));
      try {
        if ((await stat(file)).isDirectory()) file = path.join(file, "index.html");
      } catch {
        if (relative.startsWith("gallery/") && request.headers.accept?.includes("text/html"))
          file = path.join(directory, "gallery/index.html");
      }
      const bytes = await readFile(file);
      responses.push({ path: url.pathname, status: 200, sha256: sha256(bytes) });
      response
        .writeHead(200, {
          "Content-Type": mime[path.extname(file)] ?? "application/octet-stream",
          "Cache-Control": "no-store",
        })
        .end(bytes);
    } catch (error) {
      responses.push({ path: url.pathname, status: 404 });
      response.writeHead(404).end(String(error));
    }
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  return {
    origin: `http://127.0.0.1:${address.port}`,
    responses,
    close: () =>
      new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  };
}

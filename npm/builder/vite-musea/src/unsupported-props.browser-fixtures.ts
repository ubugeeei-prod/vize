import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createServer as createHttpServer, type Server } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build } from "vite";
import vize from "../../vite/src/index.ts";
import { compactPropsArtSource, compactPropsFixture } from "./compact-props-fixtures.ts";
import { musea } from "./plugin/index.ts";

export const vueAlias = fileURLToPath(import.meta.resolve("vue/dist/vue.runtime.esm-bundler.js"));
export const rawTitle = "Raw / Own Keys";
export const rawFilename = "unresolved-raw.art.vue";

export async function writeUnsupportedPropsFixture(root: string) {
  const fixture = await compactPropsFixture();
  const vector = fixture.browserCases.find((item) => item.names.includes("__proto__"))!;
  assert.ok(vector);
  await mkdir(path.join(root, "src"), { recursive: true });
  await writeFile(path.join(root, "src", vector.filename), vector.source);
  await writeFile(
    path.join(root, "src", vector.filename.replace(".vue", ".art.vue")),
    compactPropsArtSource(vector),
  );
  // No paired Vue component or component import: the raw inline bridge stays unflagged.
  await writeFile(
    path.join(root, "src", rawFilename),
    `<art title="${rawTitle}"><variant name="Default"><output>Raw bridge</output></variant></art>`,
  );
  await writeFile(path.join(root, "index.html"), "<!doctype html><html><body></body></html>");
  return { fixture, vector };
}

export async function buildUnsupportedPropsGallery(root: string) {
  await build({
    root,
    configFile: false,
    base: "/site/",
    resolve: { alias: { vue: vueAlias } },
    plugins: [vize(), musea({ include: ["src/**/*.art.vue"] })],
    build: { outDir: path.join(root, "dist"), emptyOutDir: true },
  });
  return JSON.parse(await readFile(path.join(root, "dist/__musea__/api/static.json"), "utf8"));
}

/** Serve emitted files directly, with no Vite server, API emulation, or SPA fallback. */
export function createUnsupportedPropsHost(directory: string): Server {
  const mime: Record<string, string> = {
    ".html": "text/html",
    ".js": "text/javascript",
    ".css": "text/css",
    ".json": "application/json",
    ".svg": "image/svg+xml",
    ".woff2": "font/woff2",
  };
  return createHttpServer(async (req, res) => {
    try {
      const pathname = decodeURIComponent(new URL(req.url!, "http://localhost").pathname);
      if (!pathname.startsWith("/site/")) {
        res.writeHead(404).end();
        return;
      }
      const relative = pathname.slice("/site/".length);
      const filename = path.resolve(
        directory,
        relative.endsWith("/") ? `${relative}index.html` : relative,
      );
      if (!filename.startsWith(`${path.resolve(directory)}${path.sep}`)) {
        res.writeHead(404).end();
        return;
      }
      const data = await readFile(filename);
      res.setHeader("Content-Type", mime[path.extname(filename)] ?? "application/octet-stream");
      res.end(data);
    } catch {
      res.writeHead(404).end();
    }
  });
}

export async function listenUnsupportedPropsHost(host: Server): Promise<string> {
  await new Promise<void>((resolve, reject) => {
    host.once("error", reject);
    host.listen(0, "127.0.0.1", resolve);
  });
  const address = host.address();
  assert.ok(address && typeof address !== "string");
  return `http://127.0.0.1:${address.port}`;
}

export function closeUnsupportedPropsHost(host: Server | undefined): Promise<void> {
  if (!host?.listening) return Promise.resolve();
  return new Promise((resolve, reject) =>
    host.close((error) => (error ? reject(error) : resolve())),
  );
}

/** Compile the exact clipboard SFC through the real native Vite plugin. */
export async function buildUnsupportedPropsCopy(root: string, copied: string) {
  await writeFile(path.join(root, "src", "Clipboard.vue"), copied);
  await writeFile(
    path.join(root, "clipboard.js"),
    `import {createApp} from 'vue'; import Clipboard from './src/Clipboard.vue';
createApp(Clipboard).mount('#app');`,
  );
  await writeFile(
    path.join(root, "clipboard.html"),
    '<!doctype html><html><body><main id="app"></main><script type="module" src="./clipboard.js"></script></body></html>',
  );
  await build({
    root,
    configFile: false,
    base: "/site/",
    resolve: { alias: { vue: vueAlias } },
    plugins: [vize()],
    build: {
      outDir: path.join(root, "dist"),
      emptyOutDir: false,
      rollupOptions: { input: path.join(root, "clipboard.html") },
    },
  });
}

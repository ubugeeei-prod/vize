import assert from "node:assert/strict";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { createServer } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { build, type Plugin, type ResolvedConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { generatePreviewModule } from "../../src/preview/index.ts";
import {
  emitStaticGallery,
  loadStaticRuntimeModule,
  museaStaticBuildConfig,
  resolveStaticRuntimeId,
} from "../../src/static-export.ts";
import type { ArtFileInfo } from "../../src/types/index.ts";

const packageRoot = fileURLToPath(new URL("../../", import.meta.url));
export const hostedBase = "/site/__musea__";

export async function buildHostedGallery(
  output: string,
  setupDelayMs?: number,
  additionalArts: ArtFileInfo[] = [],
) {
  const artPath = path.join(output, "Host.art.vue");
  const fixture: { setupDelayMs: number; variants: string[] } = JSON.parse(
    await readFile(
      new URL(
        "../../../../../tests/_fixtures/differential/musea/hosted-audits.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  const names = fixture.variants;
  const art: ArtFileInfo = {
    path: artPath,
    metadata: { title: "Hosted", tags: [], status: "ready" },
    variants: names.map((name, index) => ({
      name,
      template: "<main />",
      isDefault: index === 0,
      skipVrt: false,
    })),
    hasScriptSetup: false,
    hasScript: false,
    styleCount: 0,
  };
  await mkdir(output, { recursive: true });
  await writeFile(artPath, '<art><variant name="Clean one"><main /></variant></art>');
  await build({
    configFile: false,
    root: path.join(packageRoot, "gallery"),
    base: "/__musea__/",
    plugins: [vue()],
    logLevel: "silent",
    build: { outDir: path.join(packageRoot, "dist/gallery"), emptyOutDir: true },
  });
  const arts = new Map([
    [artPath, art],
    ...additionalArts.map((extra) => [extra.path, extra] as const),
  ]);
  let config: ResolvedConfig;
  const previewPrefix = "virtual:musea-preview:";
  const setupId = "virtual:hosted-setup";
  const buildFixture: Plugin = {
    name: "musea-hosted-browser-fixture",
    config: () => museaStaticBuildConfig(),
    configResolved(resolved) {
      config = resolved;
    },
    resolveId(id) {
      if (id === setupId || id.startsWith(previewPrefix) || id.startsWith("virtual:musea-art:"))
        return `\0${id}`;
      return resolveStaticRuntimeId(id);
    },
    load(id) {
      if (id === `\0${setupId}`) {
        return `export default async function() {
          await new Promise(resolve => setTimeout(resolve, ${setupDelayMs ?? fixture.setupDelayMs}));
          window.__hostedSetupAfterLoad = document.readyState === 'complete';
        }`;
      }
      if (id.startsWith("\0virtual:musea-art:")) {
        // Native compilation is outside this contract. Real emitted HTML,
        // dynamic preview modules, async setup, Vue and axe run on an HTTP host.
        return (
          `import { h } from 'vue';` +
          (arts.get(id.slice("\0virtual:musea-art:".length))?.variants ?? art.variants)
            .map((variant) => {
              const name = variant.name;
              const identifier = name
                .replace(/\b\w/g, (letter) => letter.toUpperCase())
                .replaceAll(" ", "");
              const child = additionalArts.some(
                (extra) => extra.path === id.slice("\0virtual:musea-art:".length),
              )
                ? `h('div', { style: ${JSON.stringify(variant.template.match(/style="([^"]+)"/)?.[1] ?? "")} }, ${JSON.stringify(name)})`
                : name.startsWith("Clean")
                  ? `h('button', { type: 'button' }, 'Accessible action')`
                  : `h('button', { type: 'button' })`;
              return `export const ${identifier} = { render() { return h('main', {}, [${child}]); } };`;
            })
            .join("\n")
        );
      }
      if (id.startsWith(`\0${previewPrefix}`)) {
        const artAndVariant = id.slice(`\0${previewPrefix}`.length);
        const previewArt = [...arts.values()].find((candidate) =>
          artAndVariant.startsWith(`${candidate.path}:`),
        );
        if (!previewArt) throw new Error(`Unknown fixture preview: ${artAndVariant}`);
        const name = artAndVariant.slice(previewArt.path.length + 1);
        const identifier = name
          .replace(/\b\w/g, (letter) => letter.toUpperCase())
          .replaceAll(" ", "");
        return generatePreviewModule(previewArt, identifier, name, [], setupId);
      }
      return loadStaticRuntimeModule(id, arts);
    },
    async generateBundle(_options, bundle) {
      await emitStaticGallery(
        (asset) => {
          this.emitFile(asset);
        },
        bundle,
        {
          config,
          projectRoot: output,
          artFiles: arts,
          scanRoots: [output],
          tokensPath: undefined,
          basePath: "/__musea__",
          resolvedPreviewCss: [],
          resolvedPreviewSetup: setupId,
          devSessionToken: "hosted-fixture",
          themeConfig: undefined,
        },
      );
    },
  };
  const dist = path.join(output, "dist");
  await build({
    configFile: false,
    root: packageRoot,
    base: "/site/",
    plugins: [buildFixture],
    logLevel: "silent",
    build: { outDir: dist, emptyOutDir: true },
  });
  let refuseAxe = false;
  const requests: string[] = [];
  const server = createServer((request, response) => {
    const url = new URL(request.url || "/", "http://hosted.invalid");
    requests.push(url.pathname);
    const relative = decodeURIComponent(url.pathname.replace(/^\/site\//, ""));
    const candidate = path.resolve(dist, relative);
    void (async () => {
      if (!candidate.startsWith(`${dist}${path.sep}`)) throw new Error("Foreign path");
      if (refuseAxe && relative.endsWith("/vendor/axe-core.min.js")) throw new Error("Missing axe");
      let content: Buffer;
      let extension = path.extname(candidate);
      try {
        content = await readFile(candidate);
      } catch {
        if (extension) throw new Error("Missing asset");
        content = await readFile(path.join(dist, "__musea__/index.html"));
        extension = ".html";
      }
      const types: Record<string, string> = {
        ".html": "text/html",
        ".js": "text/javascript",
        ".css": "text/css",
        ".json": "application/json",
      };
      response.setHeader("Content-Type", types[extension] || "application/octet-stream");
      response.setHeader("Cache-Control", "no-store");
      response.end(content);
    })().catch(() => {
      response.statusCode = 404;
      response.end("Not found");
    });
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  assert.ok(address && typeof address !== "string");
  return {
    url: `http://127.0.0.1:${address.port}${hostedBase}`,
    requests,
    art,
    dist,
    refuseAxe() {
      refuseAxe = true;
    },
    close: () =>
      new Promise<void>((resolve, reject) =>
        server.close((error) => (error ? reject(error) : resolve())),
      ),
  };
}

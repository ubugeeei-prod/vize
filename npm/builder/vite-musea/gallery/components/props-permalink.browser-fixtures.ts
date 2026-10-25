import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import path from "node:path";
import type { BrowserContext } from "playwright";
import { createServer, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import { generatePreviewHtml, generatePreviewModule } from "../../src/preview/index.ts";
import type { ArtFileInfo } from "../../src/types/index.ts";
import { normalizeToolbar } from "../../src/toolbar.ts";

export const base = "/docs/design-system";
export const defaults = { label: "Authored", tone: "brand", config: { nested: [false, null, 42] } };
export const art: ArtFileInfo = {
  path: "components/日本語 🎨.art.vue",
  metadata: { title: "Props permalink", component: "PropsProbe", tags: [], status: "ready" },
  variants: [{ name: "Default", template: "<PropsProbe />", isDefault: true, skipVrt: true }],
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
};
const previewId = "virtual:musea-permalink-preview";
const artId = `virtual:musea-art:${art.path}`;
const toolbar = normalizeToolbar([
  { id: "theme", title: "Theme", type: "select", options: ["light", "dark"], default: "light" },
]);

export function createPermalinkServer(output: string) {
  const fixtures: Plugin = {
    name: "musea-props-permalink-fixtures",
    enforce: "pre",
    transformIndexHtml() {
      return [
        {
          tag: "script",
          injectTo: "head-prepend",
          children: `window.__MUSEA_BASE_PATH__=${JSON.stringify(base)};window.__MUSEA_TOOLBAR__=${JSON.stringify(toolbar)};`,
        },
      ];
    },
    resolveId(id) {
      if ([previewId, artId].includes(id)) return `\0${id}`;
    },
    load(id) {
      if (id === `\0${previewId}`) return generatePreviewModule(art, "Default", "Default");
      if (id === `\0${artId}`) {
        return `import { h, toRaw } from 'vue';
          const Probe = {
            inheritAttrs: false,
            props: ['label', 'tone', 'config'],
            setup(props, { attrs, slots }) {
              return () => h('div', [
                h('output', JSON.stringify({ props: Object.fromEntries(Object.entries(toRaw(props)).filter(([, value]) => value !== undefined)), attrs: { ...attrs } })),
                h('section', { id: 'slot-content' }, slots.default?.())
              ]);
            }
          };
          export const Default = Probe;
          export const __component__ = Probe;`;
      }
    },
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const url = new URL(request.url || "/", "http://fixture.invalid");
        if (!url.pathname.startsWith(`${base}/preview`)) return next();
        void (async () => {
          if (url.pathname === `${base}/preview-module`) {
            const transformed = await server.transformRequest(previewId);
            assert.ok(transformed);
            response.setHeader("Content-Type", "text/javascript");
            response.end(transformed.code);
          } else {
            response.setHeader("Content-Type", "text/html");
            response.end(generatePreviewHtml(art, art.variants[0], base, `${base}/`));
          }
        })().catch(next);
      });
    },
  };
  return createServer({
    configFile: false,
    root: fileURLToPath(new URL("../", import.meta.url)),
    base: `${base}/`,
    plugins: [fixtures, vue()],
    server: { host: "127.0.0.1", port: 0 },
    cacheDir: path.join(output, "vite-cache"),
  });
}

export async function preparePermalinkContext(context: BrowserContext) {
  await context.route(`**${base}/api/**`, async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    const data = pathname.endsWith("/arts")
      ? [art]
      : pathname.endsWith("/palette")
        ? {
            title: "PropsProbe",
            groups: [],
            json: "",
            typescript: "",
            controls: Object.entries(defaults).map(([name, value]) => ({
              name,
              default_value: value,
              control: typeof value === "string" ? "text" : "object",
              required: false,
              options: [],
            })),
          }
        : pathname.endsWith("/analysis")
          ? { props: [], emits: [] }
          : pathname.endsWith("/tokens")
            ? { categories: [], tokenMap: {}, meta: {} }
            : pathname.endsWith("/docs")
              ? { markdown: "", title: "PropsProbe", variant_count: 1 }
              : art;
    await route.fulfill({ json: data });
  });
}

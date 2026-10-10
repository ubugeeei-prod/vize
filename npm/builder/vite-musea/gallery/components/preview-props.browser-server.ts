import assert from "node:assert/strict";
import { fileURLToPath } from "node:url";
import path from "node:path";
import type { BrowserContext } from "playwright";
import { createServer, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import {
  generatePreviewHtml,
  generatePreviewModule,
  generatePreviewModuleWithProps,
} from "../../src/preview/index.ts";
import type { ArtFileInfo } from "../../src/types/index.ts";

const base = "/__musea__";
const previewId = "virtual:musea-props-browser-preview";
const staticId = "virtual:musea-props-browser-static";
const vueId = "virtual:musea-props-browser-vue";
export const art: ArtFileInfo = {
  path: "PropsProbe.art.vue",
  metadata: { title: "PropsProbe", component: "PropsProbe", tags: [], status: "ready" },
  variants: [{ name: "Default", template: "<PropsProbe />", isDefault: true, skipVrt: true }],
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
};
const artId = `virtual:musea-art:${art.path}`;

function fixtures(values: Record<string, unknown>): Plugin {
  return {
    name: "musea-generated-props-browser-fixtures",
    enforce: "pre",
    transformIndexHtml() {
      return [
        {
          tag: "script",
          injectTo: "head-prepend",
          children: `window.__MUSEA_BASE_PATH__=${JSON.stringify(base)};`,
        },
      ];
    },
    resolveId(id, importer) {
      if (id === "vue" && (importer === `\0${previewId}` || importer === `\0${staticId}`))
        return `\0${vueId}`;
      if ([previewId, staticId, vueId, artId].includes(id)) return `\0${id}`;
    },
    load(id) {
      if (id === `\0${previewId}`) return generatePreviewModule(art, "Default", "Default");
      if (id === `\0${staticId}`)
        return generatePreviewModuleWithProps(art, "Default", "Default", values);
      if (id === `\0${vueId}`) {
        // Observe the real Vue proxy used by the unchanged generated module.
        // This wrapper never substitutes rendering, messages or store updates.
        return `export { createApp } from 'vue';
          import { h as realH, shallowRef as realShallowRef, reactive as realReactive } from 'vue';
          export function h(component, props, ...args) {
            if (props) window.__propsStaticStore = props;
            return realH(component, props, ...args);
          }
          function capture(proxy, isRef) {
            if (!Object.hasOwn(window, '__propsBrowserStore')) {
              Object.defineProperty(window, '__propsBrowserStore', {
                get: () => isRef ? proxy.value : proxy
              });
              window.__propsBrowserPrototype = Object.getPrototypeOf(isRef ? proxy.value : proxy);
            }
            return proxy;
          }
          export function shallowRef(value) {
            return capture(realShallowRef(value), true);
          }
          export function reactive(value) {
            return capture(realReactive(value), false);
          }`;
      }
      if (id === `\0${artId}`) {
        // Authored art-module fixture: native SFC compilation is a separate gate.
        return `import { h, toRaw } from 'vue';
          const Probe = {
            inheritAttrs: false,
            props: ['label', 'config', 'constructor', 'hasOwnProperty', 'custom'],
            setup(props, { attrs }) {
              return () => h('output', JSON.stringify({
                props: Object.fromEntries(Object.entries(toRaw(props)).filter(([, value]) => value !== undefined)),
                attrs: { ...attrs }
              }));
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
            const transformed = await server.transformRequest(
              url.searchParams.get("static") === "1" ? staticId : previewId,
            );
            assert.ok(transformed);
            response.setHeader("Content-Type", "text/javascript");
            response.end(transformed.code);
          } else {
            response.setHeader("Content-Type", "text/html");
            let html = generatePreviewHtml(art, art.variants[0], base, `${base}/`);
            if (url.searchParams.get("static") === "1")
              html = html.replace("&variant=Default", "&variant=Default&static=1");
            response.end(html);
          }
        })().catch(next);
      });
    },
  };
}

export function createPropsBrowserServer(output: string, values: Record<string, unknown>) {
  return createServer({
    configFile: false,
    root: fileURLToPath(new URL("../", import.meta.url)),
    base: `${base}/`,
    plugins: [fixtures(values), vue()],
    server: { host: "127.0.0.1", port: 0 },
    cacheDir: path.join(output, "preview-props-vite-cache"),
  });
}

export async function preparePropsContext(
  context: BrowserContext,
  values: Record<string, unknown>,
) {
  const palette = {
    title: "PropsProbe",
    groups: [],
    json: "",
    typescript: "",
    controls: Object.entries(values).map(([name, value]) => ({
      name,
      default_value: value,
      control: typeof value === "string" ? "text" : "object",
      required: false,
      options: [],
    })),
  };
  await context.route("**/__musea__/api/**", async (route) => {
    const name = new URL(route.request().url()).pathname;
    const data = name.endsWith("/arts")
      ? [art]
      : name.endsWith("/palette")
        ? palette
        : name.endsWith("/analysis")
          ? { props: [], emits: [] }
          : name.endsWith("/tokens")
            ? { categories: [], tokenMap: {}, meta: {} }
            : name.endsWith("/docs")
              ? { markdown: "", title: "PropsProbe", variant_count: 1 }
              : art;
    await route.fulfill({ json: data });
  });
}

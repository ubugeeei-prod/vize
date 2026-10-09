import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath } from "node:url";
import type { BrowserContext, Page } from "playwright";
import { createServer, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import { stringifyQuery } from "vue-router";
import { generatePreviewHtml, generatePreviewModule } from "../../src/preview/index.ts";
import { normalizeToolbar } from "../../src/toolbar.ts";
import type { ArtFileInfo } from "../../src/types/index.ts";

const gallery = fileURLToPath(new URL("../", import.meta.url));
export const basePath = "/__musea__";
const toolbar = normalizeToolbar([
  { id: "theme", title: "Theme", type: "select", options: ["light", "dark"], default: "light" },
  {
    id: "locale",
    title: "Locale",
    type: "select",
    default: "en",
    options: [
      { value: "en", label: "English" },
      { value: "fr", label: "French" },
    ],
  },
  {
    id: "rtl",
    title: "Direction",
    type: "toggle",
    default: false,
    options: [
      { value: false, label: "LTR" },
      { value: true, label: "RTL" },
    ],
  },
]);
export const defaults = { theme: "light", locale: "en", rtl: false };
export const changed = { theme: "dark", locale: "fr", rtl: true };
const arts: ArtFileInfo[] = ["First", "Second"].map((title) => ({
  path: `${title}.art.vue`,
  metadata: { title, tags: [], status: "ready" },
  variants: ["Default", "Alternate"].map((name, index) => ({
    name,
    template: "<Globals />",
    isDefault: index === 0,
    skipVrt: true,
  })),
  hasScriptSetup: false,
  hasScript: false,
  styleCount: 0,
}));
const setupId = "virtual:musea-toolbar-browser-setup";
const previewPrefix = "virtual:musea-toolbar-browser-preview:";
const artPrefix = "virtual:musea-art:";

function fixtures(): Plugin {
  return {
    name: "musea-global-toolbar-browser-fixtures",
    transformIndexHtml() {
      return [
        {
          tag: "script",
          injectTo: "head-prepend",
          children: `window.__MUSEA_BASE_PATH__=${JSON.stringify(basePath)};window.__MUSEA_TOOLBAR__=${JSON.stringify(toolbar)};`,
        },
      ];
    },
    resolveId(id) {
      if (id === setupId || id.startsWith(previewPrefix) || id.startsWith(artPrefix))
        return `\0${id}`;
    },
    load(id) {
      if (id === `\0${setupId}`) {
        return `export default function(app, { globals }) {
          if (window.__toolbarReference && window.__toolbarReference !== globals) {
            throw new Error('previewSetup globals Ref changed during remount');
          }
          if (!window.__toolbarSetupCalls) window.__toolbarFirstSetupGlobals = { ...globals.value };
          window.__toolbarReference = globals;
          app.provide('browser-globals', globals);
          window.__toolbarSetupCalls = (window.__toolbarSetupCalls || 0) + 1;
        }`;
      }
      if (id.startsWith(`\0${artPrefix}`)) {
        // Explicit art-module fixtures: the actual generated preview, Vue apps,
        // setup hook and message runtime run here; native compilation does not.
        return `import { h, inject, isRef } from 'vue';
          const Globals = { props: ['test'], setup() {
            const globals = inject('browser-globals');
            return () => h('output', {
              'data-theme': globals.value.theme, 'data-locale': globals.value.locale,
              'data-rtl': String(globals.value.rtl), 'data-is-ref': String(isRef(globals))
            }, JSON.stringify(globals.value));
          }};
          export const Default = Globals;
          export const Alternate = Globals;
          export const __component__ = Globals;`;
      }
      if (id.startsWith(`\0${previewPrefix}`)) {
        const query = new URLSearchParams(id.slice(previewPrefix.length + 1));
        const art = arts.find((candidate) => candidate.path === query.get("art"));
        assert.ok(art);
        return generatePreviewModule(
          art,
          query.get("variant")!,
          query.get("variant")!,
          [],
          setupId,
          3,
          toolbar,
        );
      }
    },
    configureServer(server) {
      server.middlewares.use((request, response, next) => {
        const url = new URL(request.url || "/", "http://fixture.invalid");
        if (!url.pathname.startsWith(`${basePath}/preview`)) return next();
        void (async () => {
          if (url.pathname === `${basePath}/preview-module`) {
            const transformed = await server.transformRequest(
              `${previewPrefix}${url.searchParams}`,
            );
            assert.ok(transformed);
            response.setHeader("Content-Type", "text/javascript");
            response.end(transformed.code);
          } else {
            const art = arts.find((candidate) => candidate.path === url.searchParams.get("art"));
            assert.ok(art);
            const variant = art.variants.find(
              (candidate) => candidate.name === url.searchParams.get("variant"),
            );
            assert.ok(variant);
            response.setHeader("Content-Type", "text/html");
            response.end(generatePreviewHtml(art, variant, basePath, `${basePath}/`));
          }
        })().catch(next);
      });
    },
  };
}

export async function prepareContext(context: BrowserContext) {
  // Literal browser code keeps tsx's Node-only function-name helpers out of Chromium.
  await context.addInitScript({
    content: `(() => {
    // Chromium may re-run the context initializer in a popup's existing realm.
    // Retain the original inventory and wrappers when that realm is initialized.
    if (Object.hasOwn(window, "__toolbarMessageListenerNames")) return;
    const listeners = new Set();
    for (const method of ["addEventListener", "removeEventListener"]) {
      const original = window[method];
      Object.defineProperty(window, method, {
        configurable: true, enumerable: true, writable: true,
        value(...args) {
          const result = Reflect.apply(original, this, args);
          if (this === window && args[0] === "message" && args[1] != null) {
            if (method === "addEventListener") listeners.add(args[1]);
            else listeners.delete(args[1]);
          }
          return result;
        }
      });
    }
    Object.defineProperty(window, "__toolbarMessageListenerCount", {
      configurable: true,
      get() { return listeners.size; }
    });
    Object.defineProperty(window, "__toolbarMessageListenerNames", {
      configurable: true,
      get() {
        return [...listeners].map((listener) =>
          typeof listener === "function" ? listener.name || "(anonymous)"
            : listener?.handleEvent?.name || "(object listener)"
        ).sort();
      }
    });
  })();`,
  });
  // The gallery API supplies explicit fixtures, independently of Rust parsing.
  await context.route("**/__musea__/api/**", async (route) => {
    const url = new URL(route.request().url());
    const data = url.pathname.endsWith("/arts")
      ? arts
      : url.pathname.endsWith("/palette")
        ? { title: "Globals", groups: [], controls: [], json: "", typescript: "" }
        : url.pathname.endsWith("/analysis")
          ? { props: [], emits: [] }
          : arts.find((art) => art.path === url.searchParams.get("path")) || arts[0];
    await route.fulfill({ json: data });
  });
}

export function createFixtureServer(output: string) {
  return createServer({
    configFile: false,
    root: gallery,
    base: `${basePath}/`,
    plugins: [fixtures(), vue()],
    server: { host: "127.0.0.1", port: 0 },
    cacheDir: path.join(output, "vite-cache"),
  });
}

export async function checkOpenedPreview(
  context: BrowserContext,
  page: Page,
  expected: typeof defaults,
) {
  const pending = context.waitForEvent("page");
  await page
    .locator(".variant-card")
    .first()
    .getByTitle("Open in new tab", { exact: true })
    .click();
  const opened = await pending;
  const errors: string[] = [];
  opened.on("pageerror", (error) => errors.push(String(error)));
  try {
    await opened.waitForFunction((values) => {
      const element = document.querySelector("output");
      return (
        element?.getAttribute("data-is-ref") === "true" &&
        element.getAttribute("data-theme") === values.theme &&
        element.getAttribute("data-locale") === values.locale &&
        element.getAttribute("data-rtl") === String(values.rtl)
      );
    }, expected);
    const result = await opened.evaluate(() => ({
      url: location.href,
      globals: JSON.parse(document.querySelector("output")!.textContent!),
      firstSetupGlobals: Reflect.get(window, "__toolbarFirstSetupGlobals"),
      setupCalls: Reflect.get(window, "__toolbarSetupCalls"),
    }));
    assert.deepEqual(JSON.parse(new URL(result.url).searchParams.get("museaGlobals")!), expected);
    assert.deepEqual(result.globals, expected);
    assert.deepEqual(
      result.firstSetupGlobals,
      expected,
      "New-tab setup receives the current globals before mounting",
    );
    assert.equal(result.setupCalls, 1);
    assert.deepEqual(errors, []);
    return result;
  } finally {
    await opened.close();
  }
}

export async function checkMobileToolbar(page: Page) {
  const measured = await page.locator(".addon-toolbar").evaluate((element) => {
    const bounds = element.getBoundingClientRect();
    return {
      left: bounds.left,
      right: bounds.right,
      viewport: window.innerWidth,
      clientWidth: element.clientWidth,
      scrollWidth: element.scrollWidth,
    };
  });
  assert.ok(
    measured.clientWidth > 0 && measured.left >= 0 && measured.right <= measured.viewport,
    JSON.stringify(measured),
  );
  assert.ok(measured.scrollWidth <= measured.clientWidth, JSON.stringify(measured));
  return measured;
}

export async function openHashComponent(page: Page) {
  const href = await page.locator(".art-item").first().getAttribute("href");
  assert.ok(href);
  const url = new URL(href, page.url());
  url.search = stringifyQuery({
    ...Object.fromEntries(url.searchParams),
    museaGlobals: JSON.stringify(defaults),
  });
  url.hash = "#variant-default";
  // A genuine navigation lets the production router establish its hash/history state.
  await page.goto(url.href);
  await page.waitForURL((actual) => actual.href === url.href);
  await page.getByRole("heading", { name: "First", exact: true }).waitFor();
  assert.equal(new URL(page.url()).hash, "#variant-default");
}

export async function checkHashToolbarChanges(page: Page) {
  const retainedHash = new URL(page.url()).hash;
  assert.equal(retainedHash, "#variant-default");
  let values = { ...defaults };
  const observations = [];
  for (const step of [
    { id: "theme", title: "Theme", value: "dark" },
    { id: "locale", title: "Locale", value: "fr" },
    { id: "rtl", title: "Direction", value: true },
  ] as const) {
    values = { ...values, [step.id]: step.value };
    const expectedUrl = new URL(page.url());
    expectedUrl.search = stringifyQuery({
      ...Object.fromEntries(expectedUrl.searchParams),
      museaGlobals: JSON.stringify(values),
    });
    if (step.id === "rtl") {
      await page
        .getByRole("group", { name: step.title, exact: true })
        .getByRole("button", { name: "RTL", exact: true })
        .click();
    } else {
      await page.getByLabel(step.title, { exact: true }).selectOption({ index: 1 });
    }
    await page.waitForURL((actual) => actual.href === expectedUrl.href);
    const actual = new URL(page.url());
    assert.equal(actual.hash, retainedHash);
    const globals = JSON.parse(actual.searchParams.get("museaGlobals")!);
    assert.deepEqual(globals, values);
    observations.push({ control: step.title, url: actual.href, hash: actual.hash, globals });
  }
  return observations;
}

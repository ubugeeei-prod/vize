import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { VizePluginState } from "./state.ts";
import { getBoundaryPlaceholderCode } from "./load.ts";
import { loadHook } from "./load.ts";
import { transformHook } from "./load.ts";
import { toVirtualId } from "../virtual.ts";

const ssrClientPlaceholder = getBoundaryPlaceholderCode("/src/Foo.client.vue", true);
assert.ok(ssrClientPlaceholder, "SSR should stub .client.vue components");
assert.match(
  ssrClientPlaceholder,
  /createElementBlock\("div"\)/,
  "SSR .client.vue placeholder should render a simple div",
);

const clientServerPlaceholder = getBoundaryPlaceholderCode("/src/Foo.server.vue", false);
assert.ok(clientServerPlaceholder, "Client build should stub .server.vue components");
assert.match(
  clientServerPlaceholder,
  /ServerPlaceholder/,
  "Client .server.vue placeholder should use the server placeholder component",
);

assert.equal(
  getBoundaryPlaceholderCode("/src/Foo.client.vue", false),
  null,
  "Client build must not stub .client.vue components",
);
assert.equal(
  getBoundaryPlaceholderCode("/src/Foo.server.vue", true),
  null,
  "SSR build must not stub .server.vue components",
);
assert.equal(
  getBoundaryPlaceholderCode("/src/Foo.vue", true),
  null,
  "Regular SFCs must not be stubbed",
);

const realPath = "/src/Hmr.vue";
const hmrState: VizePluginState = {
  cache: new Map([
    [
      realPath,
      {
        code: `export function render() { return null }
const _sfc_main = {}
_sfc_main.render = render
export default _sfc_main`,
        scopeId: "hmr12345",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map([[realPath, "template-only"]]),
  isProduction: false,
  root: "/src",
  clientViteBase: "/",
  serverViteBase: "/",
  server: {} as never,
  filter: () => true,
  scanPatterns: ["**/*.vue"],
  ignorePatterns: [],
  mergedOptions: {},
  initialized: true,
  dynamicImportAliasRules: [],
  cssAliasRules: [],
  extractCss: false,
  clientViteDefine: {},
  serverViteDefine: {},
  logger: {
    log() {},
    info() {},
    warn() {},
    error() {},
  } as never,
};

const virtualDefineState: VizePluginState = {
  ...hmrState,
  clientViteDefine: {
    "import.meta.client": "true",
    "import.meta.server": "false",
    "import.meta.dev": "true",
  },
};

const virtualDefineTransform = await transformHook(
  virtualDefineState,
  `export const flags = [import.meta.client, import.meta.server, import.meta.dev, import.meta.hot];`,
  toVirtualId("/src/EnvFlags.vue"),
  { ssr: false },
);

assert.ok(
  virtualDefineTransform && typeof virtualDefineTransform === "object",
  "Virtual module transforms should succeed",
);
assert.doesNotMatch(
  virtualDefineTransform.code,
  /import\.meta\.(client|server|dev)/,
  "Virtual module transforms should inline environment flags that Vite skips for \\0 IDs",
);
assert.match(
  virtualDefineTransform.code,
  /import\.meta\.hot/,
  "Virtual module transforms must leave import.meta.hot available for Vite HMR",
);

const definePageDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-define-page-"));
const definePagePath = path.join(definePageDir, "Home.vue");
fs.writeFileSync(
  definePagePath,
  `<script setup lang="ts">
definePage({
  name: "home",
  meta: { requiresAuth: true },
})

const msg = "ready"
</script>
<template><div>{{ msg }}</div></template>`,
);

const definePageLoad = loadHook(
  { ...hmrState, cache: new Map(), ssrCache: new Map(), root: definePageDir },
  `\0${definePagePath}?definePage`,
  { ssr: false },
);

assert.ok(
  definePageLoad && typeof definePageLoad === "object",
  "Vue Router definePage queries should load as code objects",
);
assert.match(
  definePageLoad.code,
  /export default \{/,
  "Vue Router definePage queries should return the extracted route record module",
);
assert.doesNotMatch(
  definePageLoad.code,
  /const msg/,
  "Vue Router definePage queries should not return the component setup body",
);

const definePageMetaDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-define-page-meta-"));
const definePageMetaPath = path.join(definePageMetaDir, "Docs.vue");
fs.writeFileSync(
  definePageMetaPath,
  `<script setup lang="ts">
definePageMeta({
  name: "docs",
  meta: { scrollMargin: 180 },
})

const msg = "ready"
</script>
<template><div>{{ msg }}</div></template>`,
);

const definePageMetaLoad = loadHook(
  { ...hmrState, cache: new Map(), ssrCache: new Map(), root: definePageMetaDir },
  `\0${definePageMetaPath}?macro=true`,
  { ssr: false },
);

assert.ok(
  definePageMetaLoad && typeof definePageMetaLoad === "object",
  "Nuxt definePageMeta macro queries should load as code objects",
);
assert.match(
  definePageMetaLoad.code,
  /export default \{/,
  "Nuxt definePageMeta macro queries should return the extracted page metadata module",
);
assert.match(
  definePageMetaLoad.code,
  /scrollMargin/,
  "Nuxt definePageMeta macro queries should preserve page metadata",
);
assert.doesNotMatch(
  definePageMetaLoad.code,
  /const msg/,
  "Nuxt definePageMeta macro queries should not return the component setup body",
);

const firstLoad = loadHook(hmrState, toVirtualId(realPath), { ssr: false });
assert.ok(firstLoad && typeof firstLoad === "object", "Virtual module should load as code object");
assert.match(
  firstLoad.code,
  /__hmrUpdateType = "template-only"/,
  "Pending template-only HMR updates must stay granular when render is exposed",
);
assert.equal(
  hmrState.pendingHmrUpdateTypes.has(realPath),
  false,
  "Pending HMR updates should be consumed after the client load",
);

const secondLoad = loadHook(hmrState, toVirtualId(realPath), { ssr: false });
assert.ok(
  secondLoad && typeof secondLoad === "object",
  "Subsequent virtual module loads should still succeed",
);
assert.match(
  secondLoad.code,
  /__hmrUpdateType = "full-reload"/,
  "Consumed pending updates must fall back to the default HMR mode",
);

const inlinePath = "/src/InlineHmr.vue";
const inlineState: VizePluginState = {
  ...hmrState,
  cache: new Map([
    [
      inlinePath,
      {
        code: `export default {
  __name: "InlineHmr",
  setup() {
    return (_ctx, _cache) => null
  },
}`,
        scopeId: "inline1234",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map(),
  pendingHmrUpdateTypes: new Map([[inlinePath, "template-only"]]),
};

const inlineLoad = loadHook(inlineState, toVirtualId(inlinePath), {
  ssr: false,
});
assert.ok(
  inlineLoad && typeof inlineLoad === "object",
  "Inline-template virtual modules should load as code objects",
);
assert.match(
  inlineLoad.code,
  /__hmrUpdateType = "full-reload"/,
  "Inline-template components must downgrade template-only HMR to full-reload",
);

const envPath = "/src/Environment.vue";
const environmentState: VizePluginState = {
  ...hmrState,
  cache: new Map([
    [
      envPath,
      {
        code: `export default { __name: "ClientCompiled" }`,
        scopeId: "clientenv",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map([
    [
      envPath,
      {
        code: `export default { __name: "ServerCompiled" }`,
        scopeId: "serverenv",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  pendingHmrUpdateTypes: new Map(),
};

const clientEnvironmentLoad = loadHook(environmentState, toVirtualId(envPath), {
  ssr: false,
});
assert.ok(
  clientEnvironmentLoad && typeof clientEnvironmentLoad === "object",
  "Client environment loads should succeed",
);
assert.match(
  clientEnvironmentLoad.code,
  /ClientCompiled/,
  "Client loads should read from the client compilation cache",
);

const ssrEnvironmentLoad = loadHook(environmentState, toVirtualId(envPath, true), { ssr: true });
assert.ok(
  ssrEnvironmentLoad && typeof ssrEnvironmentLoad === "object",
  "SSR environment loads should succeed",
);
assert.match(
  ssrEnvironmentLoad.code,
  /ServerCompiled/,
  "SSR loads should read from the SSR compilation cache",
);

const cssPath = "/src/SsrStyles.vue";
const cssState: VizePluginState = {
  ...hmrState,
  cache: new Map([
    [
      cssPath,
      {
        code: `export default { __name: "ClientCss" }`,
        css: ".demo { color: tomato; }",
        scopeId: "clientcss",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
  ssrCache: new Map([
    [
      cssPath,
      {
        code: `export default { __name: "ServerCss" }`,
        css: ".demo { color: tomato; }",
        scopeId: "servercss",
        hasScoped: false,
        styles: [],
      },
    ],
  ]),
};

const clientCssLoad = loadHook(cssState, toVirtualId(cssPath), { ssr: false });
assert.ok(clientCssLoad && typeof clientCssLoad === "object", "Client CSS load should succeed");
assert.match(
  clientCssLoad.code,
  /__vize_css__/,
  "Client loads should keep inline CSS injection in development",
);

const ssrCssLoad = loadHook(cssState, toVirtualId(cssPath, true), {
  ssr: true,
});
assert.ok(ssrCssLoad && typeof ssrCssLoad === "object", "SSR CSS load should succeed");
assert.doesNotMatch(
  ssrCssLoad.code,
  /__vize_css__/,
  "SSR loads should not inject client-only CSS runtime shims",
);
assert.doesNotMatch(
  ssrCssLoad.code,
  /document\.createElement/,
  "SSR loads should stay free of document-based side effects",
);

const cssModulePath = "/src/ModuleButton.vue";
const cssModuleState: VizePluginState = {
  ...hmrState,
  cache: new Map([
    [
      cssModulePath,
      {
        code: `const _sfc_main = { name: "ModuleButton" }
export default _sfc_main`,
        scopeId: "modulecss",
        hasScoped: false,
        styles: [
          {
            content: ".root { color: red; }",
            lang: "css",
            scoped: false,
            module: "buttonStyles",
            index: 0,
          },
        ],
      },
    ],
  ]),
  ssrCache: new Map(),
};

const cssModuleLoad = loadHook(cssModuleState, toVirtualId(cssModulePath), {
  ssr: false,
});
assert.ok(
  cssModuleLoad && typeof cssModuleLoad === "object",
  "CSS module virtual loads should succeed",
);
assert.match(
  cssModuleLoad.code,
  /import buttonStyles from "\/src\/ModuleButton\.vue\?vue=&type=style&index=0&lang=css&module=buttonStyles";/,
  "CSS module virtual loads should emit delegated style imports",
);
assert.match(
  cssModuleLoad.code,
  /_sfc_main\.__cssModules\["buttonStyles"\] = buttonStyles;/,
  "CSS module bindings should be attached for normal-script output without relying on semicolons",
);

const applyCssPath = "/src/ApplyStyles.vue";
const applyCssState: VizePluginState = {
  ...hmrState,
  cache: new Map([
    [
      applyCssPath,
      {
        code: `const _sfc_main = { name: "ApplyStyles" }
export default _sfc_main`,
        scopeId: "applycss",
        hasScoped: true,
        styles: [
          {
            content: ".root { @apply text-fg; }",
            lang: "css",
            scoped: true,
            module: false,
            index: 0,
          },
        ],
      },
    ],
  ]),
  ssrCache: new Map(),
};

const applyCssLoad = loadHook(applyCssState, toVirtualId(applyCssPath), {
  ssr: false,
});
assert.ok(
  applyCssLoad && typeof applyCssLoad === "object",
  "CSS with @apply should load successfully",
);
assert.match(
  applyCssLoad.code,
  /import "\/src\/ApplyStyles\.vue\?vue=&type=style&index=0&scoped=data-v-applycss&lang=css";/,
  "CSS with @apply should be delegated so PostCSS and UnoCSS transformers can run",
);
assert.doesNotMatch(
  applyCssLoad.code,
  /__vize_css__/,
  "Delegated @apply CSS should not be injected through the inline CSS runtime path",
);

const applyCssVirtualLoad = loadHook(
  applyCssState,
  "\0/src/ApplyStyles.vue?vue=&type=style&index=0&scoped=data-v-applycss&lang=css.css",
  { ssr: false },
);
assert.ok(
  applyCssVirtualLoad && typeof applyCssVirtualLoad === "object",
  "Delegated @apply CSS should load as a virtual style module",
);
assert.match(
  applyCssVirtualLoad.code,
  /\.root\[data-v-applycss\] \{ @apply text-fg; \}/,
  "Delegated @apply CSS should keep @apply while applying the scoped selector",
);

const onDemandProdDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-load-"));
const onDemandProdPath = path.join(onDemandProdDir, "OnDemandProd.vue");
fs.writeFileSync(
  onDemandProdPath,
  `<template><div class="prod">Prod</div></template><style>.prod { color: seagreen; }</style>`,
);

const onDemandProdState: VizePluginState = {
  ...hmrState,
  cache: new Map(),
  ssrCache: new Map(),
  collectedCss: new Map(),
  isProduction: true,
  extractCss: true,
};

const onDemandProdLoad = loadHook(onDemandProdState, toVirtualId(onDemandProdPath), {
  ssr: false,
});
assert.ok(
  onDemandProdLoad && typeof onDemandProdLoad === "object",
  "Production on-demand loads should still compile successfully",
);
assert.doesNotMatch(
  onDemandProdLoad.code,
  /__vize_css__/,
  "Production on-demand loads should not inline CSS when extraction is enabled",
);
assert.equal(
  onDemandProdState.collectedCss.get(onDemandProdPath),
  ".prod { color: seagreen; }",
  "Production on-demand compilation should collect extracted CSS for generateBundle",
);

console.log("✅ vite-plugin-vize load boundary tests passed!");

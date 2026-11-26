import assert from "node:assert/strict";
import fs from "node:fs";
import { createServer as createHttpServer } from "node:http";
import { createRequire } from "node:module";
import type { AddressInfo } from "node:net";
import os from "node:os";
import path from "node:path";
import { test, type TestContext } from "node:test";

import { createServer, isFileServingAllowed, resolveConfig, type ResolvedConfig } from "vite";

import { vize } from "../index.ts";
import { loadHook } from "./load.ts";
import { createServedFileCheck } from "./served-files.ts";
import type { VizePluginState } from "./state.ts";

const require = createRequire(import.meta.url);

const APP = `<template><p class="app">app</p></template>
<style>.app { color: red; }</style>
<docs>app docs</docs>
<notes src="./notes.txt"></notes>
<far src="../../outside/far.txt"></far>
<env src="./.env"></env>
`;
const BUTTON = "<template><button>ui</button></template>\n";
const LINKED = "<template><p>linked</p></template>\n";
const OUTSIDE = `<template><p class="outside">outside</p></template>
<style>.outside { color: blue; }</style>
<docs>outside docs</docs>
`;

/**
 * workspace/app is the Vite root and workspace/ its workspace root, so
 * workspace/packages is served too. outside/ and linked/ are not.
 */
async function project(t: TestContext) {
  const base = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vize-served-files-"));
  t.after(() => fs.rmSync(base, { recursive: true, force: true }));
  const file = (name: string) => path.join(base, name);
  for (const [name, source] of Object.entries({
    "workspace/pnpm-workspace.yaml": "packages:\n  - app\n  - packages/*\n",
    "workspace/app/App.vue": APP,
    "workspace/app/notes.txt": "app notes\n",
    "workspace/app/.env": "TOKEN=app\n",
    "workspace/packages/ui/Button.vue": BUTTON,
    "linked/Linked.vue": LINKED,
    "outside/Outside.vue": OUTSIDE,
    "outside/far.txt": "far text\n",
  })) {
    fs.mkdirSync(path.dirname(file(name)), { recursive: true });
    fs.writeFileSync(file(name), source);
  }
  const root = file("workspace/app");
  const config = await resolveConfig({ root, configFile: false, logLevel: "silent" }, "serve");
  return { root, config, file };
}

function createState(root: string, config: ResolvedConfig | null): VizePluginState {
  return {
    cache: new Map(),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: config === null,
    root,
    clientViteBase: "/",
    serverViteBase: "/",
    server: config && ({ config, watcher: { add() {} } } as never),
    filter: () => true,
    scanPatterns: null,
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {},
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {},
    serverViteDefine: {},
    logger: { log() {}, info() {}, warn() {}, error() {} } as never,
  };
}

function loaded(state: VizePluginState, id: string, ssr = false): string | null {
  const result = loadHook(state, id, { ssr });
  return result === null || typeof result === "string" ? result : result.code;
}

void test("a dev server loads the SFCs its file rules allow", async (t) => {
  const { root, config, file } = await project(t);
  const state = createState(root, config);
  const app = file("workspace/app/App.vue");

  assert.equal(loaded(state, app), APP);
  assert.equal(loaded(state, `${app}?vue&vize`), APP);
  assert.equal(loaded(state, file("workspace/packages/ui/Button.vue")), BUTTON);
  assert.equal(loaded(state, `${app}?vue&type=style&index=0&lang.css`), ".app { color: red; }");
  assert.equal(loaded(state, `${app}?vue&type=docs&index=0`), "app docs");
  assert.equal(loaded(state, `${app}?vue&type=notes&index=1&src=true`), "app notes\n");
});

void test("a dev server leaves an SFC outside its file rules to Vite", async (t) => {
  const { root, config, file } = await project(t);
  const state = createState(root, config);
  const outside = file("outside/Outside.vue");

  assert.equal(loaded(state, outside), null);
  assert.equal(loaded(state, `${outside}?vue&vize`), null);
  assert.equal(loaded(state, `${outside}?vue&vize-ssr`), null);
  assert.equal(loaded(state, `${outside}?vue&type=style&index=0&lang.css`), null);
  assert.equal(loaded(state, `${outside}?vue&type=docs&index=0`), null);
  assert.deepEqual([...state.cache.keys()], []);
});

void test("a dev server loads an outside SFC once a served module imports it", async (t) => {
  const { root, config, file } = await project(t);
  const state = createState(root, config);
  const linked = file("linked/Linked.vue");

  assert.equal(loaded(state, linked), null);
  // Vite records every import of a served module here.
  (config as ResolvedConfig & { safeModulePaths: Set<string> }).safeModulePaths.add(linked);
  assert.equal(loaded(state, linked), LINKED);
});

void test("a build and the server's own SSR graph load what they import", async (t) => {
  const { root, config, file } = await project(t);
  const outside = file("outside/Outside.vue");
  const app = file("workspace/app/App.vue");

  assert.equal(loaded(createState(root, config), outside, true), OUTSIDE);
  const build = createState(root, null);
  assert.equal(loaded(build, outside), OUTSIDE);
  assert.equal(loaded(build, `${outside}?vue&type=docs&index=0`), "outside docs");
  assert.equal(loaded(build, `${app}?vue&type=far&index=2&src=true`), "far text\n");
  assert.equal(loaded(build, `${app}?vue&type=env&index=3&src=true`), "TOKEN=app\n");
});

void test("a dev server refuses a custom block src outside its file rules", async (t) => {
  const { root, config, file } = await project(t);
  const state = createState(root, config);
  const app = file("workspace/app/App.vue");
  const watched: string[] = [];
  const load = (id: string) =>
    loadHook(state, id, { ssr: false, addWatchFile: (path) => watched.push(path) });

  assert.throws(() => load(`${app}?vue&type=far&index=2&src=true`), {
    message: `[vize] Custom block <far src="../../outside/far.txt"> is outside the files the dev server may read (resolved: ${file("outside/far.txt")}) in ${app}. See Vite's server.fs.allow.`,
  });
  assert.throws(() => load(`${app}?vue&type=env&index=3&src=true`), {
    message: `[vize] Custom block <env src="./.env"> is outside the files the dev server may read (resolved: ${file("workspace/app/.env")}) in ${app}. See Vite's server.fs.allow.`,
  });
  assert.deepEqual(watched, []);
});

void test("the file rules still apply when Vite lacks the newer helpers", async (t) => {
  const { root, config, file } = await project(t);
  const state = createState(root, config);
  const app = file("workspace/app/App.vue");
  const outside = file("outside/Outside.vue");

  const asked: string[] = [];
  const older = createServedFileCheck({
    isFileServingAllowed(resolved, url) {
      asked.push(url);
      return isFileServingAllowed(resolved, url);
    },
  });
  assert.equal(older(state, app, false), true);
  assert.equal(older(state, outside, false), false);
  assert.equal(older(state, "C:\\workspace\\app\\App.vue", false), false);
  assert.deepEqual(asked, [app, outside, "C:/workspace/app/App.vue"]);

  const oldest = createServedFileCheck({});
  assert.equal(oldest(state, app, false), true);
  assert.equal(oldest(state, file("workspace/packages/ui/Button.vue"), false), true);
  assert.equal(oldest(state, outside, false), false);
  assert.equal(oldest(state, file("workspace-other/App.vue"), false), false);
  assert.equal(oldest(state, outside, true), true);
  assert.equal(oldest(createState(root, null), outside, false), true);
});

void test("the dev server answers for an outside SFC as Vite does for any outside file", async (t) => {
  const { root, file } = await project(t);
  fs.writeFileSync(file("workspace/app/Plain.vue"), "<template><p>plain</p></template>\n");
  fs.writeFileSync(
    file("workspace/app/entry.js"),
    'import Plain from "./Plain.vue";\nimport Linked from "../../linked/Linked.vue";\nconsole.log(Plain, Linked);\n',
  );
  const server = await createServer({
    root,
    configFile: false,
    logLevel: "silent",
    plugins: [vize({ configMode: false, isProduction: false })],
    resolve: { alias: [{ find: /^vue$/u, replacement: require.resolve("vue") }] },
    optimizeDeps: { noDiscovery: true, include: [] },
    server: { middlewareMode: true, ws: false, preTransformRequests: false },
  });
  const http = createHttpServer(server.middlewares);
  await new Promise<void>((resolve) => http.listen(0, "127.0.0.1", resolve));
  t.after(async () => {
    http.close();
    await server.close();
  });
  const origin = `http://127.0.0.1:${(http.address() as AddressInfo).port}`;
  const status = async (url: string) => (await fetch(origin + url)).status;
  const outside = `/@fs${file("outside/Outside.vue")}`;
  const linked = `/@fs${file("linked/Linked.vue")}`;

  assert.equal(await status("/Plain.vue"), 200);
  assert.equal(await status(outside), 403);
  assert.equal(await status(`${outside}?vue&vize`), 403);
  assert.equal(await status(`${outside}?vue&type=style&index=0&lang.css`), 403);
  assert.equal(await status(`${outside}?vue&type=docs&index=0`), 403);
  assert.equal(await status(`${linked}?vue&vize`), 403);

  // The entry imports the linked SFC, so Vite now serves it.
  const entry = await (await fetch(`${origin}/entry.js`)).text();
  const linkedImport = /from\s+"([^"]*Linked\.vue[^"]*)"/.exec(entry)?.[1];
  assert.equal(linkedImport, `${linked}?vue&vize`);
  assert.equal(await status(linkedImport), 200);
  assert.equal(await status(outside), 403);
});

import assert from "node:assert/strict";
import { test } from "node:test";
import { fromPluginVisibleVirtualId, toPluginVisibleVirtualId } from "../virtual.ts";
import { resolveHookSsr } from "./index-helpers.ts";

void test("client/server share Vue identity while SSR follows the execution environment", () => {
  assert.equal(toPluginVisibleVirtualId("/app/Page.vue"), "/app/Page.vue?vue&vize");
  assert.equal(
    toPluginVisibleVirtualId("/app/Page.vue", true),
    toPluginVisibleVirtualId("/app/Page.vue"),
  );
  assert.equal(
    toPluginVisibleVirtualId("/app/Page.vue", true, "?vize-ssr&vue&used=true"),
    "/app/Page.vue?vue&vize&used=true",
  );
  for (const id of [
    "/app/Page.vue?vue&vize",
    "/app/Page.vue.ts?vue&vize",
    "/app/Page.vue.tsx?vue&vize-ssr",
    "/@fs/app/Page.vue?vue&vize",
  ])
    assert.equal(fromPluginVisibleVirtualId(id), "/app/Page.vue");
  for (const id of [
    "/app/Page.vue",
    "/app/Page.vue?raw",
    "/app/Page.ts?vue&vize",
    "/app/Page.vue?vue",
  ])
    assert.equal(fromPluginVisibleVirtualId(id), null);
  assert.deepEqual(
    ["ssr", "server", "client", "browser", undefined].map((name) =>
      resolveHookSsr({ environment: { name } }),
    ),
    [true, true, false, false, false],
  );
  assert.equal(resolveHookSsr({ environment: { name: "ssr" } }, { ssr: false }), false);
  assert.equal(resolveHookSsr({}, { ssr: true }), true);
});

void test("the same canonical ID loads real client and SSR compilation caches independently", async () => {
  const fs = await import("node:fs");
  const os = await import("node:os");
  const path = await import("node:path");
  const { pathToFileURL } = await import("node:url");
  const { createRequire } = await import("node:module");
  const { compileFile } = await import("../compiler.ts");
  const { getCompileOptionsForRequest } = await import("./state.ts");
  const { loadHook, transformHook } = await import("./load.ts");
  const root = fs.mkdtempSync(path.join(fs.realpathSync(os.tmpdir()), "vize-mode-cache-"));
  try {
    const filename = path.join(root, "app/pages/Page.vue");
    fs.mkdirSync(path.dirname(filename), { recursive: true });
    fs.writeFileSync(
      filename,
      '<script setup lang="ts">const message: string = "hello";</script><template><p>{{ message }}</p></template>',
    );
    const state = {
      cache: new Map(),
      ssrCache: new Map(),
      collectedCss: new Map(),
      precompileMetadata: new Map(),
      pendingHmrUpdateTypes: new Map(),
      isProduction: true,
      root,
      server: null,
      initialized: true,
      clientViteBase: "/",
      serverViteBase: "/",
      filter: () => true,
      mergedOptions: { sourceMap: false, ssrModuleIdRoot: path.join(root, "app") },
      extractCss: false,
      cssAliasRules: [],
      dynamicImportAliasRules: [],
      clientViteDefine: {},
      serverViteDefine: {},
      logger: { log() {}, warn() {}, error() {}, info() {} },
    } as unknown as import("./state.ts").VizePluginState;
    const client = compileFile(filename, state.cache, getCompileOptionsForRequest(state, false));
    const server = compileFile(filename, state.ssrCache, getCompileOptionsForRequest(state, true));
    assert.notEqual(client, server);
    const id = toPluginVisibleVirtualId(filename);
    const outputs: string[] = [];
    for (const ssr of [false, true, false, true]) {
      const loaded = loadHook(state, id, { ssr });
      assert.ok(loaded && typeof loaded === "object");
      assert.equal(await transformHook(state, loaded.code, id, { ssr }), null);
      outputs.push(loaded.code);
    }
    assert.equal(outputs[0], outputs[2]);
    assert.equal(outputs[1], outputs[3]);
    assert.notEqual(outputs[0], outputs[1]);
    assert.equal(state.cache.get(filename), client);
    assert.equal(state.ssrCache.get(filename), server);
    const requireFrom = createRequire(import.meta.url);
    const vueRoot = path.dirname(requireFrom.resolve("vue/package.json"));
    fs.mkdirSync(path.join(root, "node_modules"));
    fs.symlinkSync(vueRoot, path.join(root, "node_modules/vue"), "dir");
    for (const [i, code] of outputs.slice(0, 2).entries())
      fs.writeFileSync(path.join(root, `${i}.mjs`), code);
    const clientModule = await import(pathToFileURL(path.join(root, "0.mjs")).href);
    const serverModule = await import(pathToFileURL(path.join(root, "1.mjs")).href);
    assert.equal(typeof clientModule.default.render, "function");
    assert.equal(clientModule.default.ssrRender, undefined);
    assert.equal(typeof serverModule.default.ssrRender, "function");
    assert.equal(serverModule.default.render, undefined);
    const { createSSRApp } = await import(pathToFileURL(requireFrom.resolve("vue")).href);
    const { renderToString } = await import(
      pathToFileURL(requireFrom.resolve("vue/server-renderer")).href
    );
    const context: { modules?: Set<string> } = {};
    assert.equal(await renderToString(createSSRApp(serverModule.default), context), "<p>hello</p>");
    assert.deepEqual([...(context.modules ?? [])], ["pages/Page.vue"]);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

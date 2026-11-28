import assert from "node:assert/strict";
import { test } from "node:test";
import { createNuxtTransformBridge, type NuxtUnimportContext } from "./transform-bridge.ts";

void test("bridge reads late unimport and build state while retaining SSR behavior", async () => {
  let context: NuxtUnimportContext | null = null;
  let build = false;
  let calls = 0;
  const plugin = createNuxtTransformBridge({
    bridge: { autoImports: false, components: false, i18n: false, stableInjectedKeys: false },
    components: null,
    getUnimportContext: () => context,
    isBuild: () => build,
    isSsrTransform: (args) => (args[0] as { ssr?: boolean } | undefined)?.ssr === true,
  });
  const source = 'import { ref } from "vue"; export const value = ref(1);';
  const id = "/src/App.vue?vue&vize";
  assert.equal(await plugin.transform.handler.call({}, source, id), undefined);
  context = {
    async injectImports(code) {
      calls++;
      return { code: `${code}\nconst injected = 1;`, s: null, imports: ["injected"] };
    },
  };
  const late = await plugin.transform.handler.call({}, source, id);
  assert.match(late!.code, /const injected = 1/);
  assert.match(late!.code, /from "vue"/);
  build = true;
  const client = await plugin.transform.handler.call({}, source, id);
  assert.match(client!.code, /from "vue\/dist\/vue.runtime.esm-bundler.js"/);
  const server = await plugin.transform.handler.call({}, source, id, { ssr: true });
  assert.match(server!.code, /from "vue"/);
  assert.equal(await plugin.transform.handler.call({}, source, "/src/plain.ts"), undefined);
  assert.equal(calls, 3);
  context = null;
  build = false;
  assert.equal(await plugin.transform.handler.call({}, source, id), undefined);
});

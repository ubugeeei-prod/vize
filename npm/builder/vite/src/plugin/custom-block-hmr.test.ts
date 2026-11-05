import assert from "node:assert/strict";
import type { HmrContext } from "vite";

import { compileFile } from "../compiler.ts";
import type { VizePluginState } from "./state.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import { toVirtualId } from "../virtual.ts";

// Editing only `<message>` must invalidate that block's module so the user
// plugin transforms the new text, and reload the component that calls it.
{
  const vueFile = "/src/Message.vue";
  const previousSource = `<script>export default { name: "App" }</script>\n<message>Hello from custom block</message>\n`;
  const nextSource = `<script>export default { name: "App" }</script>\n<message>Updated custom block</message>\n`;
  const previousCompiled = compileFile(
    vueFile,
    new Map(),
    { root: "/src", sourceMap: false, ssr: false, vapor: false },
    previousSource,
  );
  assert.equal(previousCompiled.customBlocks?.length, 1);
  assert.equal(previousCompiled.customBlocks?.[0]?.content.includes("Hello from custom block"), true);

  const blockId = `${vueFile}?vue&type=message&index=0`;
  const blockModule = { url: blockId };
  const componentModule = { url: toVirtualId(vueFile) };
  const invalidated: unknown[] = [];
  const state = {
    cache: new Map([[vueFile, previousCompiled]]),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    mergedOptions: {},
    cssAliasRules: [],
    clientViteBase: "/",
    root: "/src",
    filter: () => true,
    logger: { log() {}, error() {} },
  } as unknown as VizePluginState;
  const ctx = {
    file: vueFile,
    server: {
      moduleGraph: {
        getModulesByFile(id: string) {
          if (id === blockId) return new Set([blockModule]);
          if (id === toVirtualId(vueFile)) return new Set([componentModule]);
          return undefined;
        },
        invalidateModule(module: unknown) {
          invalidated.push(module);
        },
      },
      ws: { send() {} },
    },
    read: async () => nextSource,
  } as unknown as HmrContext;

  const modules = await handleHotUpdateHook(state, ctx);
  assert.equal(state.pendingHmrUpdateTypes.get(vueFile), "full-reload");
  assert.ok(
    modules?.includes(blockModule as never),
    "a custom-block edit must return the custom-block module",
  );
  assert.ok(invalidated.includes(blockModule), "the custom-block module is invalidated");
  assert.ok(
    modules?.includes(componentModule as never) || invalidated.includes(componentModule),
    "the component module reloads so the new block function runs",
  );
}

console.log("✅ vite-plugin-vize custom block hmr tests passed!");

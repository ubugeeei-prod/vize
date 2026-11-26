import assert from "node:assert/strict";
import vm from "node:vm";
import type { HmrContext } from "vite";

import { compileFile } from "../compiler.ts";
import { generateHmrCode, VIZE_RERENDER_EVENT, type HmrUpdateType } from "../hmr.ts";
import { toVirtualId } from "../virtual.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import type { VizePluginState } from "./state.ts";

// A template-only save may rerender the component and keep its setup state,
// but the compiled module runs again whenever one of its imports changes. That
// run must reload the component, or the setup keeps what the old composable
// returned while the template already calls the new one.

const vueFile = "/src/Page.vue";
const page = (template: string, script = "const v = useFresh();") =>
  `<script setup lang="ts">\n${script}\n</script>\n<template><p>${template} {{ v }}</p></template>\n`;

function hotUpdate(previousSource: string, nextSource: string) {
  const previous = compileFile(
    vueFile,
    new Map(),
    { root: "/src", sourceMap: false, ssr: false, vapor: false },
    previousSource,
  );
  const module = { url: toVirtualId(vueFile) };
  const sent: unknown[] = [];
  const state = {
    cache: new Map([[vueFile, previous]]),
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
        getModulesByFile: (id: string) => (id === module.url ? new Set([module]) : undefined),
        invalidateModule() {},
      },
      ws: { send: (payload: unknown) => sent.push(payload) },
    },
    read: async () => nextSource,
  } as unknown as HmrContext;
  return { state, ctx, sent, scopeId: previous.scopeId };
}

{
  const { state, ctx, sent, scopeId } = hotUpdate(page("a"), page("b"));
  await handleHotUpdateHook(state, ctx);
  assert.equal(state.pendingHmrUpdateTypes.get(vueFile), "template-only");
  assert.deepEqual(
    sent,
    [{ type: "custom", event: VIZE_RERENDER_EVENT, data: { ids: [scopeId] } }],
    "a template-only save names the SFC it may rerender",
  );
}

{
  const { state, ctx, sent } = hotUpdate(page("a"), page("a", "const v = useFresh() + 1;"));
  await handleHotUpdateHook(state, ctx);
  assert.equal(state.pendingHmrUpdateTypes.get(vueFile), "full-reload");
  assert.deepEqual(sent, [], "a script save asks for no rerender");
}

// Run the emitted HMR code the way the Vite client does: `hot.data` survives
// re-executions of one module, and the listeners of the previous instance
// receive the custom events sent before an update.
{
  const hmrId = "page123";
  const data: Record<string, unknown> = {};
  let listeners: Array<(payload: { ids: string[] }) => void> = [];
  const send = (ids: string[]) => {
    for (const listener of listeners) listener({ ids });
  };
  const execute = (updateType: HmrUpdateType): unknown => {
    listeners = [];
    const hot = {
      data,
      accept() {},
      on(event: string, listener: (payload: { ids: string[] }) => void) {
        if (event === VIZE_RERENDER_EVENT) listeners.push(listener);
      },
    };
    const sfcMain: Record<string, unknown> = {};
    const code = generateHmrCode(hmrId, updateType).replaceAll("import.meta.hot", "hot");
    vm.runInNewContext(code, { hot, _sfc_main: sfcMain });
    return sfcMain.__hmrUpdateType;
  };

  execute("full-reload");
  send([hmrId]);
  assert.equal(execute("template-only"), "template-only", "its own save rerenders");
  assert.equal(
    execute("template-only"),
    "full-reload",
    "a re-execution after an import changed reloads, though the output is cached",
  );
  send(["other456"]);
  assert.equal(execute("template-only"), "full-reload", "another SFC's save does not rerender");
  send([hmrId]);
  assert.equal(execute("full-reload"), "full-reload", "a script save still reloads");
  assert.equal(execute("template-only"), "full-reload", "the request is read once");
}

console.log("✅ vite-plugin-vize dependency re-run HMR tests passed!");

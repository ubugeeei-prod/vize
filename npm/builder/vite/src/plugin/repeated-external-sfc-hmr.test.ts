import assert from "node:assert/strict";
import type { HmrContext } from "vite";

import type { CompiledModule } from "../types.ts";
import { toVirtualId } from "../virtual.ts";
import { CompiledModuleCache } from "./compiled-module-cache.ts";
import { handleHotUpdateHook } from "./hmr.ts";
import type { VizePluginState } from "./state.ts";

// External SFC blocks can be saved and then repaired before Vite reloads the
// owner. Both changes must invalidate the owner; the first eviction must not
// erase the dependency route needed by the repair.
const vueFile = "/src/External.vue";
const dependencyFile = "/src/External.template.html";
const module = { url: toVirtualId(vueFile) };
const cache = new CompiledModuleCache();
cache.set(vueFile, {
  code: "export default {}",
  scopeId: "external123",
  hasScoped: false,
  dependencies: [dependencyFile],
} as CompiledModule);
const invalidatedModules: unknown[] = [];
const state = {
  cache,
  ssrCache: new CompiledModuleCache(),
  collectedCss: new Map(),
  precompileMetadata: new Map(),
  pendingHmrUpdateTypes: new Map(),
  root: "/src",
  logger: {
    log() {},
  },
} as unknown as VizePluginState;
const ctx = {
  file: dependencyFile,
  server: {
    moduleGraph: {
      getModulesByFile(id: string) {
        return id === toVirtualId(vueFile) ? new Set([module]) : undefined;
      },
      invalidateModule(receivedModule: unknown) {
        invalidatedModules.push(receivedModule);
      },
    },
  },
  read: async () => "",
} as unknown as HmrContext;

assert.deepEqual(await handleHotUpdateHook(state, ctx), [module]);
assert.equal(cache.has(vueFile), false);
assert.deepEqual(await handleHotUpdateHook(state, ctx), [module]);
assert.deepEqual(invalidatedModules, [module, module]);

console.log("vite-plugin-vize repeated external SFC HMR tests passed!");

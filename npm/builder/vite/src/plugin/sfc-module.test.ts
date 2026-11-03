import assert from "node:assert/strict";
import { test } from "node:test";

import type { VizePluginState } from "./state.ts";
import { transformHook } from "./load.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";
import { transformCachedSfcModule } from "../test/sfc-module.ts";

const ENV_FLAGS = "/src/EnvFlags.vue";

function stateWithEnvFlags(): VizePluginState {
  return {
    cache: new Map([
      [
        ENV_FLAGS,
        {
          code: `export const flags = [import.meta.client, import.meta.server, import.meta.dev, import.meta.hot];
export default {};`,
          scopeId: "envflags",
          hasScoped: false,
          styles: [],
        },
      ],
    ]),
    ssrCache: new Map(),
    collectedCss: new Map(),
    precompileMetadata: new Map(),
    pendingHmrUpdateTypes: new Map(),
    isProduction: false,
    root: "/src",
    clientViteBase: "/",
    serverViteBase: "/",
    server: {} as never,
    filter: () => true,
    scanPatterns: ["**/*.vue"],
    precompileBatchSize: 128,
    ignorePatterns: [],
    mergedOptions: {},
    initialized: true,
    dynamicImportAliasRules: [],
    cssAliasRules: [],
    extractCss: false,
    componentsCssFileName: "assets/vize-components.css",
    clientViteDefine: {
      "import.meta.client": "true",
      "import.meta.server": "false",
      "import.meta.dev": "true",
    },
    serverViteDefine: {},
    logger: { log() {}, info() {}, warn() {}, error() {} } as never,
  } as unknown as VizePluginState;
}

void test("the compile stage, not the pre stage, inlines a virtual SFC module's environment flags", async () => {
  const state = stateWithEnvFlags();
  // An SFC main module is still SFC text at the pre stage.
  assert.equal(
    await transformHook(state, "<template />", toVirtualId(ENV_FLAGS), { ssr: false }),
    null,
  );

  for (const id of [toVirtualId(ENV_FLAGS), toPluginVisibleVirtualId(ENV_FLAGS)]) {
    const compiled = await transformCachedSfcModule(state, id, { ssr: false });
    assert.doesNotMatch(
      compiled.code,
      /import\.meta\.(client|server|dev)/,
      `${id}: flags Vite skips for virtual ids are inlined before post plugins run`,
    );
    assert.match(compiled.code, /import\.meta\.hot/, `${id}: import.meta.hot stays for Vite HMR`);
    assert.equal(compiled.map, null, `${id}: no map is allocated for a module without one`);
  }
});

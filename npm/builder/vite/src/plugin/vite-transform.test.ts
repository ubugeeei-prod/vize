import assert from "node:assert/strict";

import * as vite from "vite";

import {
  createVirtualTypeScriptTransformer,
  transformVizeVirtualModule,
} from "./vite-transform.ts";
import type { VizePluginState } from "./state.ts";

{
  let used = "";
  const transform = createVirtualTypeScriptTransformer({
    transformWithOxc: async (code, id, options) => {
      used = "oxc";
      assert.equal(code, "const value: number = 1");
      assert.equal(id, "/src/App.vue");
      assert.deepEqual(options, { lang: "ts", sourcemap: false, target: "esnext" });
      return { code: "const value = 1;" };
    },
    transformWithEsbuild: async () => {
      used = "esbuild";
      return { code: "" };
    },
  });

  const result = await transform("const value: number = 1", "/src/App.vue");
  assert.equal(used, "oxc", "Vite OXC should be preferred when available");
  assert.equal(result.code, "const value = 1;");
}

{
  let used = "";
  const transform = createVirtualTypeScriptTransformer({
    transformWithEsbuild: async (code, id, options) => {
      used = "esbuild";
      assert.equal(code, "const value: number = 1");
      assert.equal(id, "/src/App.vue");
      assert.deepEqual(options, { loader: "ts", sourcemap: false, target: "esnext" });
      return { code: "const value = 1;" };
    },
  });

  const result = await transform("const value: number = 1", "/src/App.vue");
  assert.equal(used, "esbuild", "Vite 7 should fall back to transformWithEsbuild");
  assert.equal(result.code, "const value = 1;");
}

{
  const code = "const value = external ? { isActive: undefined } : { isActive: scope?.isActive };";
  const transform = createVirtualTypeScriptTransformer({
    transformWithOxc: async (_code, _id, options) => ({
      code: options.target === "esnext" ? code : code.replace("scope?.isActive", "scope.isActive"),
    }),
  });

  const result = await transform(code, "/src/Link.vue");

  assert.match(
    result.code,
    /scope\?\.isActive/,
    "virtual Vue module TS stripping must preserve template optional chaining",
  );
  assert.doesNotMatch(
    result.code,
    /scope\.isActive/,
    "virtual Vue module TS stripping must not emit an unguarded slot-scope access",
  );
}

// Vize's Rust emitter guarantees plain JavaScript for every module it produces,
// so the virtual-module transform must not re-print emitter output through
// Vite's TypeScript strip. Anything the emitter did not produce still must.
{
  let stripCalls = 0;
  const emitted = 'import { ref as _ref } from "vue";\nconst count = _ref(0);\nexport default {};';
  const state = {
    cache: new Map([["/src/Emitted.vue", {}]]),
    ssrCache: new Map(),
    clientViteDefine: {},
    serverViteDefine: {},
    isProduction: false,
    root: "/src",
    logger: { error() {} },
  } as unknown as VizePluginState;

  const viteApi = vite as { transformWithOxc?: unknown };
  const originalOxc = viteApi.transformWithOxc;
  viteApi.transformWithOxc = () => {
    stripCalls += 1;
    return { code: emitted };
  };

  try {
    const result = await transformVizeVirtualModule(state, emitted, "/src/Emitted.vue", false);
    assert.equal(result, null, "unchanged emitter output should not produce a transform result");
    assert.equal(stripCalls, 0, "emitter output must skip Vite's TypeScript strip entirely");

    await transformVizeVirtualModule(state, emitted, "/src/NotEmitted.vue", false);
    assert.equal(
      stripCalls,
      1,
      "modules Vize did not emit must still go through Vite's TypeScript strip",
    );

    await transformVizeVirtualModule(state, emitted, "/src/Emitted.vue", false, true);
    assert.equal(
      stripCalls,
      2,
      "?macro=true raw artifacts must still go through Vite's TypeScript strip",
    );
  } finally {
    viteApi.transformWithOxc = originalOxc;
  }
}

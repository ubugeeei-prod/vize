import assert from "node:assert/strict";

import {
  createVirtualTypeScriptTransformer,
  transformVizeVirtualModule,
} from "./vite-transform.ts";
import type { VizePluginState } from "./state.ts";
import { transformHook } from "./load.ts";
import { getSfcModuleRequest, transformSfcModule } from "./sfc-module.ts";
import { toPluginVisibleVirtualId, toVirtualId } from "../virtual.ts";

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

// Vize loads virtual SFC modules as JavaScript (emitter output is checked at
// the napi boundary; boundary placeholders are authored JS), so the transform
// hands them back without Vite's TypeScript strip, whatever the caches hold.
// Raw `?macro=true` script code still goes through it.
//
// The probe is behavioural rather than a call spy: the input carries a type
// annotation that only survives when the strip is skipped.
{
  const annotated = "const count: number = 1;\nexport default { count };";
  const state = {
    cache: new Map(),
    ssrCache: new Map(),
    clientViteDefine: {},
    serverViteDefine: {},
    isProduction: false,
    root: "/src",
    logger: { error() {} },
  } as unknown as VizePluginState;

  for (const ssr of [false, true]) {
    assert.equal(
      await transformVizeVirtualModule(state, annotated, "/src/Loaded.vue", ssr),
      null,
      "Vize's virtual SFC modules must be handed back untouched instead of re-printed",
    );
  }

  const macroArtifact = await transformVizeVirtualModule(
    state,
    annotated,
    "/src/Loaded.vue",
    false,
    true,
  );
  assert.ok(macroArtifact && typeof macroArtifact === "object", "macro artifacts must transform");
  assert.doesNotMatch(
    macroArtifact.code,
    /:\s*number/,
    "?macro=true raw artifacts never passed through the emitter, so they still need the strip",
  );
}

// #11210: a client build stubs `.server.vue` with Vize's own JavaScript. Its
// transform must not wait on Vite's TypeScript strip (tsconfig lookup included).
{
  const state = {
    cache: new Map(),
    ssrCache: new Map(),
    clientViteBase: "/",
    serverViteBase: "/",
    clientViteDefine: {},
    serverViteDefine: {},
    isProduction: true,
    mergedOptions: {},
    filter: () => true,
    root: "/src",
    logger: { log() {}, error() {} },
  } as unknown as VizePluginState;

  for (const id of [
    toVirtualId("/src/NoScript.server.vue"),
    toPluginVisibleVirtualId("/src/NoScript.server.vue"),
  ]) {
    const sfc = getSfcModuleRequest(state, id);
    assert.ok(sfc, id);
    const placeholder = await transformSfcModule(state, "<template><p /></template>", sfc, {
      ssr: false,
    });
    assert.match(placeholder.code, /ServerPlaceholder/, "Client builds stub .server.vue");
    assert.equal(placeholder.map, null, "the boundary placeholder is emitted as authored");
    assert.equal(
      await transformHook(state, "<template><p /></template>", id, { ssr: false }),
      null,
      "the SFC text reaches the compile stage untouched",
    );
  }

  assert.equal(
    await transformHook(state, "export default {", toVirtualId("/src/Broken.vue"), { ssr: false }),
    null,
    "malformed virtual SFC code must reach Vite's parser unchanged, never as an empty component",
  );
}

import assert from "node:assert/strict";

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

// Vize's Rust emitter guarantees plain JavaScript for every module it produces
// (`ensure_javascript_output` at the napi boundary), so the virtual-module
// transform must not re-print emitter output through Vite's TypeScript strip.
// Anything the emitter did not produce still must go through it.
//
// The probe is behavioural rather than a call spy: the input carries a type
// annotation that only survives when the strip is skipped.
{
  const annotated = "const count: number = 1;\nexport default { count };";
  const state = {
    cache: new Map([["/src/Emitted.vue", {}]]),
    ssrCache: new Map([["/src/SsrEmitted.vue", {}]]),
    clientViteDefine: {},
    serverViteDefine: {},
    isProduction: false,
    root: "/src",
    logger: { error() {} },
  } as unknown as VizePluginState;

  assert.equal(
    await transformVizeVirtualModule(state, annotated, "/src/Emitted.vue", false),
    null,
    "emitter output must be handed back untouched instead of re-printed through oxc",
  );
  assert.equal(
    await transformVizeVirtualModule(state, annotated, "/src/SsrEmitted.vue", true),
    null,
    "SSR emitter output must be handed back untouched too",
  );

  const notEmitted = await transformVizeVirtualModule(
    state,
    annotated,
    "/src/NotEmitted.vue",
    false,
  );
  assert.ok(notEmitted && typeof notEmitted === "object", "non-emitter modules must transform");
  assert.doesNotMatch(
    notEmitted.code,
    /:\s*number/,
    "modules Vize did not emit must still go through Vite's TypeScript strip",
  );

  const macroArtifact = await transformVizeVirtualModule(
    state,
    annotated,
    "/src/Emitted.vue",
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

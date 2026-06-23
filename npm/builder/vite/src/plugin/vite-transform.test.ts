import assert from "node:assert/strict";

import { createVirtualTypeScriptTransformer } from "./vite-transform.ts";

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

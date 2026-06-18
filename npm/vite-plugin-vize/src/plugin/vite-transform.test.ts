import assert from "node:assert/strict";

import { createVirtualTypeScriptTransformer } from "./vite-transform.ts";

{
  let used = "";
  const transform = createVirtualTypeScriptTransformer({
    transformWithOxc: async (code, id, options) => {
      used = "oxc";
      assert.equal(code, "const value: number = 1");
      assert.equal(id, "/src/App.vue");
      assert.deepEqual(options, { lang: "ts", sourcemap: false });
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
      assert.deepEqual(options, { loader: "ts", sourcemap: false });
      return { code: "const value = 1;" };
    },
  });

  const result = await transform("const value: number = 1", "/src/App.vue");
  assert.equal(used, "esbuild", "Vite 7 should fall back to transformWithEsbuild");
  assert.equal(result.code, "const value = 1;");
}

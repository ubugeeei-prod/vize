import assert from "node:assert/strict";
import { test } from "node:test";
import { loadConfig, validateOptions } from "./config.ts";

void test("only documented compiler and host controls are admitted", () => {
  const config = {
    compiler: {
      vapor: false,
      customElements: ["x-view"],
      templateSyntax: "strict",
      runtimeModuleName: "vue-vapor",
    },
    transformAssetUrls: false,
  };
  assert.deepEqual(validateOptions(config), config);
  assert.deepEqual(validateOptions({ compiler: { vapor: true } }), { compiler: { vapor: true } });
  for (const value of [
    null,
    [],
    true,
    { filename: "other.vue" },
    { compiler: { filename: "other.vue" } },
    { compiler: { sourceMap: false } },
    { compiler: { isTs: true } },
    { compiler: { vapor: 1 } },
    { compiler: { templateSyntax: ["strict"] } },
    { compiler: { customElements: [1] } },
    { transformAssetUrls: "false" },
    { compiler: { vapor: true, runtimeModuleName: "custom-runtime" } },
  ]) {
    assert.throws(() => validateOptions(value));
  }
});

void test("configuration uses Parcel's tracked config loader and package key", async () => {
  const calls: unknown[] = [];
  const config = {
    getConfig(...args: unknown[]) {
      calls.push(args);
      return { contents: { transformAssetUrls: false }, filePath: "/project/.vizerc.parcel.json" };
    },
  };
  assert.deepEqual(await loadConfig({ config: config as never }), { transformAssetUrls: false });
  assert.deepEqual(calls, [
    [[".vizerc.parcel.json"], { packageKey: "@vizejs/parcel-transformer" }],
  ]);
});

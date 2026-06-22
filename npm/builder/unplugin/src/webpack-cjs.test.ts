import { test } from "node:test";
import vizeWebpackCjs from "./webpack-cjs.ts";

void test("webpack CJS entry loads as a no-op plugin for Nuxt 2 host-compiler configs", (t) => {
  t.assert.doesNotThrow(() => {
    vizeWebpackCjs({ compatibility: { nuxtVersion: 2 } }).apply({} as never);
  });
});

void test("webpack CJS entry treats legacy Vue configs as host-compiler configs", (t) => {
  for (const vueVersion of [0.11, 1, 2, "legacy"] as const) {
    t.assert.doesNotThrow(() => {
      vizeWebpackCjs({ vueVersion }).apply({} as never);
    });
  }
});

void test("webpack CJS entry rejects modern compiler configs", (t) => {
  t.assert.throws(() => {
    vizeWebpackCjs().apply({} as never);
  }, /CommonJS/);

  t.assert.throws(() => {
    vizeWebpackCjs({ compatibility: { hostCompiler: false }, vueVersion: 2 }).apply({} as never);
  }, /CommonJS/);
});

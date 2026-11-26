import assert from "node:assert/strict";
import { test } from "node:test";

import { selectNuxtSourceComponentTransforms } from "./source-component-transforms.ts";

void test("the native compiler selects Nuxt's source component transforms", () => {
  // Nuxt 4.7 resolves `compilerComponentResolution` to true on its own, and a
  // project may set it explicitly. Neither can run without Vue node transforms.
  for (const experimental of [undefined, {}, { compilerComponentResolution: true }]) {
    const options = { experimental };
    selectNuxtSourceComponentTransforms(options);
    assert.equal(options.experimental?.compilerComponentResolution, false);
  }
});

void test("other experimental Nuxt options are kept", () => {
  const options = { experimental: { compilerComponentResolution: true, lazyHydration: true } };
  selectNuxtSourceComponentTransforms(options);
  assert.deepEqual(options.experimental, {
    compilerComponentResolution: false,
    lazyHydration: true,
  });
});

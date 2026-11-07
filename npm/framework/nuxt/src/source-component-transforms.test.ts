import assert from "node:assert/strict";
import { test } from "node:test";

import { selectNuxtSourceComponentTransforms } from "./source-component-transforms.ts";

void test("#11802 Nuxt 5 uses source component transforms with the native compiler", () => {
  for (const experimental of [undefined, {}, { compilerComponentResolution: true }]) {
    const options = { experimental };
    selectNuxtSourceComponentTransforms(options, 5);
    assert.equal(options.experimental?.compilerComponentResolution, false);
  }
});

void test("#11802 earlier Nuxt component resolution remains unchanged", () => {
  const options = { experimental: { compilerComponentResolution: true } };
  selectNuxtSourceComponentTransforms(options, 4);
  assert.equal(options.experimental.compilerComponentResolution, true);
});

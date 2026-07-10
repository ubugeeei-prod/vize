import assert from "node:assert/strict";
import test from "node:test";

import { appendOriginalVueSourceForUnoCss } from "./unocss.ts";

void test("Nuxt UnoCSS bridge appends bounded original Vue source", () => {
  const result = appendOriginalVueSourceForUnoCss("compiled", "/src/App.vue?vue", {
    maxBytes: 64,
    readSize: () => 42,
    readFile: () => '<template><div flex="~ col gap-2" /></template>',
  });

  assert.equal(
    result,
    'compiled\n<template><div flex="~ col gap-2" /></template>',
    "small SFC sources should still feed attributify extraction",
  );
});

void test("Nuxt UnoCSS bridge skips oversized original Vue sources", () => {
  let didRead = false;
  const result = appendOriginalVueSourceForUnoCss("compiled", "/src/Huge.vue?vue", {
    maxBytes: 64,
    readSize: () => 65,
    readFile: () => {
      didRead = true;
      return "<template />";
    },
  });

  assert.equal(result, "compiled");
  assert.equal(didRead, false, "oversized sources should not be read into Node heap");
});

void test("Nuxt UnoCSS bridge tolerates virtual or missing files", () => {
  const result = appendOriginalVueSourceForUnoCss("compiled", "\0virtual.vue?vue", {
    readSize: () => {
      throw new Error("missing");
    },
  });

  assert.equal(result, "compiled");
});

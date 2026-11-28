import assert from "node:assert/strict";
import { test } from "node:test";
import { baselineIntentionalReason } from "../../tools/support/fixtures/lint_divergence/records.ts";

function at(source: string, needle: string, ruleId: string, message: string) {
  const start = source.indexOf(needle);
  assert(start >= 0, needle);
  const lines = source.slice(0, start).split("\n");
  return {
    file: "Fixture.vue",
    ruleId,
    severity: "error",
    line: lines.length,
    column: lines.at(-1)!.length + 1,
    endLine: lines.length,
    endColumn: lines.at(-1)!.length + 2,
    message,
  };
}

void test("a static is on <component> names the rendered component", () => {
  const message = "Expected '<component>' elements to have 'v-bind:is' attribute.";
  const reason = (element: string) => {
    const source = `<template>\n  <div>\n    ${element}\n  </div>\n</template>\n`;
    return baselineIntentionalReason(
      at(source, "<component", "vue/require-component-is", message),
      source,
    );
  };
  assert.match(
    reason('<component is="TestComponent" v-for="item in 3" :key="item" />') ?? "",
    /static is/u,
  );
  for (const element of [
    "<component />",
    '<component is="" />',
    '<component :is="current" is="Fallback" />',
  ])
    assert.equal(reason(element), null, element);
});

void test("a typed computed getter leaves its default-less switch to TypeScript", () => {
  const message = "Expected to return a value in computed function.";
  const reason = (type: string, cases: string) => {
    const source = `<script setup lang="ts">\nimport { computed } from 'vue'\nconst props = defineProps<{ kind: 'a' | 'b' }>()\nconst label = computed${type}(() => {\n  switch (props.kind) {\n${cases}\n  }\n})\n</script>\n`;
    return baselineIntentionalReason(
      at(source, "() => {", "vue/return-in-computed-property", message),
      source,
    );
  };
  const exhaustive =
    "    case 'a':\n      return { text: 'A' }\n    case 'b': {\n      return null\n    }";
  for (const type of ["<{ text: string } | null>", "<string>", "<(string | null)>"])
    assert.match(reason(type, exhaustive) ?? "", /TypeScript/u, type);
  for (const [type, cases] of [
    ["", exhaustive],
    ["<{ text: string } | undefined>", exhaustive],
    ["<Label>", exhaustive],
    ["<any>", exhaustive],
    ["<string>", `${exhaustive}\n    default:\n      return ''`],
    ["<string>", "    case 'a':\n      return 'A'\n    case 'b':\n      break"],
    ["<string>", "    case 'a':\n      return 'A'\n    case 'b':\n      console.log('b')"],
  ])
    assert.equal(reason(type, cases), null, `${type} ${cases}`);
});

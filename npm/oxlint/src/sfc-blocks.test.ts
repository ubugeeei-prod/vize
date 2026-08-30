import assert from "node:assert/strict";
import { test } from "node:test";

import { extractSfcBlocks, formatBlockLabel, getDiagnosticBlock } from "./sfc-blocks.ts";

void test("SFC block extraction classifies common Vue block types", () => {
  const blocks = extractSfcBlocks(
    `<script setup lang="ts">\nconst count = 1\n</script>\n<template>\n  <div>{{ count }}</div>\n</template>\n<style scoped>\n.foo {}\n</style>\n<i18n>\n{}\n</i18n>\n`,
  );

  assert.deepEqual(
    blocks.map((block) => formatBlockLabel(block)),
    ["<script setup>", "<template>", "<style>", "<i18n>"],
  );
});

void test("SFC block extraction keeps quoted angle brackets inside attributes", () => {
  const blocks = extractSfcBlocks(
    `<script setup lang="ts" generic="T extends Record<string, unknown>">\nconst count = 1\n</script>\n`,
  );

  assert.equal(blocks[0]?.kind, "script-setup");
  assert.equal(blocks[0]?.content.trim(), "const count = 1");
});

void test("SFC block labels identify the diagnostic owner", () => {
  const blocks = extractSfcBlocks(
    `<script setup lang="ts">\nconst count = 1\n</script>\n<template>\n  <div>{{ count }}</div>\n</template>\n`,
  );

  assert.equal(
    formatBlockLabel(
      getDiagnosticBlock(
        {
          rule: "vize/vue/mock",
          severity: "error",
          message: "Mock error. Detail: extra context",
          location: {
            start: { line: 5, column: 3, offset: 0 },
            end: { line: 5, column: 8, offset: 0 },
          },
          help: null,
        },
        blocks,
      ),
    ),
    "<template>",
  );
});

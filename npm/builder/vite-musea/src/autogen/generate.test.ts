import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { generateArtFile } from "./index.js";

void test("generateArtFile preserves required object props with fixture bindings", async () => {
  const tempDir = await fs.promises.mkdtemp(path.join(os.tmpdir(), "musea-autogen-required-"));
  const componentPath = path.join(tempDir, "MoshiDetailCard.vue");

  try {
    await fs.promises.writeFile(
      componentPath,
      `<script setup lang="ts">
type MoshiDetailCardFragment = { id: string };

defineProps<{
  moshiWithStudent: MoshiDetailCardFragment;
}>();
</script>

<template>
  <div />
</template>
`,
      "utf-8",
    );

    const output = await generateArtFile(componentPath);

    assert.deepEqual(output.variants[0]?.props, {
      moshiWithStudent: null,
    });
    assert.match(output.artFileContent, /const moshiWithStudentFixture = \{\} as never;/);
    assert.match(
      output.artFileContent,
      /<MoshiDetailCard :moshi-with-student="moshiWithStudentFixture" \/>/,
    );
    assert.doesNotMatch(output.artFileContent, /<MoshiDetailCard \/>/);
  } finally {
    await fs.promises.rm(tempDir, { recursive: true, force: true });
  }
});

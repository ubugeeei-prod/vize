/** Small public inputs for cache-observer admission before any large campaign. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export function prepareCacheCarrier(root: string, provider: string, sfcs: number) {
  assert.ok(Number.isSafeInteger(sfcs) && sfcs >= 1 && sfcs <= 100);
  assert.ok(!fs.existsSync(root));
  fs.mkdirSync(path.join(root, "src"), { recursive: true });
  fs.symlinkSync(path.join(provider, "node_modules"), path.join(root, "node_modules"), "dir");
  fs.writeFileSync(
    path.join(root, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          noEmit: true,
          skipLibCheck: true,
          incremental: false,
        },
        include: ["src/**/*.vue"],
      },
      null,
      2,
    ),
  );
  const entries = [];
  for (let index = 0; index < sfcs; index++) {
    const source = `<script setup lang="ts">
import { useTemplateRef, onMounted } from "vue";
const title: string = "carrier-${index}";
const button = useTemplateRef("button");
onMounted(() => button.value?.focus());
function reset() { return title; }
defineExpose({ reset });
</script>
<template><button ref="button">{{ title }}</button></template>
`;
    const name = "src/Carrier" + index + ".vue";
    fs.writeFileSync(path.join(root, name), source);
    entries.push({
      path: name,
      bytes: Buffer.byteLength(source),
      sha256: createHash("sha256").update(source).digest("hex"),
    });
  }
  return {
    app: root,
    files: sfcs,
    sfcFiles: sfcs,
    sourceBytes: entries.reduce((total, row) => total + row.bytes, 0),
    entries,
    corpusSha256: createHash("sha256").update(JSON.stringify(entries)).digest("hex"),
  };
}

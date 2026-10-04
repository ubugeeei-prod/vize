/** Actual default collection corpus:500 Vue roots sharing a62-byte TS leaf. */
import { existsSync, mkdirSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

export function prepareSharedLeafCorpus(workRoot, vuePackageDir) {
  const dir = join(workRoot, "shared-leaf501-default");
  mkdirSync(dir, { recursive: true });
  writeFileSync(
    join(dir, "shared.ts"),
    "export interface Model {value:number}; export const shared=1;\n",
  );
  for (let index = 0; index < 500; index++) {
    const rows = Array.from(
      { length: 40 },
      (_, row) =>
        `<div class="item-${row}" v-if="index > ${row}">{{ double }}<button @click="index++">Add</button></div>`,
    );
    writeFileSync(
      join(dir, `Comp${index}.vue`),
      `<script setup lang="ts">
import { ref, computed } from 'vue';
import { shared } from './shared';
const index = ref(${index});
const double = computed(() => index.value * 2 + shared);
</script>
<template>
${rows.join("\n")}
</template>
<style scoped>div {color: red;}</style>
`,
    );
  }
  writeFileSync(
    join(dir, "Planted.vue"),
    '<script setup lang="ts">const failure: number = "planted";</script><template>{{ failure }}</template>',
  );
  const tsconfig = {
    compilerOptions: {
      strict: true,
      target: "ESNext",
      module: "ESNext",
      moduleResolution: "Bundler",
      skipLibCheck: true,
    },
    include: ["*.vue", "*.ts"],
  };
  writeFileSync(join(dir, "tsconfig.json"), `${JSON.stringify(tsconfig)}\n`);
  mkdirSync(join(dir, "node_modules"));
  symlinkSync(vuePackageDir, join(dir, "node_modules/vue"), "dir");
  const namespace = join(dirname(vuePackageDir), "@vue");
  if (existsSync(namespace)) symlinkSync(namespace, join(dir, "node_modules/@vue"), "dir");
  return {
    id: "shared-leaf501-default",
    dir,
    tsconfig,
    expectedVueFiles: 501,
    args: ["--no-config"],
  };
}

import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveCorsaPath, resolveVuePackagePath } from "../../editor-e2e/real-vue-workspace.mjs";

export const fileName = (index) => `Component${String(index).padStart(5, "0")}.vue`;

// Disjoint import pairs avoid a synthetic 10,000-member dependency cycle.
// Every SFC has a distinct public prop, so alpha interning cannot collapse the
// workspace into one interface page. All six public facets have real macros.
export function componentSource(index) {
  return `<script setup lang="ts">
import Peer from './${fileName(index ^ 1)}'
${index === 1 ? "import { useTemplateRef } from 'vue'\nconst peer = useTemplateRef('peer')\npeer.value?.publicState.toUpperCase()\n" : ""}
Peer
defineProps<{ label: string; file${index}?: number }>()
defineEmits<{ save: [value: string] }>()
defineSlots<{ default(props: { value: string }): unknown }>()
defineModel<number>('count')
const publicState: string = 'ready'
defineExpose({ publicState })
defineOptions({ inheritAttrs: false })
const checked: number = 1
</script>
<template><span>{{ label }}</span><Peer ${index === 1 ? 'ref="peer" ' : ""}:label="label" /></template>
`;
}

export function prepareWorkspace(workspace, files) {
  fs.mkdirSync(path.join(workspace, "src"), { recursive: true });
  fs.mkdirSync(path.join(workspace, "node_modules"), { recursive: true });
  fs.symlinkSync(resolveVuePackagePath(), path.join(workspace, "node_modules/vue"), "junction");
  const corsaPath = resolveCorsaPath();
  fs.writeFileSync(
    path.join(workspace, "vize.config.json"),
    JSON.stringify({
      lsp: { editor: true, ecosystem: true, lint: true, typecheck: true },
      typeChecker: { corsaPath },
    }),
  );
  fs.writeFileSync(
    path.join(workspace, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        target: "ES2022",
        module: "ESNext",
        moduleResolution: "bundler",
        lib: ["ES2022", "DOM", "DOM.Iterable"],
        strict: true,
        noEmit: true,
        skipLibCheck: true,
      },
      include: ["src/**/*.vue"],
    }),
  );
  for (let index = 0; index < files; index++) {
    fs.writeFileSync(path.join(workspace, "src", fileName(index)), componentSource(index));
  }
  return { corsaPath, vuePath: resolveVuePackagePath() };
}

export const documentUri = (workspace, index) =>
  pathToFileURL(path.join(workspace, "src", fileName(index))).href;

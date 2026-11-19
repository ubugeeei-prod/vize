<script setup lang="ts">
import { computed, ref } from "vue";
import CodeHighlight from "../../shared/code-highlight.vue";
import type { CodeOutputs } from "../atelier/codeOutputs";
import type { OutputTarget } from "./useStageLadder";

const { outputs, target, theme, backendPages } = defineProps<{
  outputs: CodeOutputs;
  target: OutputTarget;
  theme: "dark" | "light";
  backendPages?: { step: string; text: string }[];
}>();

const emit = defineEmits<{
  "update:target": [OutputTarget];
}>();

const TARGETS: { id: OutputTarget; label: string; note: string }[] = [
  { id: "dom", label: "DOM", note: "Virtual DOM render function" },
  { id: "vapor", label: "Vapor", note: "Vapor mode, no virtual DOM" },
  { id: "ssr", label: "SSR", note: "Server string renderer" },
];

const variant = computed(() => outputs[target]);
const selected = ref("backend:0");
const backendIndex = computed(() => {
  const index = Number(selected.value.slice("backend:".length));
  return Number.isInteger(index) && index >= 0 && index < (backendPages?.length ?? 0) ? index : 0;
});
const backend = computed(() => {
  return backendPages?.[backendIndex.value];
});
const moduleShown = computed(() => selected.value === "module" || !backend.value);
const code = computed(() =>
  moduleShown.value
    ? variant.value.formattedCode || variant.value.code
    : (backend.value?.text ?? ""),
);
</script>

<template>
  <div class="level-output">
    <div class="level-subtabs" role="tablist" aria-label="Emitted code">
      <button
        v-for="item in TARGETS"
        :key="item.id"
        type="button"
        role="tab"
        :class="['level-subtab', { active: target === item.id }]"
        :aria-selected="target === item.id"
        :title="item.note"
        @click="() => emit('update:target', item.id)"
      >
        {{ item.label }}
      </button>
    </div>
    <div class="level-subtabs" role="tablist" aria-label="Output boundary">
      <button
        v-for="(page, index) in backendPages ?? []"
        :key="`${page.step}:${index}`"
        type="button"
        role="tab"
        :class="['level-subtab', { active: !moduleShown && backendIndex === index }]"
        :aria-selected="!moduleShown && backendIndex === index"
        @click="() => (selected = `backend:${index}`)"
      >
        L4 {{ page.step }}
      </button>
      <button
        type="button"
        role="tab"
        :class="['level-subtab', { active: moduleShown }]"
        :aria-selected="moduleShown"
        @click="() => (selected = 'module')"
      >
        Assembled SFC module
      </button>
    </div>
    <div v-if="variant.error" class="level-message error">{{ variant.error }}</div>
    <div v-else-if="!code" class="level-message">This target emitted no code for the source.</div>
    <CodeHighlight
      v-else
      class="level-output-code"
      :code
      :language="moduleShown && variant.isTypeScript ? 'typescript' : 'javascript'"
      :theme
      show-line-numbers
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import CodeHighlight from "../../shared/CodeHighlight.vue";
import type { CodeOutputs } from "../atelier/codeOutputs";
import type { OutputTarget } from "./useDavinciLadder";

const props = defineProps<{
  outputs: CodeOutputs;
  target: OutputTarget;
  theme: "dark" | "light";
}>();

const emit = defineEmits<{
  "update:target": [OutputTarget];
}>();

const TARGETS: { id: OutputTarget; label: string; note: string }[] = [
  { id: "dom", label: "DOM", note: "Virtual DOM render function" },
  { id: "vapor", label: "Vapor", note: "Vapor mode, no virtual DOM" },
  { id: "ssr", label: "SSR", note: "Server string renderer" },
];

const variant = computed(() => props.outputs[props.target]);
const code = computed(() => variant.value.formattedCode || variant.value.code);
</script>

<template>
  <div class="davinci-output">
    <div class="davinci-subtabs" role="tablist" aria-label="Emitted code">
      <button
        v-for="item in TARGETS"
        :key="item.id"
        type="button"
        role="tab"
        :class="['davinci-subtab', { active: target === item.id }]"
        :aria-selected="target === item.id"
        :title="item.note"
        @click="() => emit('update:target', item.id)"
      >
        {{ item.label }}
      </button>
    </div>
    <div v-if="variant.error" class="davinci-message error">{{ variant.error }}</div>
    <div v-else-if="!code" class="davinci-message">This target emitted no code for the source.</div>
    <CodeHighlight
      v-else
      class="davinci-output-code"
      :code
      :language="variant.isTypeScript ? 'typescript' : 'javascript'"
      :theme
      show-line-numbers
    />
  </div>
</template>

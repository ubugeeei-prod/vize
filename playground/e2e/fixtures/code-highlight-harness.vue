<script setup lang="ts">
import { ref } from "vue";
import CodeHighlight from "../../src/shared/code-highlight.vue";

const { initial, replacement } = defineProps<{ initial: string; replacement: string }>();
const code = ref(initial);
</script>

<template>
  <button type="button" @click="code = replacement">Replace source</button>
  <CodeHighlight :code language="typescript" :show-line-numbers="true" />
</template>

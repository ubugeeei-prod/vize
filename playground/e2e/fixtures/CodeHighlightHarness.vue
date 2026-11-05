<script setup lang="ts">
import { ref } from "vue";
import CodeHighlight from "../../src/shared/CodeHighlight.vue";

const props = defineProps<{ initial: string; replacement: string }>();
const code = ref(props.initial);
</script>

<template>
  <button type="button" @click="code = props.replacement">Replace source</button>
  <CodeHighlight :code language="typescript" :show-line-numbers="true" />
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";
import json from "highlight.js/lib/languages/json";

const props = defineProps<{
  code: string;
  language: "xml" | "json";
}>();

const codeElement = ref<HTMLElement | null>(null);

hljs.registerLanguage("xml", xml);
hljs.registerLanguage("json", json);

function highlightCode() {
  const element = codeElement.value;
  if (!element) return;

  // Highlight.js escapes source text before adding its own markup.
  element.textContent = props.code;
  element.removeAttribute("data-highlighted");
  hljs.highlightElement(element);
}

onMounted(highlightCode);
watch(() => [props.code, props.language], highlightCode, { flush: "post" });
</script>

<template>
  <code ref="codeElement" :class="['musea-highlighted-code', `language-${language}`]" />
</template>

<style scoped>
.musea-highlighted-code {
  display: block;
  width: max-content;
  min-width: 100%;
  padding: 0;
  overflow: visible;
  background: transparent;
  font: inherit;
  line-height: inherit;
  white-space: pre;
  word-break: normal;
  tab-size: 2;
}
</style>

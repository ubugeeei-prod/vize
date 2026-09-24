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
  <code ref="codeElement" :class="`language-${language}`" />
</template>

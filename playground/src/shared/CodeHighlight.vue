<script setup lang="ts">
import { useTemplateRef, watch } from "vue";
import {
  codeToThemedHtmlLines,
  normalizePlainHtmlLines,
  type CodeHighlightLanguage,
} from "./codeHighlighting";

const props = defineProps<{
  code: string;
  language: CodeHighlightLanguage;
  showLineNumbers?: boolean;
  theme?: "dark" | "light";
}>();

const codeContentEl = useTemplateRef<HTMLDivElement>("codeContentEl");
const lineNumbersEl = useTemplateRef<HTMLDivElement>("lineNumbersEl");
let latestRenderId = 0;

function renderLineNumbers(count: number) {
  if (!lineNumbersEl.value) {
    return;
  }
  if (!props.showLineNumbers) {
    lineNumbersEl.value.innerHTML = "";
    return;
  }
  let html = "";
  for (let index = 0; index < count; index += 1) {
    html += `<span class="line-number">${index + 1}</span>`;
  }
  lineNumbersEl.value.innerHTML = html;
}

function renderCodeLines(lines: string[]) {
  if (!codeContentEl.value) {
    return;
  }
  let html = "";
  for (const line of lines) {
    html += `<div class="code-line">${line}</div>`;
  }
  codeContentEl.value.innerHTML = html;
  renderLineNumbers(lines.length);
}

async function highlight(renderId: number) {
  const nextLines = await codeToThemedHtmlLines(props.code, props.language);

  if (renderId !== latestRenderId) {
    return;
  }

  renderCodeLines(nextLines);
}

function render() {
  const renderId = ++latestRenderId;
  renderCodeLines(normalizePlainHtmlLines(props.code));
  void highlight(renderId);
}

function renderWhenReady() {
  if (!codeContentEl.value) {
    return;
  }
  if (props.showLineNumbers && !lineNumbersEl.value) {
    return;
  }
  render();
}

watch(
  [
    codeContentEl,
    lineNumbersEl,
    () => props.code,
    () => props.language,
    () => props.showLineNumbers,
  ],
  renderWhenReady,
  { immediate: true, flush: "post" },
);
</script>

<template>
  <div class="code-highlight" :class="{ 'with-line-numbers': props.showLineNumbers }">
    <div v-if="props.showLineNumbers" ref="lineNumbersEl" class="line-numbers"></div>
    <div ref="codeContentEl" class="code-content"></div>
  </div>
</template>

<style scoped>
.code-highlight {
  display: flex;
  font-family: "JetBrains Mono", monospace;
  font-size: 13px;
  line-height: 20px;
  border-radius: 4px;
  overflow: auto;
  background: var(--bg-secondary);
}

.line-numbers {
  display: flex;
  flex-direction: column;
  padding-top: 12px;
  padding-bottom: 12px;
  background: var(--bg-tertiary);
  border-right: 1px solid var(--border-color);
  user-select: none;
  flex-shrink: 0;
  position: sticky;
  left: 0;
}

.code-content {
  flex: 1;
  padding-top: 12px;
  padding-bottom: 12px;
  padding-left: 16px;
  padding-right: 16px;
  overflow-x: auto;
}

.code-highlight :deep(.line-number) {
  display: block;
  padding: 0 12px;
  text-align: right;
  color: var(--text-muted);
  line-height: 20px;
  height: 20px;
  box-sizing: border-box;
}

.code-highlight :deep(.code-line) {
  white-space: pre;
  line-height: 20px;
  height: 20px;
  box-sizing: border-box;
}

.code-highlight :deep(.code-line span) {
  color: var(--l);
  line-height: inherit;
}

body[data-theme="dark"] .code-highlight :deep(.code-line span) {
  color: var(--d);
}
</style>

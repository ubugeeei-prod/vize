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
  --code-line-height: 21px;

  display: flex;
  font-family: "JetBrains Mono", monospace;
  font-size: 13px;
  line-height: var(--code-line-height);
  border: 1px solid var(--code-border);
  border-radius: 6px;
  overflow: auto;
  background: var(--code-bg);
  color: var(--code-foreground);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);
}

.code-highlight :deep(.line-numbers) {
  display: flex;
  flex-direction: column;
  padding-top: 16px;
  padding-bottom: 16px;
  background: var(--code-gutter-bg);
  border-right: 1px solid var(--code-border);
  user-select: none;
  flex-shrink: 0;
  position: sticky;
  left: 0;
}

.code-highlight :deep(.code-content) {
  flex: 1;
  min-width: 0;
  padding: 16px 20px;
  overflow-x: auto;
}

.code-highlight.with-line-numbers :deep(.code-content) {
  padding-left: 16px;
}

.code-highlight :deep(.line-number) {
  display: block;
  padding: 0 14px;
  text-align: right;
  color: var(--code-line-number);
  line-height: var(--code-line-height);
  height: var(--code-line-height);
  box-sizing: border-box;
}

.code-highlight :deep(.code-line) {
  white-space: pre;
  color: var(--code-foreground);
  line-height: var(--code-line-height);
  min-height: var(--code-line-height);
  box-sizing: border-box;
}

.code-highlight :deep(.code-line span) {
  color: var(--l, var(--code-foreground));
  line-height: inherit;
}

body[data-theme="dark"] .code-highlight :deep(.code-line span) {
  color: var(--d, var(--code-foreground));
}
</style>

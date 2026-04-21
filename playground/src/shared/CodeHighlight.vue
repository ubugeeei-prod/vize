<script setup lang="ts">
import { useTemplateRef, watch } from "vue";
import { createHighlighter, type Highlighter, type ThemeRegistration } from "shiki";

const props = defineProps<{
  code: string;
  language: "javascript" | "json" | "css" | "html" | "typescript";
  showLineNumbers?: boolean;
  theme?: "dark" | "light";
}>();

// Custom themes — warm earthy tones matching brand
const vizeDarkTheme: ThemeRegistration = {
  name: "vize-dark",
  type: "dark",
  colors: {
    "editor.background": "#1a1a1a",
    "editor.foreground": "#E6E2D6",
  },
  tokenColors: [
    { scope: ["keyword", "storage.type", "storage.modifier"], settings: { foreground: "#D4BA92" } },
    { scope: ["entity.name.function", "support.function"], settings: { foreground: "#E2CBA6" } },
    {
      scope: ["entity.name.tag", "punctuation.definition.tag"],
      settings: { foreground: "#D0BA9E" },
    },
    { scope: ["entity.other.attribute-name"], settings: { foreground: "#9C9488" } },
    { scope: ["string", "string.quoted"], settings: { foreground: "#A8B5A0" } },
    { scope: ["constant.numeric", "constant.language"], settings: { foreground: "#DABA8C" } },
    { scope: ["variable", "variable.other"], settings: { foreground: "#E6E2D6" } },
    { scope: ["comment", "punctuation.definition.comment"], settings: { foreground: "#6B6560" } },
    { scope: ["punctuation", "meta.brace"], settings: { foreground: "#8A8478" } },
    { scope: ["entity.name.type", "support.type"], settings: { foreground: "#B8ADA0" } },
    {
      scope: ["meta.property-name", "support.type.property-name"],
      settings: { foreground: "#D0BA9E" },
    },
    {
      scope: ["meta.property-value", "support.constant.property-value"],
      settings: { foreground: "#A8B5A0" },
    },
  ],
};

const vizeLightTheme: ThemeRegistration = {
  name: "vize-light",
  type: "light",
  colors: {
    "editor.background": "#ddd9cd",
    "editor.foreground": "#121212",
  },
  tokenColors: [
    { scope: ["keyword", "storage.type", "storage.modifier"], settings: { foreground: "#73603E" } },
    { scope: ["entity.name.function", "support.function"], settings: { foreground: "#655232" } },
    {
      scope: ["entity.name.tag", "punctuation.definition.tag"],
      settings: { foreground: "#65573E" },
    },
    { scope: ["entity.other.attribute-name"], settings: { foreground: "#6B6050" } },
    { scope: ["string", "string.quoted"], settings: { foreground: "#5A6B50" } },
    { scope: ["constant.numeric", "constant.language"], settings: { foreground: "#735C2E" } },
    { scope: ["variable", "variable.other"], settings: { foreground: "#121212" } },
    { scope: ["comment", "punctuation.definition.comment"], settings: { foreground: "#9A9590" } },
    { scope: ["punctuation", "meta.brace"], settings: { foreground: "#6B6560" } },
    { scope: ["entity.name.type", "support.type"], settings: { foreground: "#6B5F50" } },
    {
      scope: ["meta.property-name", "support.type.property-name"],
      settings: { foreground: "#65573E" },
    },
    {
      scope: ["meta.property-value", "support.constant.property-value"],
      settings: { foreground: "#4A5F3E" },
    },
  ],
};

const codeContentEl = useTemplateRef<HTMLDivElement>("codeContentEl");
const lineNumbersEl = useTemplateRef<HTMLDivElement>("lineNumbersEl");
let latestRenderId = 0;

type SharedHighlighterState = {
  highlighter: Highlighter | null;
  highlighterPromise: Promise<Highlighter> | null;
};

type SharedGlobal = typeof globalThis & {
  __vizeCodeHighlightState?: SharedHighlighterState;
};

const sharedGlobal = globalThis as SharedGlobal;
const sharedState =
  sharedGlobal.__vizeCodeHighlightState ??
  (sharedGlobal.__vizeCodeHighlightState = {
    highlighter: null,
    highlighterPromise: null,
  });

function escapeHtml(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function normalizePlainLines(code: string): string[] {
  if (!code) {
    return [];
  }
  const lines = code.split("\n");
  if (lines.length > 0 && lines[lines.length - 1] === "") {
    lines.pop();
  }
  return lines.map((line) => (line ? escapeHtml(line) : "&nbsp;"));
}

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

async function initHighlighter() {
  if (sharedState.highlighter) {
    return sharedState.highlighter;
  }
  if (!sharedState.highlighterPromise) {
    sharedState.highlighterPromise = createHighlighter({
      themes: [vizeDarkTheme, vizeLightTheme],
      langs: ["javascript", "json", "css", "html", "typescript"],
    }).then((instance) => {
      sharedState.highlighter = instance;
      return instance;
    });
  }
  return sharedState.highlighterPromise;
}

async function highlight(renderId: number) {
  const hl = await initHighlighter();

  // Tokenize with both themes so CSS can switch colors without JS re-render
  const darkTokens = hl.codeToTokens(props.code, { lang: props.language, theme: "vize-dark" });
  const lightTokens = hl.codeToTokens(props.code, { lang: props.language, theme: "vize-light" });

  let darkLines = darkTokens.tokens;
  let lightLines = lightTokens.tokens;

  // Remove trailing empty line if present
  if (darkLines.length > 0 && darkLines[darkLines.length - 1].length === 0) {
    darkLines = darkLines.slice(0, -1);
  }
  if (lightLines.length > 0 && lightLines[lightLines.length - 1].length === 0) {
    lightLines = lightLines.slice(0, -1);
  }

  // Build HTML with both theme colors as CSS custom properties
  const nextLines = darkLines.map((lineTokens, lineIdx) => {
    if (lineTokens.length === 0) {
      return "&nbsp;";
    }
    return lineTokens
      .map((token, tokenIdx) => {
        const escaped = escapeHtml(token.content);
        const darkColor = token.color;
        const lightColor = lightLines[lineIdx]?.[tokenIdx]?.color ?? token.color;
        return `<span style="--d:${darkColor};--l:${lightColor}">${escaped}</span>`;
      })
      .join("");
  });

  if (renderId !== latestRenderId) {
    return;
  }

  renderCodeLines(nextLines);
}

function render() {
  const renderId = ++latestRenderId;
  renderCodeLines(normalizePlainLines(props.code));
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

:global(body[data-theme="dark"]) .code-highlight :deep(.code-line span) {
  color: var(--d);
}
</style>

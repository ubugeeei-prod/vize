<script setup lang="ts">
import { useTemplateRef, watch, onUnmounted, shallowRef, inject, type ComputedRef } from "vue";
import * as monaco from "monaco-editor";
import { configureMonaco, addVueCommentAction } from "./monacoConfig";
import {
  getScopeDecorationClass,
  offsetToPosition,
  getOverviewRulerColor,
  getScopeDecorationHoverMessage,
  type ScopeDecorationInfo,
} from "./scopeDecorations";

export interface Diagnostic {
  message: string;
  startLine: number;
  startColumn: number;
  endLine?: number;
  endColumn?: number;
  severity: "error" | "warning" | "info";
}

export interface ScopeDecoration extends ScopeDecorationInfo {}

/** A UTF-16 source range painted with `className` (provenance links). */
export interface EditorHighlight {
  start: number;
  end: number;
  className: string;
  /** Scroll the range into view when it appears. */
  reveal?: boolean;
}

const { modelValue, language, diagnostics, scopes, highlights, readOnly, theme } = defineProps<{
  modelValue: string;
  language: string;
  diagnostics?: Diagnostic[];
  scopes?: ScopeDecoration[];
  highlights?: EditorHighlight[];
  readOnly?: boolean;
  theme?: "dark" | "light";
}>();

const emit = defineEmits<{
  "update:modelValue": [string];
  /** The primary cursor moved; payload is its UTF-16 model offset. */
  cursor: [number];
}>();

const containerRef = useTemplateRef<HTMLDivElement>("containerRef");
const editorInstance = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null);
const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme", undefined as any);
const resolvedTheme = () => _injectedTheme?.value ?? theme ?? "light";

let scopeDecorationIds: string[] = [];
let highlightDecorationIds: string[] = [];

function applyHighlights(highlights: EditorHighlight[] | undefined) {
  const editor = editorInstance.value;
  const model = editor?.getModel();
  if (!editor || !model) return;
  const decorations = (highlights ?? []).map((highlight) => {
    const start = offsetToPosition(model, highlight.start);
    const end = offsetToPosition(model, highlight.end);
    return {
      range: new monaco.Range(start.lineNumber, start.column, end.lineNumber, end.column),
      options: { className: highlight.className, isWholeLine: false },
    };
  });
  highlightDecorationIds = editor.deltaDecorations(highlightDecorationIds, decorations);
  const revealed = (highlights ?? []).findIndex((highlight) => highlight.reveal);
  if (revealed !== -1) {
    editor.revealRangeInCenterIfOutsideViewport(decorations[revealed].range);
  }
}

function applyScopeDecorations(scopes: ScopeDecoration[] | undefined) {
  if (!editorInstance.value) return;
  const model = editorInstance.value.getModel();
  if (!model) return;

  if (!scopes || scopes.length === 0) {
    scopeDecorationIds = editorInstance.value.deltaDecorations(scopeDecorationIds, []);
    return;
  }

  const newDecorations: monaco.editor.IModelDeltaDecoration[] = scopes.map((scope) => {
    const startPos = offsetToPosition(model, scope.start);
    const endPos = offsetToPosition(model, scope.end);
    const className = getScopeDecorationClass(scope.kind);

    return {
      range: new monaco.Range(
        startPos.lineNumber,
        startPos.column,
        endPos.lineNumber,
        endPos.column,
      ),
      options: {
        className,
        hoverMessage: getScopeDecorationHoverMessage(scope),
        isWholeLine: false,
        overviewRuler: {
          color: getOverviewRulerColor(scope.kind),
          position: monaco.editor.OverviewRulerLane.Right,
        },
      },
    };
  });

  scopeDecorationIds = editorInstance.value.deltaDecorations(scopeDecorationIds, newDecorations);
}

function applyDiagnostics(diagnostics: Diagnostic[] | undefined) {
  if (!editorInstance.value) return;
  const model = editorInstance.value.getModel();
  if (!model) return;

  if (!diagnostics || diagnostics.length === 0) {
    monaco.editor.setModelMarkers(model, "vize", []);
    return;
  }

  const markers: monaco.editor.IMarkerData[] = diagnostics.map((d) => ({
    severity:
      d.severity === "error"
        ? monaco.MarkerSeverity.Error
        : d.severity === "warning"
          ? monaco.MarkerSeverity.Warning
          : monaco.MarkerSeverity.Info,
    message: d.message,
    startLineNumber: d.startLine,
    startColumn: d.startColumn,
    endLineNumber: d.endLine ?? d.startLine,
    endColumn: d.endColumn ?? d.startColumn + 1,
  }));

  monaco.editor.setModelMarkers(model, "vize", markers);
}

function setValue(value: string) {
  if (editorInstance.value) {
    editorInstance.value.setValue(value);
  }
}

function mountEditor(container: HTMLDivElement) {
  if (editorInstance.value) {
    return;
  }
  configureMonaco();

  const editor = monaco.editor.create(container, {
    value: modelValue,
    language,
    theme: resolvedTheme() === "light" ? "vue-light" : "vue-dark",
    fontSize: 14,
    fontFamily: "'JetBrains Mono', monospace",
    minimap: { enabled: false },
    lineNumbers: "on",
    scrollBeyondLastLine: false,
    padding: { top: 16 },
    automaticLayout: true,
    quickSuggestions: !readOnly,
    suggestOnTriggerCharacters: !readOnly,
    readOnly: readOnly ?? false,
    domReadOnly: readOnly ?? false,
  });
  editorInstance.value = editor;

  // Store on DOM for vite-plugin-vize workaround (ref binding may fail)
  (container as any).__monacoEditor = editor;

  editor.onDidChangeModelContent(() => {
    const value = editor.getValue() || "";
    emit("update:modelValue", value);
  });

  editor.onDidChangeCursorPosition((event) => {
    const model = editor.getModel();
    if (model) emit("cursor", model.getOffsetAt(event.position));
  });

  if (language === "vue") {
    addVueCommentAction(editor);
  }

  if (scopes && scopes.length > 0) {
    applyScopeDecorations(scopes);
  }

  if (diagnostics && diagnostics.length > 0) {
    applyDiagnostics(diagnostics);
  }

  if (highlights && highlights.length > 0) {
    applyHighlights(highlights);
  }
}

watch(
  containerRef,
  (container) => {
    if (container) {
      mountEditor(container);
    }
  },
  { immediate: true, flush: "post" },
);

onUnmounted(() => {
  editorInstance.value?.dispose();
});

watch(
  () => modelValue,
  (newValue) => {
    if (editorInstance.value && editorInstance.value.getValue() !== newValue) {
      editorInstance.value.setValue(newValue);
    }
  },
);

watch(
  () => _injectedTheme?.value ?? theme,
  (newTheme) => {
    if (editorInstance.value) {
      monaco.editor.setTheme(newTheme === "light" ? "vue-light" : "vue-dark");
    }
  },
);

watch(
  () => language,
  (newLanguage) => {
    if (editorInstance.value) {
      const model = editorInstance.value.getModel();
      if (model) {
        monaco.editor.setModelLanguage(model, newLanguage);
      }
    }
  },
);

watch(
  () => diagnostics,
  (diagnostics) => {
    applyDiagnostics(diagnostics);
  },
  { immediate: true, deep: true },
);

watch(
  () => scopes,
  (scopes) => {
    applyScopeDecorations(scopes);
  },
  { immediate: true, deep: true },
);

watch(
  () => highlights,
  (highlights) => {
    applyHighlights(highlights);
  },
  { deep: true },
);

defineExpose({
  applyDiagnostics,
  applyScopeDecorations,
  setValue,
});
</script>

<template>
  <div ref="containerRef" class="monaco-container"></div>
</template>

<style>
/* Scope decorations stay quiet in the editor, with color as a compact cue. */
.scope-decoration-setup,
.scope-decoration-plain,
.scope-decoration-extern,
.scope-decoration-vue,
.scope-decoration-client,
.scope-decoration-server,
.scope-decoration-universal,
.scope-decoration-vFor,
.scope-decoration-vSlot,
.scope-decoration-function,
.scope-decoration-closure,
.scope-decoration-block,
.scope-decoration-event,
.scope-decoration-callback,
.scope-decoration-computed,
.scope-decoration-watch,
.scope-decoration-mod,
.scope-decoration-default {
  background: color-mix(in srgb, var(--scope-accent) var(--scope-bg-strength, 14%), transparent);
  border-radius: 3px;
  box-shadow:
    inset 0 -1px 0 color-mix(in srgb, var(--scope-accent) 72%, transparent),
    inset 0 0 0 1px color-mix(in srgb, var(--scope-accent) 14%, transparent);
}

.scope-decoration-setup {
  --scope-accent: #2d6a35;
  --scope-bg-strength: 16%;
}

.scope-decoration-plain {
  --scope-accent: #8b7040;
}

.scope-decoration-extern {
  --scope-accent: #4a6b8a;
}

.scope-decoration-vue {
  --scope-accent: #3a7a70;
  --scope-bg-strength: 17%;
}

.scope-decoration-client {
  --scope-accent: #b85c38;
  --scope-bg-strength: 15%;
}

.scope-decoration-server {
  --scope-accent: #3f6fae;
  --scope-bg-strength: 15%;
}

.scope-decoration-universal {
  --scope-accent: #6b5090;
}

.scope-decoration-vFor {
  --scope-accent: #7c5fb2;
}

.scope-decoration-vSlot {
  --scope-accent: #a6537f;
}

.scope-decoration-function {
  --scope-accent: #287d87;
}

.scope-decoration-closure {
  --scope-accent: #9a6a22;
}

.scope-decoration-block {
  --scope-accent: #6b7280;
  --scope-bg-strength: 10%;
}

.scope-decoration-event {
  --scope-accent: #a6537f;
}

.scope-decoration-callback,
.scope-decoration-computed {
  --scope-accent: #b85c38;
}

.scope-decoration-watch {
  --scope-accent: #5d6cc1;
}

.scope-decoration-mod,
.scope-decoration-default {
  --scope-accent: #6b7280;
  --scope-bg-strength: 10%;
}

body[data-theme="dark"] .scope-decoration-setup {
  --scope-accent: #a8b5a0;
}

body[data-theme="dark"] .scope-decoration-plain,
body[data-theme="dark"] .scope-decoration-closure {
  --scope-accent: #d4ba92;
}

body[data-theme="dark"] .scope-decoration-extern,
body[data-theme="dark"] .scope-decoration-server {
  --scope-accent: #8aabca;
}

body[data-theme="dark"] .scope-decoration-vue,
body[data-theme="dark"] .scope-decoration-function {
  --scope-accent: #80bdb0;
}

body[data-theme="dark"] .scope-decoration-client,
body[data-theme="dark"] .scope-decoration-callback,
body[data-theme="dark"] .scope-decoration-computed {
  --scope-accent: #d8a082;
}

body[data-theme="dark"] .scope-decoration-universal,
body[data-theme="dark"] .scope-decoration-vFor,
body[data-theme="dark"] .scope-decoration-watch {
  --scope-accent: #b8a0d0;
}

body[data-theme="dark"] .scope-decoration-vSlot,
body[data-theme="dark"] .scope-decoration-event {
  --scope-accent: #d79abd;
}

:root {
  --editor-hover-bg: #f8f6ef;
  --editor-hover-border: rgba(74, 107, 138, 0.32);
  --editor-hover-shadow: rgba(18, 18, 18, 0.18);
  --editor-hover-soft-shadow: rgba(74, 107, 138, 0.1);
  --editor-hover-code-bg: #eeebe2;
  --editor-hover-rule: rgba(18, 18, 18, 0.12);
  --editor-hover-chip-bg: #e7f0ed;
  --editor-hover-chip-border: #b6d0ca;
  --editor-hover-chip-text: #245f57;
  --editor-hover-link: #315f82;
}

body[data-theme="dark"] {
  --editor-hover-bg: #18191c;
  --editor-hover-border: rgba(138, 171, 202, 0.34);
  --editor-hover-shadow: rgba(0, 0, 0, 0.42);
  --editor-hover-soft-shadow: rgba(128, 189, 176, 0.08);
  --editor-hover-code-bg: #22252a;
  --editor-hover-rule: rgba(230, 226, 214, 0.14);
  --editor-hover-chip-bg: #1f3432;
  --editor-hover-chip-border: #426760;
  --editor-hover-chip-text: #b8ded5;
  --editor-hover-link: #a5c8e3;
}

.monaco-hover {
  width: min(440px, calc(100vw - 32px)) !important;
  max-width: min(440px, calc(100vw - 32px)) !important;
  border: 1px solid var(--editor-hover-border) !important;
  border-radius: 8px !important;
  background: var(--editor-hover-bg) !important;
  box-shadow:
    0 18px 46px var(--editor-hover-shadow),
    0 0 0 1px var(--editor-hover-soft-shadow) !important;
  overflow: hidden !important;
}

.monaco-hover::before {
  content: "";
  display: block;
  height: 2px;
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--color-info) 86%, transparent),
    color-mix(in srgb, var(--color-teal) 82%, transparent),
    color-mix(in srgb, var(--color-notice) 76%, transparent)
  );
}

.monaco-hover .monaco-hover-content,
.monaco-hover .hover-contents {
  color: var(--text-primary) !important;
  font-family: var(--font-sans);
  font-size: 12px !important;
  line-height: 1.55 !important;
  max-width: 100% !important;
}

.monaco-hover .monaco-hover-content {
  padding: 10px 12px 12px !important;
  width: auto !important;
}

.monaco-hover .markdown-hover {
  font-size: 12px !important;
  overflow-wrap: anywhere;
  white-space: normal;
}

.monaco-hover .markdown-hover .hover-contents:not(:first-child) {
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px solid var(--editor-hover-rule);
}

.monaco-hover .markdown-hover p,
.monaco-hover .markdown-hover .rendered-markdown p {
  margin: 0 0 8px;
}

.monaco-hover .markdown-hover p:last-child,
.monaco-hover .markdown-hover .rendered-markdown p:last-child {
  margin-bottom: 0;
}

.monaco-hover .markdown-hover hr {
  height: 1px;
  margin: 10px -2px;
  border: 0;
  background: var(--editor-hover-rule);
}

.monaco-hover .markdown-hover strong {
  color: var(--text-primary);
  font-weight: 700;
}

.monaco-hover .markdown-hover em {
  color: var(--text-secondary);
  font-style: normal;
}

.monaco-hover .markdown-hover ul {
  margin: 7px 0 0 16px;
  padding: 0;
}

.monaco-hover .markdown-hover li {
  margin: 3px 0;
}

.monaco-hover .markdown-hover code {
  padding: 1px 5px;
  border: 1px solid var(--editor-hover-chip-border);
  border-radius: 5px;
  background: var(--editor-hover-chip-bg);
  color: var(--editor-hover-chip-text);
  font-family: var(--font-mono);
  font-size: 11px;
}

.monaco-hover .markdown-hover pre {
  margin: 8px 0 10px;
  padding: 9px 10px;
  border: 1px solid color-mix(in srgb, var(--color-info) 24%, transparent);
  border-radius: 7px;
  background: var(--editor-hover-code-bg);
}

.monaco-hover .markdown-hover .rendered-markdown div[data-code] {
  margin: 8px 0 10px;
  padding: 9px 10px;
  border: 1px solid color-mix(in srgb, var(--color-info) 24%, transparent);
  border-radius: 7px;
  background: var(--editor-hover-code-bg);
}

.monaco-hover .markdown-hover pre code {
  padding: 0;
  border: 0;
  border-radius: 0;
  background: transparent;
  color: var(--text-primary);
  font-size: 11.5px;
  line-height: 1.5;
}

.monaco-hover .markdown-hover .monaco-tokenized-source {
  font-size: 11.5px !important;
  line-height: 1.45 !important;
}

.monaco-hover .markdown-hover .monaco-tokenized-source span {
  color: var(--text-primary) !important;
}

.monaco-hover .markdown-hover a {
  color: var(--editor-hover-link);
  font-weight: 600;
  text-decoration: none;
}

.monaco-hover .markdown-hover a:hover {
  text-decoration: underline;
}
</style>

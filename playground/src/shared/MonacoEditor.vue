<script setup lang="ts">
import "./MonacoEditor.css";
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

<style scoped>
/* NOTE: Main .monaco-container styles are in styles.css (global)
   due to Vize compiler not extracting scoped styles in production builds.
   See: styles.css -> Monaco Container section */
</style>

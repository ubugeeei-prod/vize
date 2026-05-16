import {
  ref,
  computed,
  watch,
  nextTick,
  getCurrentInstance,
  onMounted,
  onUnmounted,
  inject,
  type ComputedRef,
} from "vue";
import type { Diagnostic } from "../../shared/MonacoEditor.vue";
import type { WasmModule, CroquisResult, BindingDisplay } from "../../wasm/index";
import { ANALYSIS_PRESET } from "../../shared/presets/croquis";
import { parseVirLines } from "./useVirTokenizer";

interface EditorRef {
  applyScopeDecorations: (scopes: unknown[]) => void;
}

export function useCroquisAnalysis(getCompilerProp: () => WasmModule | null) {
  const _injectedTheme = inject<ComputedRef<"dark" | "light">>("theme");
  const theme = computed<"dark" | "light">(() => _injectedTheme?.value ?? "light");

  const source = ref(ANALYSIS_PRESET);
  const analysisResult = ref<CroquisResult | null>(null);
  const error = ref<string | null>(null);
  const activeTab = ref<"vir" | "stats" | "bindings" | "scopes" | "diagnostics">("vir");
  const showScopeVisualization = ref(true);
  const editorRef = ref<EditorRef | null>(null);
  const analysisTime = ref<number | null>(null);

  const summary = computed(() => analysisResult.value?.croquis);
  const scopes = computed(() => summary.value?.scopes || []);

  const scopeDecorations = computed(() => {
    if (!scopes.value) return [];
    return scopes.value
      .filter((scope) => {
        if (scope.kind === "mod") return false;
        if (scope.start === 0 && scope.end === 0) return false;
        return true;
      })
      .map((scope) => ({
        start: scope.start,
        end: scope.end,
        kind: scope.kind,
        kindStr: scope.kindStr,
      }));
  });

  const editorScopes = computed(() => (showScopeVisualization.value ? scopeDecorations.value : []));

  watch(editorScopes, async (newScopes) => {
    await nextTick();
    editorRef.value?.applyScopeDecorations(newScopes);
  });

  const _instance = getCurrentInstance();
  function getCompiler(): WasmModule | null {
    return (
      getCompilerProp() ??
      ((_instance?.vnode?.props as Record<string, unknown>)?.compiler as WasmModule | null) ??
      null
    );
  }

  async function analyze() {
    const compiler = getCompiler();
    if (!compiler) {
      error.value = "Compiler not loaded";
      return;
    }

    error.value = null;
    const startTime = performance.now();

    try {
      const result = compiler.analyzeSfc(source.value, {
        filename: "Component.vue",
      });
      analysisResult.value = result;
      analysisTime.value = performance.now() - startTime;

      await nextTick();
      if (editorRef.value && showScopeVisualization.value) {
        editorRef.value.applyScopeDecorations(scopeDecorations.value);
      }
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
      analysisResult.value = null;
    }
  }

  let analyzeTimer: ReturnType<typeof setTimeout> | null = null;
  watch(
    source,
    () => {
      if (analyzeTimer) clearTimeout(analyzeTimer);
      analyzeTimer = setTimeout(analyze, 300);
    },
    { immediate: false },
  );

  watch(
    () => getCompilerProp(),
    () => {
      if (getCompiler()) void analyze();
    },
  );

  let compilerPollTimer: ReturnType<typeof setInterval> | null = null;
  onMounted(() => {
    if (getCompiler()) {
      void analyze();
    } else {
      compilerPollTimer = setInterval(() => {
        if (getCompiler()) {
          if (compilerPollTimer) clearInterval(compilerPollTimer);
          compilerPollTimer = null;
          void analyze();
        }
      }, 200);
    }
  });
  onUnmounted(() => {
    if (compilerPollTimer) clearInterval(compilerPollTimer);
  });

  const bindings = computed(() => summary.value?.bindings || []);
  const macros = computed(() => summary.value?.macros || []);
  const css = computed(() => summary.value?.css);
  const typeExports = computed(() => summary.value?.typeExports || []);
  const invalidExports = computed(() => summary.value?.invalidExports || []);
  const diagnostics = computed(() => analysisResult.value?.diagnostics || []);
  const stats = computed(() => summary.value?.stats);

  function offsetToLineColumn(content: string, offset: number): { line: number; column: number } {
    const beforeOffset = content.substring(0, offset);
    const lines = beforeOffset.split("\n");
    return {
      line: lines.length,
      column: lines[lines.length - 1].length + 1,
    };
  }

  const monacoDiagnostics = computed<Diagnostic[]>(() => {
    return diagnostics.value.map((d) => {
      const start = offsetToLineColumn(source.value, d.start);
      const end = offsetToLineColumn(source.value, d.end);
      return {
        message: d.message,
        startLine: start.line,
        startColumn: start.column,
        endLine: end.line,
        endColumn: end.column,
        severity: d.severity === "hint" ? "info" : (d.severity as "error" | "warning" | "info"),
      };
    });
  });

  const bindingsBySource = computed(() => {
    const groups: Record<string, BindingDisplay[]> = {};
    for (const binding of bindings.value) {
      const src = binding.source || "unknown";
      if (!groups[src]) groups[src] = [];
      groups[src].push(binding);
    }
    return groups;
  });

  const virText = computed(() => analysisResult.value?.vir || "");
  const virLines = computed(() => parseVirLines(virText.value));

  return {
    theme,
    source,
    analysisResult,
    error,
    activeTab,
    showScopeVisualization,
    editorRef,
    analysisTime,
    scopes,
    editorScopes,
    bindings,
    macros,
    css,
    typeExports,
    invalidExports,
    diagnostics,
    stats,
    monacoDiagnostics,
    bindingsBySource,
    virLines,
  };
}

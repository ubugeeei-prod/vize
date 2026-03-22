import { type Ref } from "vue";
import * as monaco from "monaco-editor";
import type { LintResult, LintDiagnostic, LintRule, LintPreset } from "../../wasm/index";
import { getWasm } from "../../wasm/index";
import { mdiAlert, mdiCloseCircle } from "@mdi/js";
import type { LocaleCode } from "./useLocale";

interface Diagnostic {
  message: string;
  help?: string;
  startLine: number;
  startColumn: number;
  endLine?: number;
  endColumn?: number;
  severity: "error" | "warning" | "info";
}

interface EditorRef {
  applyDiagnostics(diagnostics: Diagnostic[]): void;
}

export interface UseLintingOptions {
  source: Ref<string>;
  selectedPreset: Ref<LintPreset>;
  enabledRules: Ref<Set<string>>;
  severityOverrides: Ref<Map<string, "error" | "warning" | "off">>;
  currentLocale: Ref<LocaleCode>;
  editorRef: Ref<EditorRef | null>;
  lintResult: Ref<LintResult | null>;
  rules: Ref<LintRule[]>;
  error: Ref<string | null>;
  lintTime: Ref<number | null>;
  initializeRuleState: () => void;
}

export function useLinting(options: UseLintingOptions) {
  const {
    source,
    selectedPreset,
    enabledRules,
    severityOverrides,
    currentLocale,
    editorRef,
    lintResult,
    rules,
    error,
    lintTime,
    initializeRuleState,
  } = options;

  // Hover provider for showing diagnostic help in Monaco
  let hoverProviderDisposable: monaco.IDisposable | null = null;

  /** Run the linter on the current source */
  function lint() {
    const compiler = getWasm();
    if (!compiler) return;

    const startTime = performance.now();
    error.value = null;

    try {
      const result = compiler.lintSfc(source.value, {
        filename: "example.vue",
        preset: selectedPreset.value,
        enabledRules: Array.from(enabledRules.value),
        severityOverrides: Object.fromEntries(severityOverrides.value),
        locale: currentLocale.value,
      });
      lintResult.value = result;
      lintTime.value = performance.now() - startTime;

      // Directly apply diagnostics to editor (workaround for vite-plugin-vize reactivity issue)
      const diags: Diagnostic[] =
        result?.diagnostics?.map((d) => ({
          message: d.message,
          help: d.help,
          startLine: d.location.start.line,
          startColumn: d.location.start.column,
          endLine: d.location.end?.line ?? d.location.start.line,
          endColumn: d.location.end?.column ?? d.location.start.column + 1,
          severity: d.severity,
        })) ?? [];
      editorRef.value?.applyDiagnostics(diags);
    } catch (e) {
      console.error("[Patina] lintSfc error:", e);
      error.value = e instanceof Error ? e.message : String(e);
      lintResult.value = null;
    }
  }

  /** Load available lint rules from the compiler */
  function loadRules() {
    const compiler = getWasm();
    if (!compiler) return;

    try {
      rules.value = compiler.getLintRules();
      initializeRuleState();
    } catch (e) {
      console.error("Failed to load rules:", e);
    }
  }

  /** Find diagnostic at a given position (for hover provider) */
  function findDiagnosticAtPosition(line: number, col: number): LintDiagnostic | null {
    if (!lintResult.value?.diagnostics) return null;

    for (const diag of lintResult.value.diagnostics) {
      const startLine = diag.location.start.line;
      const startCol = diag.location.start.column;
      const endLine = diag.location.end?.line ?? startLine;
      const endCol = diag.location.end?.column ?? startCol + 1;

      // Check if position is within diagnostic range
      if (line > startLine && line < endLine) {
        return diag;
      }
      if (line === startLine && line === endLine) {
        if (col >= startCol && col <= endCol) {
          return diag;
        }
      }
      if (line === startLine && line < endLine && col >= startCol) {
        return diag;
      }
      if (line === endLine && line > startLine && col <= endCol) {
        return diag;
      }
    }
    return null;
  }

  /** Register Monaco hover provider to show diagnostic help on hover */
  function registerHoverProvider() {
    if (hoverProviderDisposable) {
      hoverProviderDisposable.dispose();
    }

    hoverProviderDisposable = monaco.languages.registerHoverProvider("vue", {
      provideHover(model, position) {
        const contents: monaco.IMarkdownString[] = [];

        // Check if hovering over a diagnostic
        const diag = findDiagnosticAtPosition(position.lineNumber, position.column);
        if (diag) {
          // Add diagnostic message with severity indicator
          const severityLabel = diag.severity === "error" ? "Error" : "Warning";
          contents.push({
            value: `**[${severityLabel}]** \`${diag.rule}\`\n\n${diag.message}`,
          });

          // Add help if available (render as markdown)
          if (diag.help) {
            contents.push({
              value: `---\n**Hint**\n\n${diag.help}`,
            });
          }
        }

        if (contents.length === 0) return null;

        return {
          contents,
        };
      },
    });
  }

  /** Dispose the hover provider (call in onUnmounted) */
  function disposeHoverProvider() {
    if (hoverProviderDisposable) {
      hoverProviderDisposable.dispose();
      hoverProviderDisposable = null;
    }
  }

  /** Get the MDI icon path for a severity level */
  function getSeverityIcon(severity: "error" | "warning"): string {
    return severity === "error" ? mdiCloseCircle : mdiAlert;
  }

  /** Get the CSS class for a severity level */
  function getSeverityClass(severity: "error" | "warning"): string {
    return severity === "error" ? "severity-error" : "severity-warning";
  }

  return {
    lint,
    loadRules,
    findDiagnosticAtPosition,
    registerHoverProvider,
    disposeHoverProvider,
    getSeverityIcon,
    getSeverityClass,
  };
}

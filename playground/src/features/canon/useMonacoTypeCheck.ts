import { ref, computed } from "vue";
import * as monaco from "monaco-editor";
import type { TypeCheckResult, TypeCheckCapabilities } from "../../wasm/index";
import { VUE_GLOBALS_DECLARATIONS } from "./vueTypeDeclarations";
import { mapDiagnosticsToSource, type Diagnostic, type TsDiagnostic } from "./diagnosticMapping";
import type { UseMonacoTypeCheckOptions } from "./typeCheckOptions";
import type { VirtualTsMapping } from "../../wasm/types/analysis";
import { mapSourceOffset, parseSourceMap } from "./sourceMappings";

export function useMonacoTypeCheck({
  source,
  experimentals,
  compiler: getCompiler,
  strictMode,
  checkProps,
  checkEmits,
  checkTemplateBindings,
  useMonacoTs,
}: UseMonacoTypeCheckOptions) {
  const typeCheckResult = ref<TypeCheckResult | null>(null);
  const capabilities = ref<TypeCheckCapabilities | null>(null);
  const error = ref<string | null>(null);
  const checkTime = ref<number | null>(null);
  const tsDiagnostics = ref<Diagnostic[]>([]);

  let virtualTsModel: monaco.editor.ITextModel | null = null;
  let cachedSourceMap: VirtualTsMapping[] = [];
  let checkVersion = 0;
  let patternHelpers: monaco.IDisposable | null = null;
  let patternHelpersSource: string | null = null;
  let hoverProviderDisposable: monaco.IDisposable | null = null;
  let hasConfiguredTypeScript = false;
  let isTypeScriptReady = false;
  let typeScriptReadyPromise: Promise<boolean> | null = null;

  const VIRTUAL_TS_URI = monaco.Uri.parse("ts:virtual-sfc.ts");

  function applyTypeScriptDefaults() {
    if (hasConfiguredTypeScript) return;

    monaco.typescript.typescriptDefaults.setCompilerOptions({
      target: monaco.typescript.ScriptTarget.ESNext,
      module: monaco.typescript.ModuleKind.ESNext,
      moduleResolution: monaco.typescript.ModuleResolutionKind.NodeJs,
      strict: strictMode.value,
      noEmit: true,
      allowJs: true,
      checkJs: false,
      esModuleInterop: true,
      skipLibCheck: true,
      jsx: monaco.typescript.JsxEmit.Preserve,
      noImplicitAny: false,
      strictNullChecks: strictMode.value,
    });
    monaco.typescript.typescriptDefaults.addExtraLib(VUE_GLOBALS_DECLARATIONS, "vue.d.ts");
    hasConfiguredTypeScript = true;
  }

  function isTypeScriptRegistrationError(error: unknown): boolean {
    return String(error).includes("TypeScript not registered");
  }

  async function waitForTypeScriptReady(): Promise<boolean> {
    if (!virtualTsModel) {
      virtualTsModel = monaco.editor.createModel("", "typescript", VIRTUAL_TS_URI);
    }

    for (let attempt = 0; attempt < 10; attempt++) {
      try {
        const worker = await monaco.typescript.getTypeScriptWorker();
        await worker(VIRTUAL_TS_URI);
        return true;
      } catch (error) {
        if (!isTypeScriptRegistrationError(error)) {
          throw error;
        }
        await new Promise((resolve) => setTimeout(resolve, 50));
      }
    }

    return false;
  }

  // Configure Monaco TypeScript compiler
  async function configureTypeScript(): Promise<boolean> {
    applyTypeScriptDefaults();

    if (isTypeScriptReady) return true;
    if (typeScriptReadyPromise) return typeScriptReadyPromise;

    typeScriptReadyPromise = waitForTypeScriptReady()
      .then((ready) => {
        isTypeScriptReady = ready;
        return ready;
      })
      .finally(() => {
        typeScriptReadyPromise = null;
      });

    return typeScriptReadyPromise;
  }

  async function ensureTypeScriptReady(): Promise<boolean> {
    if (isTypeScriptReady) return true;
    return configureTypeScript();
  }

  // Get hover info from TypeScript at a given position in Virtual TS
  async function getTypeScriptHover(genOffset: number): Promise<string | null> {
    if (!virtualTsModel) return null;
    if (!(await ensureTypeScriptReady())) return null;
    try {
      const worker = await monaco.typescript.getTypeScriptWorker();
      const client = await worker(VIRTUAL_TS_URI);
      const quickInfo = await client.getQuickInfoAtPosition(VIRTUAL_TS_URI.toString(), genOffset);
      if (!quickInfo) return null;

      const parts: string[] = [];
      if (quickInfo.displayParts) {
        const displayText = quickInfo.displayParts.map((p: { text: string }) => p.text).join("");
        if (displayText) parts.push("```typescript\n" + displayText + "\n```");
      }
      if (quickInfo.documentation) {
        const docs = quickInfo.documentation.map((d: { text: string }) => d.text).join("\n");
        if (docs) parts.push(docs);
      }
      return parts.length > 0 ? parts.join("\n\n") : null;
    } catch (e) {
      if (isTypeScriptRegistrationError(e)) {
        isTypeScriptReady = false;
        return null;
      }
      console.error("Failed to get TypeScript hover:", e);
      return null;
    }
  }

  function mapSourceToGenerated(srcOffset: number): number | null {
    return mapSourceOffset(srcOffset, cachedSourceMap);
  }

  function findDiagnosticAtPosition(line: number, col: number): Diagnostic | null {
    for (const diag of diagnostics.value) {
      const startLine = diag.startLine;
      const startCol = diag.startColumn;
      const endLine = diag.endLine ?? startLine;
      const endCol = diag.endColumn ?? startCol + 1;

      if (line > startLine && line < endLine) return diag;
      if (line === startLine && line === endLine && col >= startCol && col <= endCol) return diag;
      if (line === startLine && line < endLine && col >= startCol) return diag;
      if (line === endLine && line > startLine && col <= endCol) return diag;
    }
    return null;
  }

  function getSeverityLabel(severity: Diagnostic["severity"]): string {
    if (severity === "error") return "Error";
    if (severity === "warning") return "Warning";
    return "Info";
  }

  function getDiagnosticRangeLabel(diag: Diagnostic): string {
    const endLine = diag.endLine ?? diag.startLine;
    const endColumn = diag.endColumn ?? diag.startColumn + 1;
    if (endLine === diag.startLine) {
      return `Line ${diag.startLine}, columns ${diag.startColumn}-${endColumn}`;
    }
    return `Lines ${diag.startLine}:${diag.startColumn}-${endLine}:${endColumn}`;
  }

  function getDiagnosticSourceLabel(diag: Diagnostic): string {
    if (diag.code) return `TypeScript TS${diag.code}`;
    if (diag.message.startsWith("[vize:")) return "Vize type checker";
    return "Type analysis";
  }

  function buildDiagnosticHover(diag: Diagnostic): monaco.IMarkdownString[] {
    const contents: monaco.IMarkdownString[] = [
      {
        value: [
          `**${getSeverityLabel(diag.severity)}**`,
          "",
          `_${getDiagnosticSourceLabel(diag)}_ - ${getDiagnosticRangeLabel(diag)}`,
          "",
          diag.message,
        ].join("\n"),
      },
    ];

    if (diag.help) {
      contents.push({
        value: ["---", "**How to fix**", "", diag.help].join("\n"),
      });
    }

    return contents;
  }

  function registerHoverProvider() {
    if (hoverProviderDisposable) hoverProviderDisposable.dispose();

    hoverProviderDisposable = monaco.languages.registerHoverProvider("vue", {
      async provideHover(model, position) {
        const contents: monaco.IMarkdownString[] = [];

        const diag = findDiagnosticAtPosition(position.lineNumber, position.column);
        if (diag) {
          contents.push(...buildDiagnosticHover(diag));
        }

        const srcOffset = model.getOffsetAt(position);
        const genOffset = mapSourceToGenerated(srcOffset);
        if (genOffset !== null) {
          const hoverContent = await getTypeScriptHover(genOffset);
          if (hoverContent) {
            if (contents.length > 0) contents.push({ value: "---" });
            contents.push({
              value: ["**TypeScript quick info**", "", hoverContent].join("\n"),
            });
          }
        }

        if (contents.length === 0) return null;
        return { contents };
      },
    });
  }

  // Get TypeScript diagnostics from Monaco Worker
  async function getTypeScriptDiagnostics(virtualTs: string): Promise<TsDiagnostic[]> {
    if (!virtualTs) return [];

    if (virtualTsModel) {
      virtualTsModel.setValue(virtualTs);
    } else {
      virtualTsModel = monaco.editor.createModel(virtualTs, "typescript", VIRTUAL_TS_URI);
    }

    if (!(await ensureTypeScriptReady())) return [];

    try {
      const worker = await monaco.typescript.getTypeScriptWorker();
      const client = await worker(VIRTUAL_TS_URI);

      const [semanticDiags, syntacticDiags] = await Promise.all([
        client.getSemanticDiagnostics(VIRTUAL_TS_URI.toString()),
        client.getSyntacticDiagnostics(VIRTUAL_TS_URI.toString()),
      ]);

      return [...syntacticDiags, ...semanticDiags] as TsDiagnostic[];
    } catch (e) {
      if (isTypeScriptRegistrationError(e)) {
        isTypeScriptReady = false;
        return [];
      }
      console.error("Failed to get TypeScript diagnostics:", e);
      return [];
    }
  }

  function getPositionFromOffset(src: string, offset: number): { line: number; column: number } {
    const lines = src.substring(0, offset).split("\n");
    return { line: lines.length, column: lines[lines.length - 1].length + 1 };
  }

  // Combined diagnostics: WASM + Monaco TS Worker
  const diagnostics = computed((): Diagnostic[] => {
    const wasmDiags: Diagnostic[] = [];

    if (typeCheckResult.value?.diagnostics) {
      for (const d of typeCheckResult.value.diagnostics) {
        const startPos = getPositionFromOffset(source.value, d.start);
        const endPos = getPositionFromOffset(source.value, d.end);
        const message = d.code ? `[vize:${d.code}] ${d.message}` : `[vize] ${d.message}`;
        wasmDiags.push({
          message,
          help: d.help,
          startLine: startPos.line,
          startColumn: startPos.column,
          endLine: endPos.line,
          endColumn: endPos.column,
          severity:
            d.severity === "error" ? "error" : d.severity === "warning" ? "warning" : "info",
        });
      }
    }

    if (useMonacoTs.value) {
      return [...wasmDiags, ...tsDiagnostics.value];
    }

    return wasmDiags;
  });

  const errorCount = computed(() => {
    const wasmErrors = typeCheckResult.value?.errorCount ?? 0;
    const tsErrors = tsDiagnostics.value.filter((d) => d.severity === "error").length;
    return wasmErrors + tsErrors;
  });

  const warningCount = computed(() => {
    const wasmWarnings = typeCheckResult.value?.warningCount ?? 0;
    const tsWarnings = tsDiagnostics.value.filter((d) => d.severity === "warning").length;
    return wasmWarnings + tsWarnings;
  });

  async function typeCheck() {
    const comp = getCompiler();
    if (!comp) return;

    const startTime = performance.now();
    const version = ++checkVersion;
    const checkedSource = source.value;
    error.value = null;
    tsDiagnostics.value = [];

    try {
      const result = comp.typeCheck(checkedSource, {
        filename: "example.vue",
        strict: strictMode.value,
        includeVirtualTs: true,
        checkProps: checkProps.value,
        checkEmits: checkEmits.value,
        checkTemplateBindings: checkTemplateBindings.value,
        ...experimentals.value,
      });
      typeCheckResult.value = result;
      if (patternHelpersSource !== (result.virtualTsHelpers ?? null)) {
        patternHelpers?.dispose();
        patternHelpersSource = result.virtualTsHelpers ?? null;
        patternHelpers = patternHelpersSource
          ? monaco.typescript.typescriptDefaults.addExtraLib(
              patternHelpersSource,
              "vize-patterns.d.ts",
            )
          : null;
      }

      if (useMonacoTs.value && result.virtualTs) {
        const mappings = result.sourceMappings ?? parseSourceMap(result.virtualTs);
        cachedSourceMap = mappings;
        const tsDiags = await getTypeScriptDiagnostics(result.virtualTs);
        if (version !== checkVersion || source.value !== checkedSource) return;
        tsDiagnostics.value = mapDiagnosticsToSource(
          tsDiags,
          mappings,
          checkedSource,
          result.virtualTs,
        );
      } else {
        tsDiagnostics.value = [];
        cachedSourceMap = [];
      }

      checkTime.value = performance.now() - startTime;
    } catch (e) {
      if (version !== checkVersion) return;
      error.value = e instanceof Error ? e.message : String(e);
      typeCheckResult.value = null;
      tsDiagnostics.value = [];
    }
  }

  function loadCapabilities() {
    const comp = getCompiler();
    if (!comp) return;
    try {
      capabilities.value = comp.getTypeCheckCapabilities();
    } catch (e) {
      console.error("Failed to load capabilities:", e);
    }
  }

  function dispose() {
    patternHelpers?.dispose();
    checkVersion++;
    if (virtualTsModel) {
      virtualTsModel.dispose();
      virtualTsModel = null;
    }
    if (hoverProviderDisposable) {
      hoverProviderDisposable.dispose();
      hoverProviderDisposable = null;
    }
  }

  return {
    typeCheckResult,
    capabilities,
    error,
    checkTime,
    diagnostics,
    errorCount,
    warningCount,
    configureTypeScript,
    registerHoverProvider,
    typeCheck,
    loadCapabilities,
    dispose,
  };
}

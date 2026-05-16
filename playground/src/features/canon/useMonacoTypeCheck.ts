import { ref, computed, type Ref } from "vue";
import * as monaco from "monaco-editor";
import type { WasmModule, TypeCheckResult, TypeCheckCapabilities } from "../../wasm/index";
import { VUE_GLOBALS_DECLARATIONS } from "./vueTypeDeclarations";
import { generateHelp } from "./generateHelp";

interface Diagnostic {
  message: string;
  help?: string;
  code?: number;
  startLine: number;
  startColumn: number;
  endLine?: number;
  endColumn?: number;
  severity: "error" | "warning" | "info";
}

interface SourceMapEntry {
  genStart: number;
  genEnd: number;
  srcStart: number;
  srcEnd: number;
}

interface TsDiagnostic {
  start: number;
  length: number;
  messageText: string | { messageText: string };
  message?: string;
  category: number;
  code: number;
}

interface UseMonacoTypeCheckOptions {
  source: Ref<string>;
  compiler: () => WasmModule | null;
  strictMode: Ref<boolean>;
  includeVirtualTs: Ref<boolean>;
  checkProps: Ref<boolean>;
  checkEmits: Ref<boolean>;
  checkTemplateBindings: Ref<boolean>;
  useMonacoTs: Ref<boolean>;
}

export function useMonacoTypeCheck({
  source,
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
  let cachedSourceMap: SourceMapEntry[] = [];
  let hoverProviderDisposable: monaco.IDisposable | null = null;
  let hasConfiguredTypeScript = false;
  let isTypeScriptReady = false;
  let typeScriptReadyPromise: Promise<boolean> | null = null;

  const VIRTUAL_TS_URI = monaco.Uri.parse("ts:virtual-sfc.ts");

  function applyTypeScriptDefaults() {
    if (hasConfiguredTypeScript) return;

    monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
      target: monaco.languages.typescript.ScriptTarget.ESNext,
      module: monaco.languages.typescript.ModuleKind.ESNext,
      moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
      strict: strictMode.value,
      noEmit: true,
      allowJs: true,
      checkJs: false,
      esModuleInterop: true,
      skipLibCheck: true,
      jsx: monaco.languages.typescript.JsxEmit.Preserve,
      noImplicitAny: false,
      strictNullChecks: strictMode.value,
    });
    monaco.languages.typescript.typescriptDefaults.addExtraLib(
      VUE_GLOBALS_DECLARATIONS,
      "vue.d.ts",
    );
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
        const worker = await monaco.languages.typescript.getTypeScriptWorker();
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
      const worker = await monaco.languages.typescript.getTypeScriptWorker();
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
    for (const entry of cachedSourceMap) {
      if (srcOffset >= entry.srcStart && srcOffset < entry.srcEnd) {
        return entry.genStart + (srcOffset - entry.srcStart);
      }
    }
    return null;
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

  function registerHoverProvider() {
    if (hoverProviderDisposable) hoverProviderDisposable.dispose();

    hoverProviderDisposable = monaco.languages.registerHoverProvider("vue", {
      async provideHover(model, position) {
        const contents: monaco.IMarkdownString[] = [];

        const diag = findDiagnosticAtPosition(position.lineNumber, position.column);
        if (diag) {
          const severityLabel =
            diag.severity === "error" ? "Error" : diag.severity === "warning" ? "Warning" : "Info";
          contents.push({ value: `**[${severityLabel}]** ${diag.message}` });
          if (diag.help) {
            contents.push({ value: `---\n**Hint**\n\n${diag.help}` });
          }
        }

        const srcOffset = model.getOffsetAt(position);
        const genOffset = mapSourceToGenerated(srcOffset);
        if (genOffset !== null) {
          const hoverContent = await getTypeScriptHover(genOffset);
          if (hoverContent) {
            if (contents.length > 0) contents.push({ value: "---" });
            contents.push({ value: hoverContent });
          }
        }

        if (contents.length === 0) return null;
        return { contents };
      },
    });
  }

  // Get TypeScript diagnostics from Monaco Worker
  async function getTypeScriptDiagnostics(virtualTs: string): Promise<Diagnostic[]> {
    if (!virtualTs) return [];

    if (virtualTsModel) {
      virtualTsModel.setValue(virtualTs);
    } else {
      virtualTsModel = monaco.editor.createModel(virtualTs, "typescript", VIRTUAL_TS_URI);
    }

    if (!(await ensureTypeScriptReady())) return [];

    try {
      const worker = await monaco.languages.typescript.getTypeScriptWorker();
      const client = await worker(VIRTUAL_TS_URI);

      const [semanticDiags, syntacticDiags] = await Promise.all([
        client.getSemanticDiagnostics(VIRTUAL_TS_URI.toString()),
        client.getSyntacticDiagnostics(VIRTUAL_TS_URI.toString()),
      ]);

      const allDiags = [...syntacticDiags, ...semanticDiags] as TsDiagnostic[];

      console.log(
        "[TypeCheck] Virtual TS diagnostics:",
        allDiags.length,
        JSON.stringify(allDiags, null, 2),
      );

      return allDiags.map((d) => {
        const startPos = virtualTsModel!.getPositionAt(d.start);
        const endPos = virtualTsModel!.getPositionAt(d.start + d.length);

        let message = "Unknown error";
        if (typeof d.messageText === "string") {
          message = d.messageText;
        } else if (d.messageText && typeof d.messageText === "object") {
          message = d.messageText.messageText || "Unknown error";
        } else if (typeof d.message === "string") {
          message = d.message;
        }

        const severity = d.category === 1 ? "error" : d.category === 0 ? "warning" : "info";

        return {
          message,
          code: d.code,
          startLine: startPos.lineNumber,
          startColumn: startPos.column,
          endLine: endPos.lineNumber,
          endColumn: endPos.column,
          severity: severity as "error" | "warning" | "info",
        };
      });
    } catch (e) {
      if (isTypeScriptRegistrationError(e)) {
        isTypeScriptReady = false;
        return [];
      }
      console.error("Failed to get TypeScript diagnostics:", e);
      return [];
    }
  }

  function parseSourceMap(virtualTs: string): SourceMapEntry[] {
    const entries: SourceMapEntry[] = [];
    const regex = /\/\/ @vize-map:\s*(\d+):(\d+)\s*->\s*(\d+):(\d+)/g;
    let match;
    while ((match = regex.exec(virtualTs)) !== null) {
      entries.push({
        genStart: parseInt(match[1]),
        genEnd: parseInt(match[2]),
        srcStart: parseInt(match[3]),
        srcEnd: parseInt(match[4]),
      });
    }
    return entries;
  }

  function mapDiagnosticsToSource(
    tsDiags: Diagnostic[],
    virtualTs: string,
    vueSource: string,
  ): Diagnostic[] {
    const sourceMapEntries = parseSourceMap(virtualTs);
    const mapped: Diagnostic[] = [];

    function lineColToOffset(content: string, line: number, col: number): number {
      const lines = content.split("\n");
      let offset = 0;
      for (let i = 0; i < line - 1 && i < lines.length; i++) {
        offset += lines[i].length + 1;
      }
      return offset + col - 1;
    }

    function offsetToLineCol(content: string, offset: number): { line: number; col: number } {
      const lines = content.split("\n");
      let currentOffset = 0;
      for (let i = 0; i < lines.length; i++) {
        const lineEnd = currentOffset + lines[i].length + 1;
        if (offset < lineEnd) {
          return { line: i + 1, col: offset - currentOffset + 1 };
        }
        currentOffset = lineEnd;
      }
      return { line: lines.length, col: 1 };
    }

    for (const diag of tsDiags) {
      const diagOffset = lineColToOffset(virtualTs, diag.startLine, diag.startColumn);
      const diagEndOffset = lineColToOffset(
        virtualTs,
        diag.endLine || diag.startLine,
        diag.endColumn || diag.startColumn,
      );

      for (const entry of sourceMapEntries) {
        if (diagOffset >= entry.genStart && diagOffset <= entry.genEnd) {
          const relativeOffset = diagOffset - entry.genStart;
          const srcOffset = entry.srcStart + relativeOffset;
          const srcEndOffset = Math.min(entry.srcEnd, srcOffset + (diagEndOffset - diagOffset));

          const startPos = offsetToLineCol(vueSource, srcOffset);
          const endPos = offsetToLineCol(vueSource, srcEndOffset);
          const help = diag.code ? generateHelp(diag.code, diag.message) : undefined;

          mapped.push({
            ...diag,
            startLine: startPos.line,
            startColumn: startPos.col,
            endLine: endPos.line,
            endColumn: endPos.col,
            message: diag.code ? `[vize:TS${diag.code}] ${diag.message}` : `[vize] ${diag.message}`,
            help,
          });
          break;
        }
      }
    }

    return mapped;
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
    error.value = null;

    try {
      const result = comp.typeCheck(source.value, {
        filename: "example.vue",
        strict: strictMode.value,
        includeVirtualTs: true,
        checkProps: checkProps.value,
        checkEmits: checkEmits.value,
        checkTemplateBindings: checkTemplateBindings.value,
      });
      typeCheckResult.value = result;

      if (useMonacoTs.value && result.virtualTs) {
        cachedSourceMap = parseSourceMap(result.virtualTs);
        const tsDiags = await getTypeScriptDiagnostics(result.virtualTs);
        tsDiagnostics.value = mapDiagnosticsToSource(tsDiags, result.virtualTs, source.value);
      } else {
        tsDiagnostics.value = [];
        cachedSourceMap = [];
      }

      checkTime.value = performance.now() - startTime;
    } catch (e) {
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

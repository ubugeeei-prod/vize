import type {
  CroquisResult,
  CrossFileResult,
  CrossFileInput,
  CrossFileOptions as WasmCrossFileOptions,
} from "../../wasm/index";
import { getWasm } from "../../wasm/index";
import type { CrossFileIssue } from "./types";
import type { Ref, ComputedRef } from "vue";
import {
  resetIssueIdCounter,
  offsetToLineColumn,
  buildSuppressionMap,
  filterSuppressedIssues,
} from "./utils";

export interface CrossFileAnalyzerOptions {
  provideInject: boolean;
  componentEmits: boolean;
  fallthroughAttrs: boolean;
  reactivityTracking: boolean;
  uniqueIds: boolean;
  serverClientBoundary: boolean;
  setupContext: boolean;
  circularDependencies: boolean;
  componentResolution: boolean;
  propsValidation: boolean;
}

interface UseCrossFileAnalysisOptions {
  files: Ref<Record<string, string>>;
  croquisResults: Ref<Record<string, CroquisResult | null>>;
  crossFileIssues: Ref<CrossFileIssue[]>;
  analysisTime: Ref<number>;
  isAnalyzing: Ref<boolean>;
  dependencyGraph: ComputedRef<Record<string, string[]>>;
  options: Ref<CrossFileAnalyzerOptions>;
}

export function useCrossFileAnalysis({
  files,
  croquisResults,
  crossFileIssues,
  analysisTime,
  isAnalyzing,
  options,
}: UseCrossFileAnalysisOptions) {
  let issueIdCounter = 0;

  async function analyzeAll() {
    const compiler = getWasm();
    if (!compiler) return;

    isAnalyzing.value = true;
    resetIssueIdCounter();
    issueIdCounter = 0;

    const results: Record<string, CroquisResult | null> = {};
    for (const [filename, source] of Object.entries(files.value)) {
      try {
        results[filename] = compiler.analyzeSfc(source, { filename });
      } catch {
        results[filename] = null;
      }
    }
    croquisResults.value = results;

    const crossFileInputs: CrossFileInput[] = Object.entries(files.value).map(([path, source]) => ({
      path,
      source,
    }));

    const wasmOptions: WasmCrossFileOptions = {
      provideInject: options.value.provideInject,
      componentEmits: options.value.componentEmits,
      fallthroughAttrs: options.value.fallthroughAttrs,
      reactivityTracking: options.value.reactivityTracking,
      uniqueIds: options.value.uniqueIds,
      serverClientBoundary: options.value.serverClientBoundary,
      setupContext: options.value.setupContext,
      circularDependencies: options.value.circularDependencies,
      maxImportDepth: options.value.circularDependencies ? 10 : undefined,
      componentResolution: options.value.componentResolution,
      propsValidation: options.value.propsValidation,
    };

    const issues: CrossFileIssue[] = [];

    const crossFileResult: CrossFileResult = compiler.analyzeCrossFile(
      crossFileInputs,
      wasmOptions,
    );

    for (const diag of crossFileResult.diagnostics) {
      const source = files.value[diag.file] || "";
      const loc = offsetToLineColumn(source, diag.offset);
      const endLoc = offsetToLineColumn(source, diag.endOffset);

      issues.push({
        id: `issue-${++issueIdCounter}`,
        type: diag.type,
        code: diag.code,
        severity: diag.severity === "hint" ? "info" : diag.severity,
        message: diag.message,
        file: diag.file,
        line: loc.line,
        column: loc.column,
        endLine: endLoc.line,
        endColumn: endLoc.column,
        relatedLocations: diag.relatedLocations?.map((rel) => {
          const relSource = files.value[rel.file] || "";
          const relLoc = offsetToLineColumn(relSource, rel.offset);
          return {
            file: rel.file,
            line: relLoc.line,
            column: relLoc.column,
            message: rel.message,
          };
        }),
        suggestion: diag.suggestion,
      });
    }

    analysisTime.value = crossFileResult.stats.analysisTimeMs;

    const suppressionMap = buildSuppressionMap(files.value);
    const filteredIssues = filterSuppressedIssues(issues, suppressionMap);

    crossFileIssues.value = filteredIssues;
    isAnalyzing.value = false;
  }

  return {
    analyzeAll,
  };
}

import fs from "node:fs";

import type { Context } from "@oxlint/plugins";

import { lintPatina } from "./binding.js";
import type { PatinaDiagnostic, SingleScriptMap } from "./model.js";
import { createSingleScriptMap } from "./script-map.js";
import { getCacheKey, getPatinaSettings } from "./settings.js";

export interface FileState {
  source: string;
  sourceLines: readonly string[];
  extractedScript: string;
  scriptMap: SingleScriptMap | null | undefined;
  allDiagnosticsByRule: Map<string, PatinaDiagnostic[]> | null;
  partialDiagnosticsByRule: Map<string, readonly PatinaDiagnostic[]>;
  requestedRules: Set<string>;
}

const fileStateCache = new Map<string, FileState>();
const EMPTY_DIAGNOSTICS: readonly PatinaDiagnostic[] = [];

export function getFileState(context: Context): FileState {
  const settings = getPatinaSettings(context);
  const cacheKey = getCacheKey(context.physicalFilename, settings);
  const cached = fileStateCache.get(cacheKey);
  if (cached) {
    return cached;
  }

  const source = fs.readFileSync(context.physicalFilename, "utf8");
  const state: FileState = {
    source,
    sourceLines: source.split(/\r\n?|\n/gu),
    extractedScript: context.sourceCode.text,
    scriptMap: undefined,
    allDiagnosticsByRule: null,
    partialDiagnosticsByRule: new Map(),
    requestedRules: new Set(),
  };
  fileStateCache.set(cacheKey, state);
  return state;
}

export function getDiagnosticsForRule(
  context: Context,
  state: FileState,
  ruleName: string,
): readonly PatinaDiagnostic[] {
  if (state.allDiagnosticsByRule) {
    return state.allDiagnosticsByRule.get(ruleName) ?? EMPTY_DIAGNOSTICS;
  }

  const cached = state.partialDiagnosticsByRule.get(ruleName);
  if (cached) {
    return cached;
  }

  const settings = getPatinaSettings(context);

  if (state.requestedRules.size === 0) {
    state.requestedRules.add(ruleName);
    const diagnostics = lintPatina(state.source, context.physicalFilename, settings, [
      ruleName,
    ]).diagnostics;
    const indexedDiagnostics = indexDiagnosticsByRule(diagnostics);
    const ruleDiagnostics = indexedDiagnostics.get(ruleName) ?? EMPTY_DIAGNOSTICS;
    state.partialDiagnosticsByRule.set(ruleName, ruleDiagnostics);
    return ruleDiagnostics;
  }

  state.requestedRules.add(ruleName);
  state.allDiagnosticsByRule = indexDiagnosticsByRule(
    lintPatina(state.source, context.physicalFilename, settings).diagnostics,
  );
  state.partialDiagnosticsByRule.clear();
  return state.allDiagnosticsByRule.get(ruleName) ?? EMPTY_DIAGNOSTICS;
}

export function getScriptMap(state: FileState): SingleScriptMap | null {
  if (state.scriptMap !== undefined) {
    return state.scriptMap;
  }

  state.scriptMap = createSingleScriptMap(state.source, state.extractedScript);
  return state.scriptMap;
}

function indexDiagnosticsByRule(
  diagnostics: readonly PatinaDiagnostic[],
): Map<string, PatinaDiagnostic[]> {
  const grouped = new Map<string, PatinaDiagnostic[]>();

  for (const diagnostic of diagnostics) {
    const existing = grouped.get(diagnostic.rule);
    if (existing) {
      existing.push(diagnostic);
      continue;
    }

    grouped.set(diagnostic.rule, [diagnostic]);
  }

  return grouped;
}

import fs from "node:fs";

import type { Context } from "@oxlint/plugins";

import { lintPatina } from "./binding.js";
import type { PatinaDiagnostic, SfcBlock, SingleScriptMap } from "./model.js";
import { extractSfcBlocks } from "./sfc-blocks.js";
import { createSingleScriptMap } from "./script-map.js";
import { getCacheKey, getVizeSettings, isIncrementalPreset } from "./settings.js";
import { resolveWorkaroundSource } from "./workaround.js";

export interface FileState {
  filename: string;
  source: string;
  extractedScript: string;
  usesOriginalLocations: boolean;
  reportedRules: Set<string>;
  sfcBlocks: readonly SfcBlock[] | undefined;
  scriptMap: SingleScriptMap | null | undefined;
  allDiagnosticsByRule: Map<string, PatinaDiagnostic[]> | null;
  partialDiagnosticsByRule: Map<string, readonly PatinaDiagnostic[]>;
  requestedRules: Set<string>;
}

const fileStateCache = new Map<string, FileState>();
const EMPTY_DIAGNOSTICS: readonly PatinaDiagnostic[] = [];

export function getFileState(context: Context): FileState {
  const settings = getVizeSettings(context);
  const physicalSource = fs.readFileSync(context.physicalFilename, "utf8");
  const resolvedSource = resolveWorkaroundSource(physicalSource, context.physicalFilename);
  const cacheKey = getCacheKey(resolvedSource.filename, settings);
  const cached = fileStateCache.get(cacheKey);
  if (cached) {
    return cached;
  }

  const state: FileState = {
    filename: resolvedSource.filename,
    source: resolvedSource.source,
    extractedScript: context.sourceCode.text,
    usesOriginalLocations: resolvedSource.usesOriginalLocations,
    reportedRules: new Set(),
    sfcBlocks: undefined,
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

  const settings = getVizeSettings(context);
  if (isIncrementalPreset(settings)) {
    const diagnostics = lintPatina(state.source, state.filename, settings, [ruleName]).diagnostics;
    const indexedDiagnostics = indexDiagnosticsByRule(diagnostics);
    const ruleDiagnostics = indexedDiagnostics.get(ruleName) ?? EMPTY_DIAGNOSTICS;
    state.partialDiagnosticsByRule.set(ruleName, ruleDiagnostics);
    return ruleDiagnostics;
  }

  if (state.requestedRules.size === 0) {
    state.requestedRules.add(ruleName);
    const diagnostics = lintPatina(state.source, state.filename, settings, [ruleName]).diagnostics;
    const indexedDiagnostics = indexDiagnosticsByRule(diagnostics);
    const ruleDiagnostics = indexedDiagnostics.get(ruleName) ?? EMPTY_DIAGNOSTICS;
    state.partialDiagnosticsByRule.set(ruleName, ruleDiagnostics);
    return ruleDiagnostics;
  }

  state.requestedRules.add(ruleName);
  const allDiagnostics = lintPatina(state.source, state.filename, settings).diagnostics;
  state.allDiagnosticsByRule = indexDiagnosticsByRule(allDiagnostics);
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

export function getSfcBlocks(state: FileState): readonly SfcBlock[] {
  if (state.sfcBlocks !== undefined) {
    return state.sfcBlocks;
  }

  state.sfcBlocks = extractSfcBlocks(state.source);
  return state.sfcBlocks;
}

export function markRuleAsReported(state: FileState, ruleName: string): boolean {
  if (state.reportedRules.has(ruleName)) {
    return false;
  }

  state.reportedRules.add(ruleName);
  return true;
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

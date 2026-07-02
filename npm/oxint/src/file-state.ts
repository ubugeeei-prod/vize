import fs from "node:fs";

import type { Context } from "@oxlint/plugins";

import { lintPatina } from "./binding.js";
import type { PatinaDiagnostic, SfcBlock, SingleScriptMap } from "./model.js";
import { extractSfcBlocks } from "./sfc-blocks.js";
import { createSingleScriptMap } from "./script-map.js";
import {
  getCacheKey,
  getVizeSettings,
  isIncrementalPreset,
  isTypeAwareRuleName,
} from "./settings.js";
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
  allDiagnosticsIncludesTypeAware: boolean;
  partialDiagnosticsByRule: Map<string, readonly PatinaDiagnostic[]>;
  requestedRules: Set<string>;
  reportedTypeAwareRuntimeDiagnostic: boolean;
}

const fileStateCache = new Map<string, FileState>();
const EMPTY_DIAGNOSTICS: readonly PatinaDiagnostic[] = [];
const TYPE_AWARE_RUNTIME_RULE = "type/corsa-runtime";

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
    allDiagnosticsIncludesTypeAware: false,
    partialDiagnosticsByRule: new Map(),
    requestedRules: new Set(),
    reportedTypeAwareRuntimeDiagnostic: false,
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
    if (isTypeAwareRuleName(ruleName) && !state.allDiagnosticsIncludesTypeAware) {
      const settings = getVizeSettings(context);
      state.allDiagnosticsByRule = indexDiagnosticsByRule(
        lintPatina(state.source, state.filename, { ...settings, typeAware: true }).diagnostics,
      );
      state.allDiagnosticsIncludesTypeAware = true;
    }

    return diagnosticsForRule(state, state.allDiagnosticsByRule, ruleName);
  }

  const cached = state.partialDiagnosticsByRule.get(ruleName);
  if (cached) {
    return cached;
  }

  const settings = getVizeSettings(context);
  const ruleSettings = isTypeAwareRuleName(ruleName) ? { ...settings, typeAware: true } : settings;
  if (isIncrementalPreset(settings)) {
    const diagnostics = lintPatina(state.source, state.filename, ruleSettings, [
      ruleName,
    ]).diagnostics;
    const indexedDiagnostics = indexDiagnosticsByRule(diagnostics);
    const ruleDiagnostics = diagnosticsForRule(state, indexedDiagnostics, ruleName);
    state.partialDiagnosticsByRule.set(ruleName, ruleDiagnostics);
    return ruleDiagnostics;
  }

  if (state.requestedRules.size === 0) {
    state.requestedRules.add(ruleName);
    const diagnostics = lintPatina(state.source, state.filename, ruleSettings, [
      ruleName,
    ]).diagnostics;
    const indexedDiagnostics = indexDiagnosticsByRule(diagnostics);
    const ruleDiagnostics = diagnosticsForRule(state, indexedDiagnostics, ruleName);
    state.partialDiagnosticsByRule.set(ruleName, ruleDiagnostics);
    return ruleDiagnostics;
  }

  state.requestedRules.add(ruleName);
  const fullPassTypeAware =
    settings.typeAware === true || Array.from(state.requestedRules).some(isTypeAwareRuleName);
  const allDiagnostics = lintPatina(state.source, state.filename, {
    ...settings,
    typeAware: fullPassTypeAware,
  }).diagnostics;
  state.allDiagnosticsByRule = indexDiagnosticsByRule(allDiagnostics);
  state.allDiagnosticsIncludesTypeAware = fullPassTypeAware;
  state.partialDiagnosticsByRule.clear();
  return diagnosticsForRule(state, state.allDiagnosticsByRule, ruleName);
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

function diagnosticsForRule(
  state: FileState,
  diagnosticsByRule: Map<string, PatinaDiagnostic[]>,
  ruleName: string,
): readonly PatinaDiagnostic[] {
  const directDiagnostics = diagnosticsByRule.get(ruleName);
  if (directDiagnostics && directDiagnostics.length > 0) {
    return directDiagnostics;
  }

  if (!isTypeAwareRuleName(ruleName) || state.reportedTypeAwareRuntimeDiagnostic) {
    return EMPTY_DIAGNOSTICS;
  }

  const runtimeDiagnostics = diagnosticsByRule.get(TYPE_AWARE_RUNTIME_RULE);
  if (!runtimeDiagnostics || runtimeDiagnostics.length === 0) {
    return EMPTY_DIAGNOSTICS;
  }

  state.reportedTypeAwareRuntimeDiagnostic = true;
  return runtimeDiagnostics;
}

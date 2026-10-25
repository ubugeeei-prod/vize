import type { PatinaDiagnostic } from "./model.js";
import { isTypeAwareRuleName } from "./settings.js";

export const TYPE_AWARE_RUNTIME_RULE = "type/corsa-runtime";
const EMPTY_DIAGNOSTICS: readonly PatinaDiagnostic[] = [];

interface RuntimeDiagnosticState {
  reportedTypeAwareRuntimeDiagnostic: boolean;
}

export function indexDiagnosticsByRule(
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

export function diagnosticsForRule(
  state: RuntimeDiagnosticState,
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

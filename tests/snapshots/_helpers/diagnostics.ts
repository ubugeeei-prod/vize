interface VizeCheckReport {
  files?: Array<{ diagnostics?: unknown }>;
}

const TARGET_PARAMETER_LABEL = /Types of parameters '([^']+)' and '[^']+' are incompatible\./g;

/**
 * Canonicalize TypeScript's target-side function parameter labels in full JSON
 * check baselines. The authored parameter, diagnostic code, anchor, and type
 * text remain exact; only the generated-side display label is non-semantic.
 */
export function normalizeTargetParameterLabels<T extends VizeCheckReport>(report: T): T {
  for (const file of report.files ?? []) {
    if (!Array.isArray(file.diagnostics)) continue;
    file.diagnostics = file.diagnostics.map((diagnostic) =>
      typeof diagnostic === "string" ? normalizeTargetParameterLabel(diagnostic) : diagnostic,
    );
  }
  return report;
}

function normalizeTargetParameterLabel(diagnostic: string): string {
  return diagnostic.replace(
    TARGET_PARAMETER_LABEL,
    "Types of parameters '$1' and '<target>' are incompatible.",
  );
}

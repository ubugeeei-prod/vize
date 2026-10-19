import { loadFormatterCheckEvidence } from "./glyph-corpus.ts";
import type { FormatterChangeEvidence } from "./tool-matrix-formatter.ts";

export interface BaselineUnusableRecord {
  readonly project: string;
  readonly file: string;
  readonly detail: string;
}

export function loadFormatterCheckEvidenceOrRecord(
  project: { readonly id: string },
  baselineUnusable: BaselineUnusableRecord[],
  reportDir = process.env.FIXTURE_REPORT_DIR,
): FormatterChangeEvidence | null {
  try {
    return loadFormatterCheckEvidence(project, reportDir);
  } catch (error) {
    baselineUnusable.push({
      project: project.id,
      file: "formatter-check",
      detail: `formatter --check evidence unavailable: ${errorMessage(error)}`,
    });
    return null;
  }
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

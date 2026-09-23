import { loadFormatterCheckEvidence } from "./glyph-corpus.mjs";

export function loadFormatterCheckEvidenceOrRecord(
  project,
  baselineUnusable,
  reportDir = process.env.FIXTURE_REPORT_DIR,
) {
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

function errorMessage(error) {
  return error instanceof Error ? error.message : String(error);
}

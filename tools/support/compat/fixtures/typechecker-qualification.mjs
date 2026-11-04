/** Separate configuration failure from semantic typechecker qualification. */

const CONFIGURATION_CODE = /TS(?:2688|6053)\b/;

export function typecheckerRunStatus(exitCode, parsed) {
  if (exitCode === 0) return "ok";
  const files = Array.isArray(parsed?.files) ? parsed.files : [];
  let sawDiagnostic = false;
  for (const file of files) {
    const path = typeof file?.file === "string" ? file.file : "";
    const diagnostics = Array.isArray(file?.diagnostics) ? file.diagnostics : [];
    for (const diagnostic of diagnostics) {
      sawDiagnostic = true;
      if (
        typeof diagnostic !== "string" ||
        !path.endsWith("tsconfig.json") ||
        !CONFIGURATION_CODE.test(diagnostic)
      ) {
        return "findings";
      }
    }
  }
  return sawDiagnostic ? "configuration" : "findings";
}

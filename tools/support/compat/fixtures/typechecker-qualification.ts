/** Separate configuration failure from semantic typechecker qualification. */

const CONFIGURATION_CODE = /TS(?:2688|6053)\b/;

export type TypecheckerRunStatus = "ok" | "configuration" | "findings";

interface QualificationPacket {
  readonly files?: unknown;
}

interface QualificationFile {
  readonly file?: unknown;
  readonly diagnostics?: unknown;
}

export function typecheckerRunStatus(
  exitCode: number | null,
  parsed: unknown,
): TypecheckerRunStatus {
  if (exitCode === 0) return "ok";
  const packet = parsed as QualificationPacket | null | undefined;
  const files: readonly unknown[] = Array.isArray(packet?.files) ? packet.files : [];
  let sawDiagnostic = false;
  for (const file of files) {
    const entry = file as QualificationFile | null | undefined;
    const path = typeof entry?.file === "string" ? entry.file : "";
    const diagnostics: readonly unknown[] = Array.isArray(entry?.diagnostics)
      ? entry.diagnostics
      : [];
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

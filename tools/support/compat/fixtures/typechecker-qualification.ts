/** Coverage is transport evidence; unavailable project inputs prevent semantic qualification. */

const diagnosticPattern = /^error:[1-9]\d*:[1-9]\d* \[TS(\d+)\] /u;
const unavailableInputCodes = new Set([
  2307, 2318, 2688, 2792, 5083, 6053, 6059, 6305, 6307, 18002, 18003,
]);

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
  for (const file of files) {
    const entry = file as QualificationFile | null | undefined;
    const path = typeof entry?.file === "string" ? entry.file : "";
    const diagnostics: readonly unknown[] = Array.isArray(entry?.diagnostics)
      ? entry.diagnostics
      : [];
    for (const diagnostic of diagnostics) {
      if (typeof diagnostic !== "string") continue;
      const match = diagnosticPattern.exec(diagnostic);
      if (match == null) continue;
      // Missing modules/generated declarations can introduce `any` into later
      // checks. An authored error elsewhere cannot qualify this partial run.
      if (
        path.endsWith(".json") ||
        path.endsWith(".jsonc") ||
        unavailableInputCodes.has(Number(match[1]))
      ) {
        return "configuration";
      }
    }
  }
  return "findings";
}

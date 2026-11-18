/** Coverage is transport evidence; unavailable project inputs prevent semantic qualification. */

const diagnosticPattern = /^error:[1-9]\d*:[1-9]\d* \[TS(\d+)\] /u;
const unavailableInputCodes = new Set([
  2307, 2318, 2688, 2792, 5083, 6053, 6059, 6305, 6307, 18002, 18003,
]);
// Deprecated and removed compiler options (TS5101, TS5102, TS5107, TS5108)
// introduce no `any`. They stay in the compared set; both checkers must agree.
const optionNoticeCodes = new Set([5101, 5102, 5107, 5108]);

export type TypecheckerRunStatus = "ok" | "configuration" | "findings";

export interface TypecheckerSetupFailure {
  readonly kind: "tsconfig-preparation" | "workspace-alias-overlap" | "unavailable-type-input";
  readonly message: string;
}

interface QualificationPacket {
  readonly files?: unknown;
}

interface QualificationFile {
  readonly file?: unknown;
  readonly diagnostics?: unknown;
}

export function typecheckerSetupFailure(
  exitCode: number | null,
  stdout: string,
  stderr: string,
): TypecheckerSetupFailure | null {
  if (exitCode !== 1 || stdout !== "" || stderr === "") return null;
  const candidates = [
    {
      kind: "tsconfig-preparation" as const,
      pattern: /^Error: Failed to prepare type checker tsconfig: .+$/mu,
    },
    {
      kind: "workspace-alias-overlap" as const,
      pattern: /^Error: IO error: Workspace package alias overlaps another package link$/mu,
    },
    {
      kind: "unavailable-type-input" as const,
      pattern:
        /^Error: corsa error \(exit code 2\): error TS(?:2307|2318|2688|2792|5083|6053|6059|6305|6307|18002|18003): .+$/mu,
    },
  ];
  for (const candidate of candidates) {
    const match = candidate.pattern.exec(stderr);
    if (match != null) return { kind: candidate.kind, message: match[0] };
  }
  return null;
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
      if (optionNoticeCodes.has(Number(match[1]))) continue;
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

import type { VirtualTsMapping } from "../../wasm/types/analysis";
import { offsetToLineColumn } from "../../utils/position";
import { mapGeneratedRange } from "./sourceMappings";
import { generateHelp } from "./generateHelp";

export interface Diagnostic {
  message: string;
  help?: string;
  code?: number;
  startLine: number;
  startColumn: number;
  endLine?: number;
  endColumn?: number;
  severity: "error" | "warning" | "info";
}

export interface TsDiagnostic {
  start: number;
  length: number;
  messageText: string | { messageText: string };
  message?: string;
  category: number;
  code: number;
}

export function mapDiagnosticsToSource(
  tsDiags: TsDiagnostic[],
  mappings: VirtualTsMapping[],
  vueSource: string,
  virtualTs: string,
): Diagnostic[] {
  const mapped: Diagnostic[] = [];
  for (const diag of tsDiags) {
    const range = mapGeneratedRange(diag.start, diag.start + diag.length, mappings);
    if (!range) continue;
    const start = offsetToLineColumn(vueSource, range.start);
    const end = offsetToLineColumn(vueSource, range.end);
    const message =
      typeof diag.messageText === "string"
        ? diag.messageText
        : (diag.messageText?.messageText ?? diag.message ?? "Unknown error");
    mapped.push({
      startLine: start.line,
      startColumn: start.column,
      endLine: end.line,
      endColumn: end.column,
      code: diag.code,
      severity: isUnreachablePattern(diag, virtualTs)
        ? "warning"
        : diag.category === 1
          ? "error"
          : diag.category === 0
            ? "warning"
            : "info",
      message: `[vize:TS${diag.code}] ${message}`,
      help: generateHelp(diag.code, message),
    });
  }

  return mapped;
}

function isUnreachablePattern(diag: TsDiagnostic, virtualTs: string) {
  if (diag.code !== 2322) return false;
  return /^__vize_match_\w*unreachable: __VizePatterns\.Reachable</.test(
    virtualTs.slice(diag.start),
  );
}

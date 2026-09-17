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
    const pattern = patternDiagnostic(diag, virtualTs, message);
    mapped.push({
      startLine: start.line,
      startColumn: start.column,
      endLine: end.line,
      endColumn: end.column,
      code: diag.code,
      severity: pattern?.warning
        ? "warning"
        : diag.category === 1
          ? "error"
          : diag.category === 0
            ? "warning"
            : "info",
      message: `[vize:TS${diag.code}] ${pattern?.message ?? message}`,
      help: pattern?.help ?? generateHelp(diag.code, message),
    });
  }

  return mapped;
}

function patternDiagnostic(diag: TsDiagnostic, virtualTs: string, details: string) {
  if (diag.code !== 2322) return;
  const assertion = virtualTs.slice(diag.start);
  if (/^__vize_match_\w*unreachable: __VizePatterns\.Reachable</.test(assertion)) {
    return {
      warning: true,
      message: "Unreachable v-when: the pattern cannot match any remaining value.",
      help: "Remove this arm or check its pattern and order. Earlier unguarded arms may already cover it.",
    };
  }
  if (/^__vize_match_\w+: __VizePatterns\.Exhaustiveness</.test(assertion)) {
    return {
      warning: false,
      message: "Non-exhaustive v-match: not all possible values are covered.",
      help: `Add branches for the remaining values or an unguarded \`v-when="_"\` fallback. Guards do not prove coverage.\n\n**TypeScript details:**\n\`\`\`text\n${details}\n\`\`\``,
    };
  }
}

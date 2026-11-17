// Shared by the Patina rules and the plugin rules: one SFC diagnostic,
// placed on the program Oxlint is visiting.
import type { Diagnostic } from "@oxlint/plugins";

import { getSfcBlocks, type FileState } from "./file-state.js";
import { formatPatinaMessage } from "./format.js";
import type { HelpLevel, PatinaDiagnostic, SfcBlock, SingleScriptMap } from "./model.js";
import { formatBlockLabel, getDiagnosticBlock } from "./sfc-blocks.js";
import { mapToScriptLoc } from "./script-map.js";
import { isLocationBridgeProgram } from "./workaround.js";
import { isScriptLikeFile } from "./file-kinds.js";

export function createOxlintDiagnostic(
  diagnostic: PatinaDiagnostic,
  state: FileState,
  scriptMap: SingleScriptMap | null,
  helpLevel: HelpLevel,
  program: { range: [number, number] },
): Diagnostic {
  const loc = state.usesOriginalLocations
    ? createOriginalSfcLoc(diagnostic)
    : mapToScriptLoc(diagnostic, scriptMap);
  const block = loc === null ? getDiagnosticBlock(diagnostic, getSfcBlocks(state)) : null;
  const message = formatPatinaMessage(diagnostic, {
    hasMappedLocation: loc !== null,
    blockLabel: formatBlockLabel(block),
    helpLevel,
  });
  // An empty `<script>` has no line for a column to land on. Oxlint rejects
  // that line/column pair with RangeError, so anchor the report on the program
  // node. Template coordinates stay in the message; the script program cannot
  // address a node that lives outside it.
  if (loc == null && state.extractedScript.length === 0) {
    return { node: program, message };
  }

  const fallbackColumn = state.extractedScript.length === 0 ? 0 : 1;
  return {
    loc: loc ?? {
      start: { line: 1, column: fallbackColumn },
      end: { line: 1, column: fallbackColumn },
    },
    message,
  };
}

export function shouldReportForCurrentProgram(
  diagnostic: PatinaDiagnostic,
  state: FileState,
  scriptMap: SingleScriptMap | null,
): boolean {
  if (state.usesOriginalLocations) {
    return isScriptLikeFile(state.filename) || isLocationBridgeProgram(state.extractedScript);
  }
  if (scriptMap == null) {
    return true;
  }

  const block = getDiagnosticBlock(diagnostic, getSfcBlocks(state));
  return !isScriptBlock(block) || block === scriptMap.block;
}

function isScriptBlock(block: SfcBlock | null): boolean {
  return block?.kind === "script" || block?.kind === "script-setup";
}

function createOriginalSfcLoc(diagnostic: PatinaDiagnostic): Diagnostic["loc"] {
  return {
    start: {
      line: diagnostic.location.start.line,
      column: Math.max(0, diagnostic.location.start.column - 1),
    },
    end: {
      line: diagnostic.location.end.line,
      column: Math.max(0, diagnostic.location.end.column - 1),
    },
  };
}

import type { Diagnostic } from "@oxlint/plugins";

import type { LineColumn, PatinaDiagnostic, SingleScriptMap } from "./model.js";
import { compareLineColumn, extractSfcBlocks } from "./sfc-blocks.js";

export function createSingleScriptMap(
  source: string,
  extractedScript: string,
): SingleScriptMap | null {
  if (!extractedScript) {
    return null;
  }

  const blocks = extractSfcBlocks(source).filter(
    (block) => block.kind === "script" || block.kind === "script-setup",
  );
  if (blocks.length !== 1) {
    return null;
  }

  const [block] = blocks;
  return block.content === extractedScript ? { block } : null;
}

export function mapToScriptLoc(
  diagnostic: PatinaDiagnostic,
  scriptMap: SingleScriptMap | null,
): Diagnostic["loc"] | null {
  if (!scriptMap) {
    return null;
  }

  const { block } = scriptMap;
  if (
    compareLineColumn(diagnostic.location.start, block.contentStart) < 0 ||
    compareLineColumn(diagnostic.location.end, block.contentEnd) > 0
  ) {
    return null;
  }

  return {
    start: toScriptPosition(diagnostic.location.start, block.contentStart),
    end: toScriptPosition(diagnostic.location.end, block.contentStart),
  };
}

function toScriptPosition(position: LineColumn, contentStart: LineColumn): LineColumn {
  if (position.line === contentStart.line) {
    return {
      line: 1,
      column: position.column - contentStart.column + 1,
    };
  }

  return {
    line: position.line - contentStart.line + 1,
    column: position.column,
  };
}

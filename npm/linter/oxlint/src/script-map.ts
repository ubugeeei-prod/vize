import type { Diagnostic } from "@oxlint/plugins";

import type { LineColumn, PatinaDiagnostic, SfcBlock, SingleScriptMap } from "./model.js";
import { compareLineColumn, extractSfcBlocks } from "./sfc-blocks.js";

export function createSingleScriptMap(
  source: string,
  extractedScript: string,
  sfcBlocks: readonly SfcBlock[] = extractSfcBlocks(source),
): SingleScriptMap | null {
  if (!extractedScript) {
    return null;
  }

  const blocks = sfcBlocks.filter(
    (block) => block.kind === "script" || block.kind === "script-setup",
  );
  let match: SingleScriptMap | null = null;

  for (const block of blocks) {
    const skipped = countSkippedPrefix(block.content, extractedScript);
    if (skipped === null) {
      continue;
    }

    if (match !== null) {
      return null;
    }

    match = {
      block,
      scriptStart: advancePosition(block.contentStart, block.content.slice(0, skipped)),
    };
  }

  return match;
}

/**
 * Oxlint trims the leading newline (and any indentation on that line) from a
 * `.vue` script block before handing the program to JS plugins, so the
 * extracted text starts partway into `block.content`. Returns how many
 * characters were dropped, or `null` when the extracted text is not this
 * block's body.
 */
function countSkippedPrefix(content: string, extractedScript: string): number | null {
  const skipped = content.length - extractedScript.length;
  if (skipped < 0 || !content.endsWith(extractedScript)) {
    return null;
  }

  return /^\s*$/u.test(content.slice(0, skipped)) ? skipped : null;
}

function advancePosition(from: LineColumn, text: string): LineColumn {
  let { line, column } = from;

  for (const character of text) {
    if (character === "\n") {
      line += 1;
      column = 1;
      continue;
    }
    if (character !== "\r") {
      column += 1;
    }
  }

  return { line, column };
}

export function mapToScriptLoc(
  diagnostic: PatinaDiagnostic,
  scriptMap: SingleScriptMap | null,
): Diagnostic["loc"] | null {
  if (!scriptMap) {
    return null;
  }

  const { block, scriptStart } = scriptMap;
  if (
    compareLineColumn(diagnostic.location.start, scriptStart) < 0 ||
    compareLineColumn(diagnostic.location.end, block.contentEnd) > 0
  ) {
    return null;
  }

  return {
    start: toScriptPosition(diagnostic.location.start, scriptStart),
    end: toScriptPosition(diagnostic.location.end, scriptStart),
  };
}

/**
 * Patina reports 1-based SFC line/column; Oxlint expects 1-based lines and
 * 0-based columns relative to the extracted program.
 */
function toScriptPosition(position: LineColumn, scriptStart: LineColumn): LineColumn {
  if (position.line === scriptStart.line) {
    return {
      line: 1,
      column: position.column - scriptStart.column,
    };
  }

  return {
    line: position.line - scriptStart.line + 1,
    column: position.column - 1,
  };
}

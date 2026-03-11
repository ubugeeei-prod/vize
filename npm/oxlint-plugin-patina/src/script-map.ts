import type { Diagnostic } from "@oxlint/plugins";

import type { LineColumn, PatinaDiagnostic, ScriptBlock, SingleScriptMap } from "./model.js";

export function createSingleScriptMap(
  source: string,
  extractedScript: string,
): SingleScriptMap | null {
  if (!extractedScript) {
    return null;
  }

  const blocks = extractScriptBlocks(source);
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

function createLineStartOffsets(source: string): number[] {
  const lineStarts = [0];

  for (let index = 0; index < source.length; index += 1) {
    if (source.charCodeAt(index) === 10) {
      lineStarts.push(index + 1);
    }
  }

  return lineStarts;
}

function offsetToLineColumn(lineStarts: readonly number[], offset: number): LineColumn {
  let low = 0;
  let high = lineStarts.length - 1;

  while (low <= high) {
    const middle = (low + high) >> 1;
    if (lineStarts[middle] <= offset) {
      low = middle + 1;
      continue;
    }

    high = middle - 1;
  }

  const lineIndex = Math.max(0, low - 1);
  return {
    line: lineIndex + 1,
    column: offset - lineStarts[lineIndex] + 1,
  };
}

function extractScriptBlocks(source: string): ScriptBlock[] {
  const blocks: ScriptBlock[] = [];
  const lineStarts = createLineStartOffsets(source);
  const scriptTag = /<script\b[^>]*>/giu;
  let match: RegExpExecArray | null;

  while ((match = scriptTag.exec(source)) !== null) {
    const openTagEnd = match.index + match[0].length;
    const closeTagStart = source.indexOf("</script>", openTagEnd);
    if (closeTagStart === -1) {
      break;
    }

    blocks.push({
      content: source.slice(openTagEnd, closeTagStart),
      contentStart: offsetToLineColumn(lineStarts, openTagEnd),
      contentEnd: offsetToLineColumn(lineStarts, closeTagStart),
    });
    scriptTag.lastIndex = closeTagStart + "</script>".length;
  }

  return blocks;
}

function compareLineColumn(left: LineColumn, right: LineColumn): number {
  if (left.line !== right.line) {
    return left.line - right.line;
  }

  return left.column - right.column;
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

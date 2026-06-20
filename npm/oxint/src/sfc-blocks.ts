import type { LineColumn, PatinaDiagnostic, SfcBlock, SfcBlockKind } from "./model.js";

const rootBlockPattern = /<([A-Za-z][\w-]*)\b[^>]*>/gu;

export function extractSfcBlocks(source: string): SfcBlock[] {
  const blocks: SfcBlock[] = [];
  const lineStarts = createLineStartOffsets(source);
  let cursor = 0;

  for (;;) {
    rootBlockPattern.lastIndex = cursor;
    const match = rootBlockPattern.exec(source);
    if (match == null) {
      return blocks;
    }

    const tagName = match[1];
    const openTag = match[0];
    const openTagEnd = match.index + openTag.length;
    const closeTag = `</${tagName}>`;
    const closeTagStart = source.indexOf(closeTag, openTagEnd);
    if (closeTagStart === -1) {
      cursor = openTagEnd;
      continue;
    }

    blocks.push({
      kind: resolveBlockKind(tagName, openTag),
      name: tagName,
      content: source.slice(openTagEnd, closeTagStart),
      contentStart: offsetToLineColumn(lineStarts, openTagEnd),
      contentEnd: offsetToLineColumn(lineStarts, closeTagStart),
    });

    cursor = closeTagStart + closeTag.length;
  }
}

export function getDiagnosticBlock(
  diagnostic: PatinaDiagnostic,
  blocks: readonly SfcBlock[],
): SfcBlock | null {
  for (const block of blocks) {
    if (
      compareLineColumn(diagnostic.location.start, block.contentStart) >= 0 &&
      compareLineColumn(diagnostic.location.end, block.contentEnd) <= 0
    ) {
      return block;
    }
  }

  return null;
}

export function formatBlockLabel(block: SfcBlock | null): string {
  if (block == null) {
    return "SFC";
  }

  switch (block.kind) {
    case "template":
      return "<template>";
    case "script":
      return "<script>";
    case "script-setup":
      return "<script setup>";
    case "style":
      return "<style>";
    case "custom":
      return `<${block.name}>`;
  }
}

export function compareLineColumn(left: LineColumn, right: LineColumn): number {
  if (left.line !== right.line) {
    return left.line - right.line;
  }

  return left.column - right.column;
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

function resolveBlockKind(tagName: string, openTag: string): SfcBlockKind {
  if (tagName === "template") {
    return "template";
  }

  if (tagName === "script") {
    return /\bsetup(?:\s|>|=)/u.test(openTag) ? "script-setup" : "script";
  }

  if (tagName === "style") {
    return "style";
  }

  return "custom";
}

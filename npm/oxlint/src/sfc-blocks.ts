import type { LineColumn, PatinaDiagnostic, SfcBlock, SfcBlockKind } from "./model.js";

export function extractSfcBlocks(source: string): SfcBlock[] {
  const blocks: SfcBlock[] = [];
  const lineStarts = createLineStartOffsets(source);
  let cursor = 0;

  for (;;) {
    const match = findNextOpenTag(source, cursor);
    if (match == null) {
      return blocks;
    }

    const { tagName, openTag, openTagEnd } = match;
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

interface OpenTagMatch {
  readonly tagName: string;
  readonly openTag: string;
  readonly openTagEnd: number;
}

function findNextOpenTag(source: string, cursor: number): OpenTagMatch | null {
  let index = cursor;

  for (;;) {
    const openTagStart = source.indexOf("<", index);
    if (openTagStart === -1) {
      return null;
    }

    const tagNameStart = openTagStart + 1;
    if (!isAsciiLetter(source.charCodeAt(tagNameStart))) {
      index = tagNameStart;
      continue;
    }

    let tagNameEnd = tagNameStart + 1;
    while (isTagNameCharacter(source.charCodeAt(tagNameEnd))) {
      tagNameEnd += 1;
    }

    if (!isTagNameBoundary(source.charCodeAt(tagNameEnd))) {
      index = tagNameEnd;
      continue;
    }

    const openTagEnd = findOpenTagEnd(source, tagNameEnd);
    if (openTagEnd == null) {
      return null;
    }

    return {
      tagName: source.slice(tagNameStart, tagNameEnd),
      openTag: source.slice(openTagStart, openTagEnd),
      openTagEnd,
    };
  }
}

function findOpenTagEnd(source: string, from: number): number | null {
  let quote: number | null = null;

  for (let index = from; index < source.length; index += 1) {
    const code = source.charCodeAt(index);
    if (quote != null) {
      if (code === quote) {
        quote = null;
      }
      continue;
    }

    if (code === 34 || code === 39) {
      quote = code;
      continue;
    }

    if (code === 62) {
      return index + 1;
    }
  }

  return null;
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

function isAsciiLetter(code: number): boolean {
  return (code >= 65 && code <= 90) || (code >= 97 && code <= 122);
}

function isTagNameCharacter(code: number): boolean {
  return (
    (code >= 48 && code <= 57) ||
    (code >= 65 && code <= 90) ||
    code === 45 ||
    code === 95 ||
    (code >= 97 && code <= 122)
  );
}

function isTagNameBoundary(code: number): boolean {
  return (
    code === 9 ||
    code === 10 ||
    code === 12 ||
    code === 13 ||
    code === 32 ||
    code === 47 ||
    code === 62
  );
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

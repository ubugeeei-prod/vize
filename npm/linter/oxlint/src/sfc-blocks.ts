import { isUtf8 } from "node:buffer";

import type {
  LineColumn,
  NativeSfcBlock,
  NativeSfcDescriptor,
  PatinaDiagnostic,
  SfcBlock,
  SfcBlockKind,
} from "./model.js";
import { loadBinding } from "./native.ts";

export function extractSfcBlocks(source: string): SfcBlock[] {
  // Resolve the image and validate its export outside the user-input error catch.
  const binding = loadBinding();
  if (typeof binding.parseSfc !== "function")
    throw new Error("Vize native binding does not expose the SFC parser.");

  let descriptor: NativeSfcDescriptor;
  try {
    descriptor = binding.parseSfc(source);
  } catch (error) {
    if (
      error !== null &&
      typeof error === "object" &&
      "code" in error &&
      error.code === "InvalidSfc"
    )
      // No ownership can be assigned to malformed input. The lint pass retains
      // the original parser diagnostic; this classifier must not invent blocks.
      return [];
    throw error;
  }
  return projectSfcBlocks(source, descriptor);
}

interface ParsedBlock {
  block: NativeSfcBlock;
  kind: SfcBlockKind;
  name: string;
}

/** Project checked native geometry; never rediscover tags from the source text. */
export function projectSfcBlocks(source: string, descriptor: NativeSfcDescriptor): SfcBlock[] {
  if (
    descriptor.source !== source ||
    !Array.isArray(descriptor.styles) ||
    !Array.isArray(descriptor.customBlocks)
  )
    throw new Error("Vize SFC descriptor does not describe the complete original source.");
  const parsed: ParsedBlock[] = [];
  const add = (block: NativeSfcBlock | null, kind: SfcBlockKind, name: string) => {
    if (block !== null) parsed.push({ block, kind, name });
  };
  add(descriptor.template, "template", "template");
  if (descriptor.script !== null && descriptor.script.setup !== false)
    throw new Error("Vize SFC script descriptor has invalid setup ownership.");
  if (descriptor.scriptSetup !== null && descriptor.scriptSetup.setup !== true)
    throw new Error("Vize SFC setup descriptor has invalid setup ownership.");
  add(descriptor.script, "script", "script");
  add(descriptor.scriptSetup, "script-setup", "script");
  for (const block of descriptor.styles) add(block, "style", "style");
  for (const block of descriptor.customBlocks) {
    if (typeof block.type !== "string" || block.type.length === 0)
      throw new Error("Vize SFC custom descriptor has no block name.");
    add(block, "custom", block.type);
  }

  const bytes = Buffer.from(source, "utf8");
  if (bytes.toString("utf8") !== source)
    throw new Error("Vize SFC source cannot round-trip through UTF-8.");
  for (const { block } of parsed) {
    const loc = block.loc;
    if (
      !loc ||
      typeof block.content !== "string" ||
      ![loc.start, loc.end, loc.tagStart, loc.tagEnd].every(
        (offset) => Number.isSafeInteger(offset) && offset >= 0 && offset <= bytes.length,
      ) ||
      loc.tagStart > loc.start ||
      loc.start > loc.end ||
      loc.end > loc.tagEnd
    )
      throw new Error("Vize SFC descriptor has invalid byte geometry.");
  }
  parsed.sort((left, right) => left.block.loc.tagStart - right.block.loc.tagStart);
  let previousEnd = 0;
  for (const { block } of parsed) {
    if (block.loc.tagStart < previousEnd)
      throw new Error("Vize SFC descriptor contains overlapping top-level blocks.");
    previousEnd = block.loc.tagEnd;
  }
  const positions = projectPositions(
    bytes,
    parsed.flatMap(({ block }) => [
      block.loc.tagStart,
      block.loc.start,
      block.loc.end,
      block.loc.tagEnd,
    ]),
  );
  return parsed.map(({ block, kind, name }) => {
    const { loc, content } = block;
    const start = positions.get(loc.start)!;
    const end = positions.get(loc.end)!;
    if (
      source.slice(start.index, end.index) !== content ||
      loc.startLine !== start.line ||
      loc.startColumn !== start.byteColumn ||
      loc.endLine !== end.line ||
      loc.endColumn !== end.byteColumn
    )
      throw new Error(
        "Vize SFC descriptor content and geometry disagree with the original source.",
      );
    return {
      kind,
      name,
      content,
      contentStart: { line: start.line, column: start.column },
      contentEnd: { line: end.line, column: end.column },
    };
  });
}

interface ProjectedPosition extends LineColumn {
  index: number;
  byteColumn: number;
}

/** Decode disjoint byte segments once. LF is the parser's line boundary; CR stays exact. */
function projectPositions(
  bytes: Buffer,
  offsets: readonly number[],
): Map<number, ProjectedPosition> {
  const positions = new Map<number, ProjectedPosition>();
  let previous = 0;
  let index = 0;
  let line = 1;
  let column = 1;
  let byteColumn = 1;
  for (const offset of [...new Set(offsets)].sort((left, right) => left - right)) {
    const segment = bytes.subarray(previous, offset);
    if (!isUtf8(segment)) throw new Error("Vize SFC byte endpoint splits a UTF-8 scalar.");
    const text = segment.toString("utf8");
    index += text.length;
    const newline = text.lastIndexOf("\n");
    if (newline === -1) {
      column += text.length;
      byteColumn += segment.length;
    } else {
      for (const character of text) if (character === "\n") line += 1;
      column = text.length - newline;
      byteColumn = segment.length - segment.lastIndexOf(10);
    }
    positions.set(offset, { index, line, column, byteColumn });
    previous = offset;
  }
  return positions;
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

import fs from "node:fs";
import path from "node:path";
import { extractSfcSrcInfo } from "@vizejs/native";
import type { SourceProvenanceSpan } from "@vizejs/source-map";

interface MappedPiece {
  text: string;
  source: string;
  sourceContent: string;
  sourceStart: number;
}

interface MappedDocument {
  pieces: MappedPiece[];
  source: string;
}

export interface ResolvedSfcSrcImports {
  source: string;
  dependencies: string[];
  provenance: SourceProvenanceSpan[];
}

function readSrcImport(
  filePath: string,
  tag: string,
  src: string,
): { path: string; content: string } {
  const resolvedPath = path.isAbsolute(src) ? src : path.resolve(path.dirname(filePath), src);
  try {
    return { path: resolvedPath, content: fs.readFileSync(resolvedPath, "utf-8") };
  } catch {
    throw new Error(
      `[vize] <${tag} src="${src}"> not found (resolved: ${resolvedPath}) in ${filePath}`,
    );
  }
}

function stripSrcAttribute(attrs: string): string {
  return attrs.replace(/\s*\bsrc\s*=\s*(?:"[^"]*"|'[^']*')/i, "");
}

function inlineSingleSrcBlock(
  document: MappedDocument,
  filePath: string,
  tag: "script" | "template",
  src: string | undefined,
  dependencies: string[],
): MappedDocument {
  if (!src) return document;
  const imported = readSrcImport(filePath, tag, src);
  dependencies.push(imported.path);
  const pattern = new RegExp(
    `<${tag}\\b([^>]*)\\bsrc\\s*=\\s*(['"])[^'"]+\\2([^>]*)>[\\s\\S]*?<\\/${tag}>`,
    "i",
  );
  const match = pattern.exec(document.source);
  if (!match || match.index === undefined) return document;

  const attrs = stripSrcAttribute(`${match[1] ?? ""}${match[3] ?? ""}`);
  const opening = `<${tag}${attrs}>`;
  const closing = `</${tag}>`;
  const replacement: MappedPiece[] = [
    mainPiece(opening, filePath, document, match.index),
    externalPiece(imported),
    mainPiece(closing, filePath, document, match.index + match[0].length - closing.length),
  ];
  return replaceRange(document, match.index, match.index + match[0].length, replacement);
}

function inlineStyleSrcBlocks(
  initial: MappedDocument,
  filePath: string,
  dependencies: string[],
): MappedDocument {
  const pattern = /<style\b([^>]*)\bsrc\s*=\s*(['"])([^'"]+)\2([^>]*)>[\s\S]*?<\/style>/i;
  let document = initial;
  while (true) {
    const match = pattern.exec(document.source);
    if (!match || match.index === undefined) return document;
    const imported = readSrcImport(filePath, "style", match[3]!);
    dependencies.push(imported.path);
    const attrs = stripSrcAttribute(`${match[1] ?? ""}${match[4] ?? ""}`);
    const opening = `<style${attrs}>`;
    const closing = "</style>";
    document = replaceRange(document, match.index, match.index + match[0].length, [
      mainPiece(opening, filePath, document, match.index),
      externalPiece(imported),
      mainPiece(closing, filePath, document, match.index + match[0].length - closing.length),
    ]);
  }
}

function mainPiece(
  text: string,
  filePath: string,
  document: MappedDocument,
  generatedOffset: number,
): MappedPiece {
  const containing = pieceAt(document.pieces, generatedOffset);
  return {
    text,
    source: filePath,
    sourceContent: containing?.sourceContent ?? document.source,
    sourceStart: containing?.sourceStart ?? generatedOffset,
  };
}

function externalPiece(imported: { path: string; content: string }): MappedPiece {
  return {
    text: imported.content,
    source: imported.path,
    sourceContent: imported.content,
    sourceStart: 0,
  };
}

function pieceAt(pieces: readonly MappedPiece[], offset: number): MappedPiece | null {
  let cursor = 0;
  for (const piece of pieces) {
    if (offset >= cursor && offset < cursor + piece.text.length) {
      return { ...piece, sourceStart: piece.sourceStart + (offset - cursor) };
    }
    cursor += piece.text.length;
  }
  return null;
}

function replaceRange(
  document: MappedDocument,
  start: number,
  end: number,
  replacement: MappedPiece[],
): MappedDocument {
  const pieces = [
    ...slicePieces(document.pieces, 0, start),
    ...replacement,
    ...slicePieces(document.pieces, end, document.source.length),
  ];
  return { pieces, source: pieces.map((piece) => piece.text).join("") };
}

function slicePieces(pieces: readonly MappedPiece[], start: number, end: number): MappedPiece[] {
  const output: MappedPiece[] = [];
  let cursor = 0;
  for (const piece of pieces) {
    const pieceEnd = cursor + piece.text.length;
    const overlapStart = Math.max(start, cursor);
    const overlapEnd = Math.min(end, pieceEnd);
    if (overlapStart < overlapEnd) {
      const localStart = overlapStart - cursor;
      const localEnd = overlapEnd - cursor;
      output.push({
        ...piece,
        text: piece.text.slice(localStart, localEnd),
        sourceStart: piece.sourceStart + localStart,
      });
    }
    cursor = pieceEnd;
  }
  return output;
}

function spansFromPieces(pieces: readonly MappedPiece[]): SourceProvenanceSpan[] {
  const spans: SourceProvenanceSpan[] = [];
  let generatedStart = 0;
  for (const piece of pieces) {
    const generatedEnd = generatedStart + piece.text.length;
    if (generatedEnd > generatedStart) {
      spans.push({
        generatedStart,
        generatedEnd,
        source: piece.source,
        sourceContent: piece.sourceContent,
        sourceStart: piece.sourceStart,
      });
    }
    generatedStart = generatedEnd;
  }
  return spans;
}

export function resolveSfcSrcImports(filePath: string, source: string): ResolvedSfcSrcImports {
  const dependencies: string[] = [];
  const srcInfo = extractSfcSrcInfo(source, filePath);
  let document: MappedDocument = {
    source,
    pieces: [{ text: source, source: filePath, sourceContent: source, sourceStart: 0 }],
  };
  document = inlineSingleSrcBlock(document, filePath, "script", srcInfo.scriptSrc, dependencies);
  document = inlineSingleSrcBlock(
    document,
    filePath,
    "template",
    srcInfo.templateSrc,
    dependencies,
  );
  document = inlineStyleSrcBlocks(document, filePath, dependencies);
  return {
    source: document.source,
    dependencies,
    provenance: spansFromPieces(document.pieces),
  };
}

import fs from "node:fs";
import path from "node:path";
import { parseSfc } from "@vizejs/native";

export interface InlinedSfcSource {
  source: string;
  /** Absolute paths read for `src` blocks, for watch mode. */
  dependencies: string[];
}

// Necessary for a `src` block; other sources skip the parse (`<img src>` too).
const SRC_BLOCK_HINT = /<(?:template|script|style)\b[^>]*\ssrc\s*=/i;
const ATTRIBUTE_RE = /\s+([^\s"'<>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/gy;

type BlockLoc = { start: number; end: number; tagStart: number; tagEnd: number };
type Block = { loc: BlockLoc } | null | undefined;

/** Split an opening tag into its `src` value and the tag without it. */
function splitSrcAttribute(tag: string, openTag: string): { src: string; openTag: string } | null {
  const head = `<${tag}`;
  if (!openTag.toLowerCase().startsWith(head)) return null;
  const body = openTag.slice(head.length).replace(/\s*\/?\s*>$/, "");
  let rest = "";
  let src: string | null = null;
  let consumed = 0;
  ATTRIBUTE_RE.lastIndex = 0;
  for (let match = ATTRIBUTE_RE.exec(body); match; match = ATTRIBUTE_RE.exec(body)) {
    consumed = ATTRIBUTE_RE.lastIndex;
    if (match[1].toLowerCase() === "src" && src === null) {
      src = match[2] ?? match[3] ?? match[4] ?? "";
    } else {
      rest += match[0];
    }
  }
  // Leave tags this tokenizer cannot fully read to the compiler.
  if (src === null || consumed !== body.length) return null;
  return { src, openTag: `${head}${rest}>` };
}

function readSrcBlock(
  filePath: string,
  tag: string,
  src: string,
): { path: string; content: string } {
  const resolved = path.resolve(path.dirname(filePath), src);
  try {
    return { path: resolved, content: fs.readFileSync(resolved, "utf8") };
  } catch {
    throw new Error(
      `[vize] <${tag} src="${src}"> not found (resolved: ${resolved}) in ${filePath}`,
    );
  }
}

/**
 * Inline `<template src>`, `<script src>` and `<style src>` files into the SFC
 * source, as `@vizejs/vite-plugin` does. Blocks are located by the native
 * parser, so `src` text inside scripts or comments is left alone.
 */
export function inlineSfcSrcBlocks(filePath: string, source: string): InlinedSfcSource {
  if (!SRC_BLOCK_HINT.test(source)) return { source, dependencies: [] };

  let parsed: ReturnType<typeof parseSfc>;
  try {
    parsed = parseSfc(source, { filename: filePath });
  } catch {
    // The compiler reports malformed sources with its own diagnostics.
    return { source, dependencies: [] };
  }
  const blocks: Array<[string, Block]> = [
    ["template", parsed.template],
    ["script", parsed.script],
    ...parsed.styles.map((style): [string, Block] => ["style", style]),
  ];
  const bytes = Buffer.from(source);
  const edits: Array<{ start: number; end: number; text: string }> = [];
  const dependencies: string[] = [];

  for (const [tag, block] of blocks) {
    if (!block) continue;
    const { start, tagStart, tagEnd } = block.loc;
    const split = splitSrcAttribute(tag, bytes.subarray(tagStart, start).toString("utf8"));
    if (!split) continue;
    const file = readSrcBlock(filePath, tag, split.src);
    dependencies.push(file.path);
    edits.push({
      start: tagStart,
      end: tagEnd,
      text: `${split.openTag}\n${file.content}\n</${tag}>`,
    });
  }
  if (edits.length === 0) return { source, dependencies };

  edits.sort((a, b) => a.start - b.start);
  const parts: Buffer[] = [];
  let offset = 0;
  for (const edit of edits) {
    parts.push(bytes.subarray(offset, edit.start), Buffer.from(edit.text));
    offset = edit.end;
  }
  parts.push(bytes.subarray(offset));
  return { source: Buffer.concat(parts).toString("utf8"), dependencies };
}

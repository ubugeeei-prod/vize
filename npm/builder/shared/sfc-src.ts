import fs from "node:fs";
import path from "node:path";

/** The bundler's answers for one SFC's `src` module requests: request -> file. */
export type SfcSrcResolutions = ReadonlyMap<string, string>;

export interface InlinedSfcSource {
  source: string;
  /** Absolute paths read for `src` blocks, for watch mode. */
  dependencies: string[];
  /** The file each inlined `<style src>` block came from, by style block index. */
  styleSrcs: Map<number, string>;
}

/**
 * Whether a block `src` is a path relative to the SFC. Any other `src`
 * (package, alias, absolute) is a module request that the bundler resolves,
 * as in Vue's SFC spec and `@vitejs/plugin-vue`.
 */
export function isRelativeSfcSrc(src: string): boolean {
  return src.startsWith("./") || src.startsWith("../");
}

/** The file a block `src` names: next to the SFC, or where the bundler resolved it. */
export function sfcSrcFile(
  importer: string,
  tag: string,
  src: string,
  resolutions: SfcSrcResolutions | undefined,
): string {
  if (isRelativeSfcSrc(src)) return path.resolve(path.dirname(importer), src);
  const file = resolutions?.get(src);
  if (file !== undefined) return file;
  throw new Error(
    `[vize] <${tag} src="${src}"> not found: the bundler resolved no file for this module request from ${importer}`,
  );
}

/** Resolve an SFC's `src` module requests with the bundler's own resolver. */
export async function resolveSfcSrcRequests(
  importer: string,
  requests: readonly string[],
  resolve: (request: string, importer: string) => Promise<string | null | undefined>,
): Promise<Map<string, string>> {
  const resolutions = new Map<string, string>();
  for (const request of new Set(requests)) {
    if (isRelativeSfcSrc(request)) continue;
    // Virtual ids and queries cannot be read from disk; they stay unresolved.
    const file = (await resolve(request, importer))?.replace(/[?#].*$/, "");
    if (file && path.isAbsolute(file)) resolutions.set(request, file);
  }
  return resolutions;
}

// Necessary for a `src` block; other sources skip the parse (`<img src>` too).
const SRC_BLOCK_HINT = /<(?:template|script|style)\b[^>]*\ssrc\s*=/i;
const ATTRIBUTE_RE = /\s+([^\s"'<>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/gy;

type BlockLoc = { start: number; end: number; tagStart: number; tagEnd: number };
type Block = { loc: BlockLoc } | null | undefined;

/** `parseSfc` from `@vizejs/native`, which the plugins (not this directory) depend on. */
export type ParseSfc = (
  source: string,
  options: { filename: string },
) => { template: Block; script: Block; scriptSetup: Block; styles: NonNullable<Block>[] };

type SrcBlock = {
  tag: string;
  src: string;
  openTag: string;
  tagStart: number;
  tagEnd: number;
  /** The block's index among the SFC's `<style>` blocks. */
  styleIndex?: number;
};

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

/** The `src` blocks the native parser finds, so `src` text in scripts or comments is not one. */
function collectSrcBlocks(
  parseSfc: ParseSfc,
  filePath: string,
  bytes: Buffer,
  source: string,
): SrcBlock[] {
  let parsed: ReturnType<ParseSfc>;
  try {
    parsed = parseSfc(source, { filename: filePath });
  } catch {
    // The compiler reports malformed sources with its own diagnostics.
    return [];
  }
  const blocks: Array<[string, Block, number?]> = [
    ["template", parsed.template],
    ["script", parsed.script],
    ["script", parsed.scriptSetup],
    ...parsed.styles.map((style, index): [string, Block, number] => ["style", style, index]),
  ];
  const found: SrcBlock[] = [];
  for (const [tag, block, styleIndex] of blocks) {
    if (!block) continue;
    const { start, tagStart, tagEnd } = block.loc;
    const split = splitSrcAttribute(tag, bytes.subarray(tagStart, start).toString("utf8"));
    if (split) found.push({ tag, ...split, tagStart, tagEnd, styleIndex });
  }
  return found;
}

/** The `src` module requests (not relative paths) of an SFC's blocks, in source order. */
export function listSfcSrcRequests(parseSfc: ParseSfc, filePath: string, source: string): string[] {
  if (!SRC_BLOCK_HINT.test(source)) return [];
  return collectSrcBlocks(parseSfc, filePath, Buffer.from(source), source)
    .map((block) => block.src)
    .filter((src) => !isRelativeSfcSrc(src));
}

function readSrcBlock(
  filePath: string,
  tag: string,
  src: string,
  resolutions: SfcSrcResolutions | undefined,
): { path: string; content: string } {
  const resolved = sfcSrcFile(filePath, tag, src, resolutions);
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
 * source. Blocks are located by the native parser. Module requests come from
 * the bundler's `resolutions` (see {@link sfcSrcFile}).
 */
export function inlineSfcSrcBlocks(
  parseSfc: ParseSfc,
  filePath: string,
  source: string,
  resolutions?: SfcSrcResolutions,
): InlinedSfcSource {
  const styleSrcs = new Map<number, string>();
  if (!SRC_BLOCK_HINT.test(source)) return { source, dependencies: [], styleSrcs };
  const bytes = Buffer.from(source);
  const edits: Array<{ start: number; end: number; text: string }> = [];
  const dependencies: string[] = [];

  for (const block of collectSrcBlocks(parseSfc, filePath, bytes, source)) {
    const file = readSrcBlock(filePath, block.tag, block.src, resolutions);
    dependencies.push(file.path);
    if (block.styleIndex !== undefined) styleSrcs.set(block.styleIndex, file.path);
    edits.push({
      start: block.tagStart,
      end: block.tagEnd,
      text: `${block.openTag}\n${file.content}\n</${block.tag}>`,
    });
  }
  if (edits.length === 0) return { source, dependencies, styleSrcs };

  edits.sort((a, b) => a.start - b.start);
  const parts: Buffer[] = [];
  let offset = 0;
  for (const edit of edits) {
    parts.push(bytes.subarray(offset, edit.start), Buffer.from(edit.text));
    offset = edit.end;
  }
  parts.push(bytes.subarray(offset));
  return { source: Buffer.concat(parts).toString("utf8"), dependencies, styleSrcs };
}

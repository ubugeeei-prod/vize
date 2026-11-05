import fs from "node:fs";
import path from "node:path";

/** The bundler's answer for a `src` module request: a file to inline, or an external module. */
export type SfcSrcResolution = { file: string } | { external: true };

/** The bundler's answers for one SFC's `src` module requests. */
export type SfcSrcResolutions = ReadonlyMap<string, SfcSrcResolution>;

/** What a host resolver returned for a request; `external` is truthy when it is not bundled. */
export type HostResolution = { id: string; external?: boolean | string } | null | undefined;

/** The bundler's own resolver (aliases, packages, every `resolveId` plugin). */
export type HostResolve = (request: string, importer: string) => Promise<HostResolution>;

export interface InlinedSfcSource {
  source: string;
  /** Absolute paths read for `src` blocks, for watch mode. */
  dependencies: string[];
  /** The file each inlined `<style src>` block came from, by style block index. */
  styleSrcs: Map<number, string>;
  /** External `<style src>` requests left in place, for the output to import. */
  imports: string[];
}

/**
 * Whether a block `src` is a path relative to the SFC. Any other `src`
 * (package, alias, absolute) is a module request that the bundler resolves,
 * as in Vue's SFC spec and `@vitejs/plugin-vue`.
 */
export function isRelativeSfcSrc(src: string): boolean {
  return src.startsWith("./") || src.startsWith("../");
}

/** Where a block `src` points: next to the SFC, or where the bundler resolved it. */
function sfcSrcTarget(
  importer: string,
  tag: string,
  src: string,
  resolutions: SfcSrcResolutions | undefined,
): SfcSrcResolution {
  if (isRelativeSfcSrc(src)) return { file: path.resolve(path.dirname(importer), src) };
  const resolution = resolutions?.get(src);
  if (resolution !== undefined) return resolution;
  throw new Error(
    `[vize] <${tag} src="${src}"> not found: the bundler resolved no file for this module request from ${importer}`,
  );
}

/** Resolve an SFC's `src` module requests with the bundler's own resolver. */
export async function resolveSfcSrcRequests(
  importer: string,
  requests: readonly string[],
  resolve: HostResolve,
): Promise<Map<string, SfcSrcResolution>> {
  const resolutions = new Map<string, SfcSrcResolution>();
  for (const request of new Set(requests)) {
    if (isRelativeSfcSrc(request)) continue;
    const resolved = await resolve(request, importer);
    if (!resolved) continue;
    if (resolved.external) {
      resolutions.set(request, { external: true });
      continue;
    }
    // Virtual ids and queries cannot be read from disk; they stay unresolved.
    const file = resolved.id.replace(/[?#].*$/, "");
    if (path.isAbsolute(file)) resolutions.set(request, { file });
  }
  return resolutions;
}

// Necessary for a `src` block; other sources skip the parse (`<img src>` too).
const SRC_BLOCK_HINT = /<(?:template|script|style)\b[^>]*\ssrc\s*=/i;
const ATTRIBUTE_RE = /\s+([^\s"'<>/=]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/gy;

type BlockLoc = { start: number; end: number; tagStart: number; tagEnd: number };
type Block = { loc: BlockLoc; scoped?: boolean; module?: string | null } | null | undefined;

/** `parseSfc` from `@vizejs/native`, which the plugins (not this directory) depend on. */
export type ParseSfc = (
  source: string,
  options: { filename: string },
) => { template: Block; script: Block; scriptSetup: Block; styles: NonNullable<Block>[] };

type BlockKind = "template" | "script" | "script setup" | "style";
type SrcBlock = {
  kind: BlockKind;
  tag: string;
  src: string;
  openTag: string;
  tagStart: number;
  tagEnd: number;
  /** The block's index among the SFC's `<style>` blocks. */
  styleIndex?: number;
  /** Why an external module cannot stand in for the block's contents, if it cannot. */
  needsContents: string | null;
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

function needsContents(kind: BlockKind, block: NonNullable<Block>): string | null {
  const compiled = "is compiled into the component, so vize needs its contents";
  if (kind === "template") return `the template ${compiled}`;
  if (kind === "script setup") return `<script setup> ${compiled}`;
  if (block.scoped) return "a scoped style needs its contents";
  if (block.module) return "a CSS module needs its contents";
  return null;
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
  const blocks: Array<[BlockKind, Block, number?]> = [
    ["template", parsed.template],
    ["script", parsed.script],
    ["script setup", parsed.scriptSetup],
    ...parsed.styles.map((style, index): [BlockKind, Block, number] => ["style", style, index]),
  ];
  const found: SrcBlock[] = [];
  for (const [kind, block, styleIndex] of blocks) {
    if (!block) continue;
    const tag = kind === "script setup" ? "script" : kind;
    const { start, tagStart, tagEnd } = block.loc;
    const split = splitSrcAttribute(tag, bytes.subarray(tagStart, start).toString("utf8"));
    if (split) {
      found.push({
        kind,
        tag,
        ...split,
        tagStart,
        tagEnd,
        styleIndex,
        needsContents: needsContents(kind, block),
      });
    }
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

function readSrcFile(filePath: string, tag: string, src: string, file: string): string {
  try {
    return fs.readFileSync(file, "utf8");
  } catch {
    throw new Error(`[vize] <${tag} src="${src}"> not found (resolved: ${file}) in ${filePath}`);
  }
}

/**
 * The block for an external module, as `@vitejs/plugin-vue` keeps it: a style
 * stays in place for the output to import (`null`), a script imports it.
 */
function externalSrcBlock(filePath: string, block: SrcBlock): string | null {
  if (block.needsContents) {
    const tag = `${block.openTag.slice(0, -1)} src="${block.src}">`;
    throw new Error(
      `[vize] ${tag} resolved to an external module; ${block.needsContents}. Let the bundler bundle "${block.src}" (in ${filePath})`,
    );
  }
  if (block.kind === "style") return null;
  const request = JSON.stringify(block.src);
  return `${block.openTag}\nimport __vize_src_main__ from ${request};\nexport * from ${request};\nexport default __vize_src_main__;\n</script>`;
}

/**
 * Inline `<template src>`, `<script src>` and `<style src>` files into the SFC
 * source. Blocks are located by the native parser. Module requests come from
 * the bundler's `resolutions`; an external one stays an import.
 */
export function inlineSfcSrcBlocks(
  parseSfc: ParseSfc,
  filePath: string,
  source: string,
  resolutions?: SfcSrcResolutions,
): InlinedSfcSource {
  const styleSrcs = new Map<number, string>();
  if (!SRC_BLOCK_HINT.test(source)) return { source, dependencies: [], styleSrcs, imports: [] };
  const bytes = Buffer.from(source);
  const edits: Array<{ start: number; end: number; text: string }> = [];
  const dependencies: string[] = [];
  const imports: string[] = [];

  for (const block of collectSrcBlocks(parseSfc, filePath, bytes, source)) {
    const target = sfcSrcTarget(filePath, block.tag, block.src, resolutions);
    let text: string | null;
    if ("external" in target) {
      text = externalSrcBlock(filePath, block);
      if (text === null) imports.push(block.src);
    } else {
      const content = readSrcFile(filePath, block.tag, block.src, target.file);
      dependencies.push(target.file);
      if (block.styleIndex !== undefined) styleSrcs.set(block.styleIndex, target.file);
      text = `${block.openTag}\n${content}\n</${block.tag}>`;
    }
    if (text !== null) edits.push({ start: block.tagStart, end: block.tagEnd, text });
  }
  if (edits.length === 0) return { source, dependencies, styleSrcs, imports };

  edits.sort((a, b) => a.start - b.start);
  const parts: Buffer[] = [];
  let offset = 0;
  for (const edit of edits) {
    parts.push(bytes.subarray(offset, edit.start), Buffer.from(edit.text));
    offset = edit.end;
  }
  parts.push(bytes.subarray(offset));
  return { source: Buffer.concat(parts).toString("utf8"), dependencies, styleSrcs, imports };
}

/**
 * A `<style src>` the compiler still sees was not inlined. Only an external one
 * may stay, as an import; any other was written in a form the inliner cannot read.
 */
export function assertSfcStyleSrcImports(
  filePath: string,
  styles: ReadonlyArray<{ src?: string | null }>,
  imports: readonly string[] = [],
): void {
  for (const style of styles) {
    if (style.src == null || imports.includes(style.src)) continue;
    throw new Error(
      `[vize] ${filePath}: <style src=${JSON.stringify(style.src)}> could not be inlined. Write it as <style src="./file.css"> with a quoted path.`,
    );
  }
}

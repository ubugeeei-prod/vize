import { sources } from "@rspack/core";
import { diffChars } from "diff";

export interface SourceMapV3 {
  version: number;
  file?: string;
  sources: string[];
  sourcesContent?: (string | null)[];
  names: string[];
  mappings: string;
}

/** Parse native JSON at the loader boundary; malformed maps are omitted. */
export function parseSourceMap(json: string | undefined | null): SourceMapV3 | null {
  if (!json) return null;
  try {
    const map = JSON.parse(json) as Partial<SourceMapV3> | null;
    return map !== null &&
      typeof map === "object" &&
      map.version === 3 &&
      Array.isArray(map.sources) &&
      map.sources.every((source) => typeof source === "string") &&
      Array.isArray(map.names) &&
      map.names.every((name) => typeof name === "string") &&
      (map.sourcesContent === undefined ||
        (Array.isArray(map.sourcesContent) &&
          map.sourcesContent.every((source) => source === null || typeof source === "string"))) &&
      typeof map.mappings === "string"
      ? (map as SourceMapV3)
      : null;
  } catch {
    return null;
  }
}

/** Keep native mappings aligned through module assembly, including column edits. */
export class MappedModule {
  code: string;
  map: SourceMapV3 | null;

  constructor(code: string, map: SourceMapV3 | null) {
    this.code = code;
    this.map = map;
  }

  edit(next: string): void {
    const previous = this.code;
    this.code = next;
    if (this.map === null || next === previous) return;

    const original = new sources.SourceMapSource(
      previous,
      this.map.file ?? "sfc.js",
      JSON.stringify(this.map),
    );
    const edited = new sources.ConcatSource();
    let offset = 0;

    // Native asset rewriting can change several disjoint spans. Keep the
    // unchanged spans between them instead of treating the whole region as one
    // replacement. Offsets use UTF-16 code units, as Rspack's sources API does.
    for (const change of diffChars(previous, next)) {
      if (change.added) edited.add(change.value);
      else if (change.removed) offset += change.value.length;
      else {
        edited.add(
          sliceMappedSource(original, previous.length, offset, offset + change.value.length),
        );
        offset += change.value.length;
      }
    }
    this.map = edited.map({ columns: true });
  }

  /** Apply a known edit without inferring which occurrence of repeated text changed. */
  replace(start: number, end: number, content: string | sources.Source): void {
    const previous = this.code;
    const text = typeof content === "string" ? content : content.source().toString();
    this.code = previous.slice(0, start) + text + previous.slice(end);
    if (this.map === null) return;
    const original = new sources.SourceMapSource(
      previous,
      this.map.file ?? "sfc.js",
      JSON.stringify(this.map),
    );
    this.map = new sources.ConcatSource(
      sliceMappedSource(original, previous.length, 0, start),
      content,
      sliceMappedSource(original, previous.length, end, previous.length),
    ).map({ columns: true });
  }

  prepend(content: string): void {
    this.replace(0, 0, content);
  }

  append(content: string): void {
    this.replace(this.code.length, this.code.length, content);
  }
}

/** Inline external SFC content while retaining the external file as its source. */
export function inlineSrcBlock(
  module: MappedModule,
  block: "script" | "template",
  content: string,
  filename?: string,
): void {
  const pattern = new RegExp(
    `(<${block})([^>]*)\\bsrc=["'][^"']+["']([^>]*>)[\\s\\S]*?(<\\/${block}>)`,
    "i",
  );
  const match = pattern.exec(module.code);
  if (!match) return;
  const [, open, beforeSrc, afterSrc, close] = match;
  const attrs = (beforeSrc + afterSrc).replace(/\bsrc=["'][^"']+["']\s*/g, "");
  module.replace(
    match.index,
    match.index + match[0].length,
    new sources.ConcatSource(
      `${open}${attrs}\n`,
      filename ? new sources.OriginalSource(content, filename) : content,
      `\n${close}`,
    ),
  );
}

function sliceMappedSource(source: sources.Source, length: number, start: number, end: number) {
  if (start === end) return new sources.RawSource("");
  const slice = new sources.ReplaceSource(source);
  if (start > 0) slice.replace(0, start - 1, "");
  if (end < length) slice.replace(end, length - 1, "");
  return slice;
}

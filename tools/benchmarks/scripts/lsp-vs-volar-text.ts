/**
 * Open-document model shared by both editors of `lsp-vs-volar.ts`: UTF-16
 * positions (LSP's and JavaScript's unit), anchors and incremental edits.
 */

export interface Position {
  line: number;
  character: number;
}

export interface Range {
  start: Position;
  end: Position;
}

export interface TextEdit {
  range: Range;
  text: string;
}

/** A spot found by unique text, so specs survive line shifts in the pinned source. */
export interface Anchor {
  find: string;
  /** UTF-16 offset inside `find`; defaults to its start. */
  at?: number;
}

export class OpenDocument {
  version = 1;
  readonly path: string;
  readonly uri: string;
  text: string;

  constructor(path: string, uri: string, text: string) {
    this.path = path;
    this.uri = uri;
    this.text = text;
  }

  offsetAt(position: Position): number {
    let offset = 0;
    for (let line = 0; line < position.line; line += 1) {
      const next = this.text.indexOf("\n", offset);
      if (next < 0) throw new Error(`line ${position.line} is past the end of ${this.path}`);
      offset = next + 1;
    }
    return offset + position.character;
  }

  positionAt(offset: number): Position {
    let line = 0;
    let lineStart = 0;
    for (let index = this.text.indexOf("\n"); index >= 0 && index < offset;) {
      line += 1;
      lineStart = index + 1;
      index = this.text.indexOf("\n", lineStart);
    }
    return { line, character: offset - lineStart };
  }

  /** Throws unless `anchor.find` occurs exactly once: a guessed position measures nothing. */
  resolve(anchor: Anchor): Position {
    const first = this.text.indexOf(anchor.find);
    if (first < 0)
      throw new Error(`${this.path}: anchor not found: ${JSON.stringify(anchor.find)}`);
    if (this.text.indexOf(anchor.find, first + 1) >= 0) {
      throw new Error(`${this.path}: anchor is not unique: ${JSON.stringify(anchor.find)}`);
    }
    const at = anchor.at ?? 0;
    if (at < 0 || at > anchor.find.length)
      throw new Error(`anchor offset ${at} is outside its text`);
    return this.positionAt(first + at);
  }

  /** Replaces the unique `find` with `replacement`; returns the edit to send. */
  replace(find: string, replacement: string): TextEdit {
    const start = this.resolve({ find });
    const end = this.positionAt(this.offsetAt(start) + find.length);
    return this.apply({ range: { start, end }, text: replacement });
  }

  apply(edit: TextEdit): TextEdit {
    const start = this.offsetAt(edit.range.start);
    const end = this.offsetAt(edit.range.end);
    this.text = `${this.text.slice(0, start)}${edit.text}${this.text.slice(end)}`;
    this.version += 1;
    return edit;
  }
}

/** Text with `insert` placed right after the unique `after`. */
export function insertAfter(source: string, after: string, insert: string): string {
  const first = source.indexOf(after);
  if (first < 0) throw new Error(`insertion anchor not found: ${JSON.stringify(after)}`);
  if (source.indexOf(after, first + 1) >= 0) {
    throw new Error(`insertion anchor is not unique: ${JSON.stringify(after)}`);
  }
  const at = first + after.length;
  return `${source.slice(0, at)}${insert}${source.slice(at)}`;
}

/** The anchor at `at`, a unique part of the unique `text`. */
export function anchorIn(text: string, at: string): Anchor {
  return { find: text, at: text.indexOf(at) };
}

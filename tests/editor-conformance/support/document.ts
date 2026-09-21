// LSP text-document arithmetic shared by the judge and the replay driver.
// Positions are UTF-16 code units (the only encoding every TS-45 client offers
// and the one `vize lsp` negotiates), so JavaScript string indices are exact.

export type Position = { line: number; character: number };
export type Range = { start: Position; end: Position };
export type TextEdit = { range: Range; newText: string };
export type ContentChange = { text: string; range?: Range };

export function offsetAt(text: string, position: Position): number {
  let offset = 0;
  for (let line = 0; line < position.line; line += 1) {
    const newline = text.indexOf("\n", offset);
    if (newline < 0) throw new Error(`line ${position.line} is past the end of the document`);
    offset = newline + 1;
  }
  const lineEnd = text.indexOf("\n", offset);
  const lineLength = (lineEnd < 0 ? text.length : lineEnd) - offset;
  if (position.character > lineLength) {
    throw new Error(`character ${position.character} is past the end of line ${position.line}`);
  }
  return offset + position.character;
}

export function applyContentChange(text: string, change: ContentChange): string {
  if (change.range == null) return change.text;
  const start = offsetAt(text, change.range.start);
  const end = offsetAt(text, change.range.end);
  if (end < start) throw new Error("content change range ends before it starts");
  return text.slice(0, start) + change.text + text.slice(end);
}

/** Applies non-overlapping edits the way LSP defines them: all against the original text. */
export function applyTextEdits(text: string, edits: readonly TextEdit[]): string {
  const resolved = edits
    .map((edit, index) => ({
      index,
      start: offsetAt(text, edit.range.start),
      end: offsetAt(text, edit.range.end),
      newText: edit.newText,
    }))
    .sort((left, right) => left.start - right.start || left.index - right.index);
  let output = "";
  let cursor = 0;
  for (const edit of resolved) {
    if (edit.start < cursor) throw new Error("overlapping text edits");
    output += text.slice(cursor, edit.start) + edit.newText;
    cursor = edit.end;
  }
  return output + text.slice(cursor);
}

/** The edits a `WorkspaceEdit` makes to one URI, from `changes` or `documentChanges`. */
export function workspaceEditsFor(edit: unknown, uri: string): TextEdit[] {
  const value = edit as {
    changes?: Record<string, TextEdit[]>;
    documentChanges?: Array<{ textDocument?: { uri: string }; edits?: TextEdit[] }>;
  };
  if (value?.documentChanges != null) {
    return value.documentChanges
      .filter((change) => change.textDocument?.uri === uri)
      .flatMap((change) => change.edits ?? []);
  }
  return value?.changes?.[uri] ?? [];
}

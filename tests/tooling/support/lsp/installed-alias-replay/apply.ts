export type ReplayDocument = { uri: string; text: string; version: number };
export type Position = { line: number; character: number };
type JsonObject = Record<string, unknown>;
type ResolvedEdit = { start: number; end: number; newText: string; index: number };

export class WorkspaceEditFailure extends Error {
  readonly edit: unknown;
  constructor(message: string, edit: unknown) {
    super(message);
    this.name = "WorkspaceEditFailure";
    this.edit = edit;
  }
}

function object(value: unknown, label: string): JsonObject {
  if (!value || typeof value !== "object" || Array.isArray(value))
    throw new Error(`${label} must be an object`);
  return value as JsonObject;
}

function lines(text: string): Array<{ start: number; end: number }> {
  const result: Array<{ start: number; end: number }> = [];
  let start = 0;
  for (let index = 0; index < text.length; index++) {
    if (text[index] !== "\r" && text[index] !== "\n") continue;
    result.push({ start, end: index });
    if (text[index] === "\r" && text[index + 1] === "\n") index++;
    start = index + 1;
  }
  result.push({ start, end: text.length });
  return result;
}

/** Offsets use JS UTF-16 units; invalid positions never clamp onto a different token. */
export function utf16Offset(text: string, value: unknown): number {
  const position = object(value, "position");
  const { line, character } = position;
  if (
    !Number.isSafeInteger(line) ||
    (line as number) < 0 ||
    !Number.isSafeInteger(character) ||
    (character as number) < 0
  )
    throw new Error("invalid UTF16 position");
  const bounds = lines(text)[line as number];
  if (!bounds) throw new Error("line is outside document");
  const { start, end } = bounds;
  if ((character as number) > end - start) throw new Error("character is outside line");
  const offset = start + (character as number);
  if (
    offset > 0 &&
    offset < text.length &&
    /[\uD800-\uDBFF]/.test(text[offset - 1]) &&
    /[\uDC00-\uDFFF]/.test(text[offset])
  )
    throw new Error("position splits a surrogate pair");
  return offset;
}

export function utf16Position(text: string, offset: number): Position {
  if (!Number.isSafeInteger(offset) || offset < 0 || offset > text.length)
    throw new Error("invalid UTF16 offset");
  const bounds = lines(text);
  const line = bounds.findIndex((value) => offset >= value.start && offset <= value.end);
  if (line < 0) throw new Error("offset lies inside a line ending");
  const character = offset - bounds[line].start;
  const position = { line, character };
  if (utf16Offset(text, position) !== offset) throw new Error("offset is not an LSP position");
  return position;
}

/** A validated transaction applies against the original text, never a partial result. */
export function applyWorkspaceEdit(
  edit: unknown,
  documents: readonly ReplayDocument[],
): ReplayDocument[] {
  try {
    const workspace = object(edit, "WorkspaceEdit");
    if (workspace.changes !== undefined && workspace.documentChanges !== undefined)
      throw new Error("both WorkspaceEdit edit forms are present");
    const known = new Map<string, ReplayDocument>();
    for (const document of documents) {
      if (known.has(document.uri)) throw new Error("duplicate input document");
      if (!Number.isSafeInteger(document.version) || document.version < 1)
        throw new Error("invalid input document version");
      known.set(document.uri, document);
    }
    const edits = new Map<string, unknown[]>();
    const add = (uri: unknown, values: unknown, version?: unknown) => {
      if (typeof uri !== "string" || !known.has(uri))
        throw new Error("edit targets an unowned document");
      if (!Array.isArray(values)) throw new Error("text edits must be an array");
      if (version !== undefined && version !== null && version !== known.get(uri)!.version)
        throw new Error("TextDocumentEdit version does not match");
      if (edits.has(uri)) throw new Error("duplicate document edit group");
      edits.set(uri, values);
    };
    if (workspace.changes !== undefined) {
      for (const [uri, values] of Object.entries(object(workspace.changes, "changes")))
        add(uri, values);
    } else if (workspace.documentChanges !== undefined) {
      if (!Array.isArray(workspace.documentChanges))
        throw new Error("documentChanges must be an array");
      for (const raw of workspace.documentChanges) {
        const change = object(raw, "TextDocumentEdit");
        if ("kind" in change)
          throw new Error("resource operation is outside the owned replay documents");
        const identifier = object(change.textDocument, "textDocument");
        if (!("version" in identifier))
          throw new Error("versioned TextDocumentEdit is missing version");
        add(identifier.uri, change.edits, identifier.version);
      }
    }
    const results = new Map<string, string>();
    for (const [uri, rawEdits] of edits) {
      const text = known.get(uri)!.text;
      const resolved: ResolvedEdit[] = rawEdits.map((raw, index) => {
        const value = object(raw, "TextEdit");
        const range = object(value.range, "range");
        if (typeof value.newText !== "string") throw new Error("newText must be a string");
        if (value.annotationId !== undefined) {
          const annotations = object(workspace.changeAnnotations, "changeAnnotations");
          if (
            typeof value.annotationId !== "string" ||
            !Object.hasOwn(annotations, value.annotationId)
          )
            throw new Error("unknown edit annotation");
          const annotation = object(annotations[value.annotationId], "change annotation");
          if (
            typeof annotation.label !== "string" ||
            (annotation.needsConfirmation !== undefined &&
              typeof annotation.needsConfirmation !== "boolean") ||
            (annotation.description !== undefined && typeof annotation.description !== "string")
          )
            throw new Error("malformed edit annotation");
        }
        const start = utf16Offset(text, range.start);
        const end = utf16Offset(text, range.end);
        if (start > end) throw new Error("reversed text edit");
        return { start, end, newText: value.newText, index };
      });
      const ordered = [...resolved].sort(
        (left, right) =>
          left.start - right.start || left.end - right.end || left.index - right.index,
      );
      for (let index = 1; index < ordered.length; index++) {
        const previous = ordered[index - 1];
        const current = ordered[index];
        if (current.start === previous.start && current.end === previous.end)
          throw new Error("duplicate edit range");
        if (current.start < previous.end || current.start === previous.start)
          throw new Error("overlapping text edits");
      }
      let result = text;
      for (const value of ordered.reverse())
        result = result.slice(0, value.start) + value.newText + result.slice(value.end);
      results.set(uri, result);
    }
    return documents.map((document) => ({
      ...document,
      text: results.get(document.uri) ?? document.text,
    }));
  } catch (error) {
    throw new WorkspaceEditFailure(error instanceof Error ? error.message : String(error), edit);
  }
}

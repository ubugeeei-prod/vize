import type { RichTextAttrs, RichTextSchema } from "./rich-text-schema.ts";

// ---------------------------------------------------------------------------
// Typed document (derived from the schema)
// ---------------------------------------------------------------------------

/** One applied mark. */
export type RichTextMark<Schema extends RichTextSchema> = {
  [Name in keyof Schema["marks"] & string]: {
    readonly type: Name;
    readonly attrs: Schema["marks"][Name]["defaults"];
  };
}[keyof Schema["marks"] & string];

/** A run of text with marks. */
export interface RichTextText<Schema extends RichTextSchema> {
  readonly type: "text";
  readonly text: string;
  readonly marks: readonly RichTextMark<Schema>[];
}

/** Atomic inline leaf such as an image. */
export type RichTextInlineLeaf<Schema extends RichTextSchema> = {
  [Name in keyof Schema["nodes"] & string]: Schema["nodes"][Name]["role"] extends "inline"
    ? {
        readonly type: Name;
        readonly attrs: Schema["nodes"][Name]["defaults"];
        readonly marks: readonly RichTextMark<Schema>[];
      }
    : never;
}[keyof Schema["nodes"] & string];

/** Inline content of a textblock. */
export type RichTextInline<Schema extends RichTextSchema> =
  | RichTextInlineLeaf<Schema>
  | RichTextText<Schema>;

/** Any block node; textblocks hold inline content, the rest hold blocks. */
export type RichTextBlock<Schema extends RichTextSchema> = {
  [Name in keyof Schema["nodes"] & string]: Schema["nodes"][Name]["role"] extends "inline"
    ? never
    : {
        readonly type: Name;
        readonly attrs: Schema["nodes"][Name]["defaults"];
        readonly content: Schema["nodes"][Name]["role"] extends "textblock"
          ? readonly RichTextInline<Schema>[]
          : readonly RichTextBlock<Schema>[];
      };
}[keyof Schema["nodes"] & string];

/** A complete document typed by its schema (assignable to {@link RtDoc}). */
export interface RichTextDoc<Schema extends RichTextSchema = RichTextSchema> {
  readonly type: "doc";
  readonly content: readonly RichTextBlock<Schema>[];
}

// ---------------------------------------------------------------------------
// Value-erased runtime shapes (every typed document is assignable to these)
// ---------------------------------------------------------------------------

/** Erased mark. */
export interface RtMark {
  readonly type: string;
  readonly attrs: RichTextAttrs;
}

/** Erased text run. */
export interface RtText {
  readonly type: "text";
  readonly text: string;
  readonly marks: readonly RtMark[];
}

/** Erased inline leaf. */
export interface RtLeaf {
  readonly type: string;
  readonly attrs: RichTextAttrs;
  readonly marks: readonly RtMark[];
}

/** Erased inline node. */
export type RtInline = RtLeaf | RtText;

/** Erased block. */
export interface RtBlock {
  readonly type: string;
  readonly attrs: RichTextAttrs;
  readonly content: readonly (RtBlock | RtInline)[];
}

/** Erased document. */
export interface RtDoc {
  readonly type: "doc";
  readonly content: readonly RtBlock[];
}

/** Position inside a textblock: `path` indexes blocks from the doc, `offset` counts inline units. */
export interface RichTextPosition {
  /** Child indexes from the document to a textblock. */
  readonly path: readonly number[];

  /** Characters (text) and leaves (1 each) before the position. */
  readonly offset: number;
}

/** Selection between two positions; `head` moves with Shift+arrows. */
export interface RichTextSelection {
  readonly anchor: RichTextPosition;
  readonly head: RichTextPosition;
}

/** A located textblock. */
export interface RtTextblockEntry {
  readonly path: readonly number[];
  readonly node: RtBlock;
}

const modelDiagnostic = "VIZE_UI_RICH_TEXT_MODEL";

/** Whether an inline node is a text run. */
export function isText(node: RtBlock | RtInline): node is RtText {
  return node.type === "text" && "text" in node;
}

/** Whether a node is a block (has `content`). */
export function isBlock(node: RtBlock | RtInline): node is RtBlock {
  return "content" in node;
}

/** Whether a node is inline (text or leaf). */
export function isInline(node: RtBlock | RtInline): node is RtInline {
  return !("content" in node);
}

/** Inline children of a textblock. */
export function inlineContent(block: RtBlock): RtInline[] {
  return block.content.filter(isInline);
}

/** Block children of a non-textblock. */
export function blockContent(block: RtBlock | RtDoc): RtBlock[] {
  const children: readonly (RtBlock | RtInline)[] = block.content;
  return children.filter(isBlock);
}

/** Inline length of one inline node. */
export function inlineSize(node: RtInline): number {
  return isText(node) ? node.text.length : 1;
}

/** Inline length of a textblock. */
export function textblockSize(block: RtBlock): number {
  return inlineContent(block).reduce((total, node) => total + inlineSize(node), 0);
}

/** Role of a node type in `schema` (text nodes are inline). */
export function roleOf(schema: RichTextSchema, type: string) {
  if (type === "text") return "inline";
  const spec = schema.nodes[type];
  if (!spec) throw new Error(`${modelDiagnostic}: unknown node type "${type}"`);
  return spec.role;
}

/** Plain text of a block tree (textblocks joined by newlines). */
export function textContent(node: RtBlock | RtDoc): string {
  const parts: string[] = [];
  const visit = (block: RtBlock | RtDoc): void => {
    const inline = block.content.filter(isInline);
    if (inline.length > 0 || block.content.length === 0) {
      if (block.type !== "doc") {
        parts.push(inline.map((child) => (isText(child) ? child.text : "")).join(""));
      }
      return;
    }
    for (const child of blockContent(block)) visit(child);
  };
  visit(node);
  return parts.join("\n");
}

/** Every textblock in document order. */
export function textblocks(schema: RichTextSchema, doc: RtDoc): RtTextblockEntry[] {
  const found: RtTextblockEntry[] = [];
  const visit = (children: readonly RtBlock[], prefix: readonly number[]): void => {
    children.forEach((child, index) => {
      const path = [...prefix, index];
      if (roleOf(schema, child.type) === "textblock") found.push({ path, node: child });
      else visit(blockContent(child), path);
    });
  };
  visit(doc.content, []);
  return found;
}

/** Block at `path`, or `undefined`. */
export function blockAt(doc: RtDoc, path: readonly number[]): RtBlock | undefined {
  let children: readonly (RtBlock | RtInline)[] = doc.content;
  let node: RtBlock | undefined;
  for (const index of path) {
    const next = children[index];
    if (!next || !isBlock(next)) return undefined;
    node = next;
    children = next.content;
  }
  return node;
}

/** Ancestors of `path` from the outermost block down to (excluding) the node itself. */
export function ancestors(
  doc: RtDoc,
  path: readonly number[],
): { path: number[]; node: RtBlock }[] {
  const found: { path: number[]; node: RtBlock }[] = [];
  for (let depth = 1; depth < path.length; depth++) {
    const prefix = path.slice(0, depth);
    const node = blockAt(doc, prefix);
    if (node) found.push({ path: prefix, node });
  }
  return found;
}

/** Replace the block at `path` (or delete it with `null`, or insert siblings). */
export function updateAt(
  doc: RtDoc,
  path: readonly number[],
  replace: (node: RtBlock) => readonly RtBlock[],
): RtDoc {
  const rebuild = (
    children: readonly (RtBlock | RtInline)[],
    depth: number,
  ): (RtBlock | RtInline)[] => {
    const index = path[depth];
    if (index === undefined) return [...children];
    const target = children[index];
    if (!target || !isBlock(target)) {
      throw new Error(`${modelDiagnostic}: no block at ${path.join(".")}`);
    }
    if (depth === path.length - 1) {
      return [...children.slice(0, index), ...replace(target), ...children.slice(index + 1)];
    }
    return [
      ...children.slice(0, index),
      { ...target, content: rebuild(target.content, depth + 1) },
      ...children.slice(index + 1),
    ];
  };
  return { type: "doc", content: rebuild(doc.content, 0).filter(isBlock) };
}

/** Compare two positions in document order. */
export function comparePositions(left: RichTextPosition, right: RichTextPosition): number {
  const length = Math.min(left.path.length, right.path.length);
  for (let index = 0; index < length; index++) {
    const difference = (left.path[index] ?? 0) - (right.path[index] ?? 0);
    if (difference !== 0) return difference;
  }
  if (left.path.length !== right.path.length) return left.path.length - right.path.length;
  return left.offset - right.offset;
}

/** Whether two paths are equal. */
export function samePath(left: readonly number[], right: readonly number[]): boolean {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

/** Ordered `[from, to]` of a selection. */
export function selectionRange(selection: RichTextSelection): [RichTextPosition, RichTextPosition] {
  return comparePositions(selection.anchor, selection.head) <= 0
    ? [selection.anchor, selection.head]
    : [selection.head, selection.anchor];
}

/** Whether a selection is collapsed. */
export function isCollapsed(selection: RichTextSelection): boolean {
  return comparePositions(selection.anchor, selection.head) === 0;
}

/** Collapsed selection at `position`. */
export function caret(position: RichTextPosition): RichTextSelection {
  return { anchor: position, head: position };
}

/** Whether two mark sets are equal (order-insensitive). */
export function sameMarks(left: readonly RtMark[], right: readonly RtMark[]): boolean {
  return (
    left.length === right.length &&
    left.every((mark) => right.some((other) => sameMark(mark, other)))
  );
}

/** Whether two marks are equal including attributes. */
export function sameMark(left: RtMark, right: RtMark): boolean {
  if (left.type !== right.type) return false;
  const keys = new Set([...Object.keys(left.attrs), ...Object.keys(right.attrs)]);
  for (const key of keys) if (left.attrs[key] !== right.attrs[key]) return false;
  return true;
}

/** Merge adjacent equal-mark text runs and drop empty ones. */
export function normalizeInline(nodes: readonly RtInline[]): RtInline[] {
  const result: RtInline[] = [];
  for (const node of nodes) {
    if (isText(node)) {
      if (node.text.length === 0) continue;
      const previous = result.at(-1);
      if (previous && isText(previous) && sameMarks(previous.marks, node.marks)) {
        result[result.length - 1] = { ...previous, text: previous.text + node.text };
        continue;
      }
    }
    result.push(node);
  }
  return result;
}

/** Inline nodes between `from` and `to` offsets. */
export function sliceInline(nodes: readonly RtInline[], from: number, to: number): RtInline[] {
  const result: RtInline[] = [];
  let position = 0;
  for (const node of nodes) {
    const size = inlineSize(node);
    const start = position;
    const end = position + size;
    position = end;
    if (end <= from || start >= to) continue;
    if (isText(node)) {
      result.push({
        ...node,
        text: node.text.slice(Math.max(0, from - start), Math.min(size, to - start)),
      });
    } else {
      result.push(node);
    }
  }
  return normalizeInline(result);
}

/** Marks in effect just before `offset` of a textblock. */
export function marksBefore(
  schema: RichTextSchema,
  block: RtBlock,
  offset: number,
): readonly RtMark[] {
  let position = 0;
  let found: readonly RtMark[] = [];
  for (const node of inlineContent(block)) {
    const size = inlineSize(node);
    if (offset > position && offset <= position + size) {
      found = node.marks;
      // Non-inclusive marks (links) do not extend past their end.
      if (offset === position + size) {
        found = found.filter((mark) => schema.marks[mark.type]?.inclusive !== false);
      }
      break;
    }
    position += size;
  }
  return found;
}

/** Empty textblock of the default type. */
export function emptyBlock(schema: RichTextSchema, type = schema.defaultBlock): RtBlock {
  return { type, attrs: schema.nodes[type]?.defaults ?? {}, content: [] };
}

/** Empty document with one default block. */
export function emptyRichTextDoc<Schema extends RichTextSchema>(
  schema: Schema,
): RichTextDoc<Schema> {
  const doc: RtDoc = { type: "doc", content: [emptyBlock(schema)] };
  return assertRichTextDoc(schema, doc);
}

/** Document of plain-text paragraphs (lines split on newlines). */
export function richTextDocFromText<Schema extends RichTextSchema>(
  schema: Schema,
  text: string,
): RichTextDoc<Schema> {
  const blocks = text.split(/\r\n|\r|\n/u).map((line): RtBlock => ({
    ...emptyBlock(schema),
    content: line.length > 0 ? [{ type: "text", text: line, marks: [] }] : [],
  }));
  return assertRichTextDoc(schema, { type: "doc", content: blocks });
}

const validated = new WeakMap<RichTextSchema, WeakSet<object>>();

/** Why a value is not a valid document for a schema, or `null` when it is. */
export function validateRichTextDoc(schema: RichTextSchema, value: unknown): string | null {
  // Documents are immutable, so a document validated once for a schema stays valid.
  if (isRecord(value) && validated.get(schema)?.has(value)) return null;
  const error = validateUncached(schema, value);
  if (error === null && isRecord(value)) {
    let known = validated.get(schema);
    if (!known) {
      known = new WeakSet();
      validated.set(schema, known);
    }
    known.add(value);
  }
  return error;
}

/** Type guard to the value-erased document shape, backed by schema validation. */
export function isRtDoc(schema: RichTextSchema, value: unknown): value is RtDoc {
  return validateRichTextDoc(schema, value) === null;
}

/** Erase a (typed) document to {@link RtDoc}, re-checking it against the schema. */
export function toRtDoc(schema: RichTextSchema, value: unknown): RtDoc {
  if (!isRtDoc(schema, value)) throw new TypeError(`${modelDiagnostic}: invalid document`);
  return value;
}

function validateUncached(schema: RichTextSchema, value: unknown): string | null {
  if (!isRecord(value) || value.type !== "doc" || !Array.isArray(value.content)) {
    return 'root must be { type: "doc", content: [] }';
  }
  if (value.content.length === 0) return "document must contain at least one block";
  const visitBlock = (node: unknown, where: string, parentRole: string): string | null => {
    if (!isRecord(node) || typeof node.type !== "string") return `${where}: not a node`;
    const spec = schema.nodes[node.type];
    if (!spec || spec.role === "inline")
      return `${where}: "${String(node.type)}" is not a block type`;
    if (!isRecord(node.attrs)) return `${where}: missing attrs`;
    if (!Array.isArray(node.content)) return `${where}: missing content`;
    if (parentRole === "list" && spec.role !== "listItem") return `${where}: lists hold list items`;
    if (spec.role === "listItem" && parentRole !== "list")
      return `${where}: list items need a list`;
    for (const [index, child] of node.content.entries()) {
      const childWhere = `${where}.${index}`;
      const error =
        spec.role === "textblock"
          ? visitInline(child, childWhere, spec.code === true)
          : visitBlock(child, childWhere, spec.role);
      if (error) return error;
    }
    return null;
  };
  const visitInline = (node: unknown, where: string, code: boolean): string | null => {
    if (!isRecord(node) || typeof node.type !== "string" || !Array.isArray(node.marks)) {
      return `${where}: not an inline node`;
    }
    if (code && node.marks.length > 0) return `${where}: code blocks hold unmarked text`;
    for (const mark of node.marks) {
      if (!isRecord(mark) || typeof mark.type !== "string" || !schema.marks[mark.type]) {
        return `${where}: unknown mark`;
      }
    }
    if (node.type === "text")
      return typeof node.text === "string" ? null : `${where}: text needs a string`;
    if (code) return `${where}: code blocks hold text only`;
    const spec = schema.nodes[node.type];
    if (spec?.role !== "inline" || !isRecord(node.attrs)) return `${where}: unknown inline node`;
    return null;
  };
  for (const [index, child] of value.content.entries()) {
    const error = visitBlock(child, String(index), "doc");
    if (error) return error;
  }
  return null;
}

/** Type guard backed by {@link validateRichTextDoc}. */
export function isRichTextDoc<Schema extends RichTextSchema>(
  schema: Schema,
  value: unknown,
): value is RichTextDoc<Schema> {
  return validateRichTextDoc(schema, value) === null;
}

/** Narrow to a typed document or throw a diagnostic describing the first problem. */
export function assertRichTextDoc<Schema extends RichTextSchema>(
  schema: Schema,
  value: unknown,
): RichTextDoc<Schema> {
  const error = validateRichTextDoc(schema, value);
  if (error !== null || !isRichTextDoc(schema, value)) {
    throw new TypeError(`${modelDiagnostic}: invalid document (${error ?? "unknown"})`);
  }
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

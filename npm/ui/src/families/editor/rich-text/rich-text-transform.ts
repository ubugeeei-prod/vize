import {
  blockAt,
  blockContent,
  comparePositions,
  emptyBlock,
  inlineContent,
  inlineSize,
  isText,
  normalizeInline,
  roleOf,
  sameMark,
  samePath,
  sliceInline,
  textblocks,
  textblockSize,
  updateAt,
} from "./rich-text-model.ts";
import type {
  RichTextPosition,
  RtBlock,
  RtDoc,
  RtInline,
  RtMark,
  RtTextblockEntry,
} from "./rich-text-model.ts";
import type { RichTextAttrs, RichTextSchema } from "./rich-text-schema.ts";

/** Result of a document transform: new doc plus where the caret lands. */
export interface RtTransformResult {
  readonly doc: RtDoc;
  readonly position: RichTextPosition;
}

const transformDiagnostic = "VIZE_UI_RICH_TEXT_TRANSFORM";

function entryIndex(entries: readonly RtTextblockEntry[], position: RichTextPosition): number {
  const index = entries.findIndex((entry) => samePath(entry.path, position.path));
  if (index === -1) {
    throw new Error(`${transformDiagnostic}: ${position.path.join(".")} is not a textblock`);
  }
  return index;
}

/** Clamp a position into its textblock (or the nearest textblock). */
export function clampPosition(
  schema: RichTextSchema,
  doc: RtDoc,
  position: RichTextPosition,
): RichTextPosition {
  const entries = textblocks(schema, doc);
  const exact = entries.find((entry) => samePath(entry.path, position.path));
  if (exact) {
    return {
      path: exact.path,
      offset: Math.max(0, Math.min(position.offset, textblockSize(exact.node))),
    };
  }
  const after = entries.find(
    (entry) => comparePositions({ path: entry.path, offset: 0 }, position) >= 0,
  );
  const target = after ?? entries.at(-1);
  if (!target) throw new Error(`${transformDiagnostic}: document has no textblocks`);
  return { path: target.path, offset: after ? 0 : textblockSize(target.node) };
}

/**
 * Remove empty wrappers (including emptied list items), keep at least one
 * block, and strip marks/leaves from code blocks.
 */
export function normalizeDoc(schema: RichTextSchema, doc: RtDoc): RtDoc {
  const fixBlock = (block: RtBlock): RtBlock | null => {
    const role = roleOf(schema, block.type);
    if (role === "textblock") {
      const inline = inlineContent(block);
      const content = schema.nodes[block.type]?.code
        ? normalizeInline(
            inline.flatMap((node): RtInline[] =>
              isText(node) ? [{ type: "text", text: node.text, marks: [] }] : [],
            ),
          )
        : normalizeInline(inline);
      return { ...block, content };
    }
    const children = blockContent(block)
      .map(fixBlock)
      .filter((child): child is RtBlock => child !== null);
    return children.length > 0 ? { ...block, content: children } : null;
  };
  const content = doc.content.map(fixBlock).filter((block): block is RtBlock => block !== null);
  return { type: "doc", content: content.length > 0 ? content : [emptyBlock(schema)] };
}

/** Replace the range `[from, to]` with inline nodes, merging across blocks. */
export function replaceRange(
  schema: RichTextSchema,
  doc: RtDoc,
  from: RichTextPosition,
  to: RichTextPosition,
  inline: readonly RtInline[],
): RtTransformResult {
  const entries = textblocks(schema, doc);
  const fromIndex = entryIndex(entries, from);
  const toIndex = entryIndex(entries, to);
  const first = entries[fromIndex];
  const last = entries[toIndex];
  if (!first || !last || toIndex < fromIndex) {
    throw new Error(`${transformDiagnostic}: invalid range`);
  }
  const code = schema.nodes[first.node.type]?.code === true;
  const inserted = code
    ? inline.flatMap((node): RtInline[] => (isText(node) ? [{ ...node, marks: [] }] : []))
    : inline;
  const head = sliceInline(inlineContent(first.node), 0, from.offset);
  const tail = sliceInline(inlineContent(last.node), to.offset, textblockSize(last.node));
  let next = doc;
  for (let index = toIndex; index > fromIndex; index--) {
    const entry = entries[index];
    if (entry) next = updateAt(next, entry.path, () => []);
  }
  next = updateAt(next, first.path, (node) => [
    { ...node, content: normalizeInline([...head, ...inserted, ...tail]) },
  ]);
  const insertedSize = inserted.reduce((total, node) => total + inlineSize(node), 0);
  const normalized = normalizeDoc(schema, next);
  return {
    doc: normalized,
    position: clampPosition(schema, normalized, {
      path: first.path,
      offset: from.offset + insertedSize,
    }),
  };
}

/** Split the textblock at `position` (splitting its list item when inside one). */
export function splitTextblock(
  schema: RichTextSchema,
  doc: RtDoc,
  position: RichTextPosition,
): RtTransformResult {
  const block = blockAt(doc, position.path);
  if (!block) throw new Error(`${transformDiagnostic}: no textblock to split`);
  const size = textblockSize(block);
  const before: RtBlock = {
    ...block,
    content: sliceInline(inlineContent(block), 0, position.offset),
  };
  const afterType =
    position.offset >= size && block.type !== schema.defaultBlock ? emptyBlock(schema) : block;
  const after: RtBlock = {
    ...afterType,
    content: sliceInline(inlineContent(block), position.offset, size),
  };
  const parentPath = position.path.slice(0, -1);
  const index = position.path.at(-1) ?? 0;
  const parent = parentPath.length > 0 ? blockAt(doc, parentPath) : undefined;
  if (parent && roleOf(schema, parent.type) === "listItem") {
    const siblings = blockContent(parent);
    const itemIndex = parentPath.at(-1) ?? 0;
    const next = updateAt(doc, parentPath, (item) => [
      { ...item, content: [...siblings.slice(0, index), before] },
      { ...item, content: [after, ...siblings.slice(index + 1)] },
    ]);
    return {
      doc: normalizeDoc(schema, next),
      position: { path: [...parentPath.slice(0, -1), itemIndex + 1, 0], offset: 0 },
    };
  }
  const next = updateAt(doc, position.path, () => [before, after]);
  return {
    doc: normalizeDoc(schema, next),
    position: { path: [...parentPath, index + 1], offset: 0 },
  };
}

/** Change every textblock in `[from, to]` to `type`. */
export function setTextblockType(
  schema: RichTextSchema,
  doc: RtDoc,
  from: RichTextPosition,
  to: RichTextPosition,
  type: string,
  attrs: RichTextAttrs,
): RtDoc {
  const entries = textblocks(schema, doc);
  let next = doc;
  for (const entry of entries.slice(entryIndex(entries, from), entryIndex(entries, to) + 1)) {
    next = updateAt(next, entry.path, (node) => [{ ...node, type, attrs }]);
  }
  return normalizeDoc(schema, next);
}

function mapMarks(
  nodes: readonly RtInline[],
  from: number,
  to: number,
  change: (marks: readonly RtMark[]) => readonly RtMark[],
): RtInline[] {
  const result: RtInline[] = [];
  let position = 0;
  for (const node of nodes) {
    const size = inlineSize(node);
    const start = position;
    const end = position + size;
    position = end;
    if (end <= from || start >= to) {
      result.push(node);
      continue;
    }
    if (!isText(node)) {
      result.push({ ...node, marks: change(node.marks) });
      continue;
    }
    const cutStart = Math.max(from, start) - start;
    const cutEnd = Math.min(to, end) - start;
    if (cutStart > 0) result.push({ ...node, text: node.text.slice(0, cutStart) });
    result.push({ ...node, text: node.text.slice(cutStart, cutEnd), marks: change(node.marks) });
    if (cutEnd < size) result.push({ ...node, text: node.text.slice(cutEnd) });
  }
  return normalizeInline(result);
}

/** Visit the inline segment of every textblock in `[from, to]`. */
export function rangeSegments(
  schema: RichTextSchema,
  doc: RtDoc,
  from: RichTextPosition,
  to: RichTextPosition,
): { entry: RtTextblockEntry; start: number; end: number }[] {
  const entries = textblocks(schema, doc);
  const fromIndex = entryIndex(entries, from);
  const toIndex = entryIndex(entries, to);
  return entries.slice(fromIndex, toIndex + 1).map((entry, index) => ({
    entry,
    start: index === 0 ? from.offset : 0,
    end: fromIndex + index === toIndex ? to.offset : textblockSize(entry.node),
  }));
}

/** Add (replacing same-type marks) or remove a mark across `[from, to]`. */
export function changeMark(
  schema: RichTextSchema,
  doc: RtDoc,
  from: RichTextPosition,
  to: RichTextPosition,
  mark: RtMark,
  add: boolean,
): RtDoc {
  let next = doc;
  for (const { entry, start, end } of rangeSegments(schema, doc, from, to)) {
    if (schema.nodes[entry.node.type]?.code || start === end) continue;
    next = updateAt(next, entry.path, (node) => [
      {
        ...node,
        content: mapMarks(inlineContent(node), start, end, (marks) => {
          const others = marks.filter((candidate) => candidate.type !== mark.type);
          return add ? [...others, mark] : others;
        }),
      },
    ]);
  }
  return next;
}

/** Whether every non-empty text in `[from, to]` carries `type` (optionally with `attrs`). */
export function rangeHasMark(
  schema: RichTextSchema,
  doc: RtDoc,
  from: RichTextPosition,
  to: RichTextPosition,
  mark: RtMark | string,
): boolean {
  let seen = false;
  for (const { entry, start, end } of rangeSegments(schema, doc, from, to)) {
    for (const node of sliceInline(inlineContent(entry.node), start, end)) {
      seen = true;
      const has = node.marks.some((candidate) =>
        typeof mark === "string" ? candidate.type === mark : sameMark(candidate, mark),
      );
      if (!has) return false;
    }
  }
  return seen;
}

/** Common parent and sibling span covering `[from, to]`, lifted out of lists. */
function wrapSpan(
  schema: RichTextSchema,
  doc: RtDoc,
  from: RichTextPosition,
  to: RichTextPosition,
): { parentPath: number[]; start: number; end: number } {
  let common = 0;
  while (
    common < from.path.length - 1 &&
    common < to.path.length - 1 &&
    from.path[common] === to.path[common]
  ) {
    common++;
  }
  let parentPath = from.path.slice(0, common);
  let start = from.path[common] ?? 0;
  let end = to.path[common] ?? start;
  // Items cannot leave their list: wrap the whole list instead.
  while (parentPath.length > 0) {
    const parent = blockAt(doc, parentPath);
    if (!parent || roleOf(schema, parent.type) !== "list") break;
    start = parentPath.at(-1) ?? 0;
    end = start;
    parentPath = parentPath.slice(0, -1);
  }
  return { parentPath, start, end };
}

/** Wrap the blocks covering `[from, to]` in a container or list. */
export function wrapRange(
  schema: RichTextSchema,
  doc: RtDoc,
  from: RichTextPosition,
  to: RichTextPosition,
  type: string,
  attrs: RichTextAttrs,
): { doc: RtDoc; map: (position: RichTextPosition) => RichTextPosition } {
  const role = roleOf(schema, type);
  if (role !== "container" && role !== "list") {
    throw new Error(`${transformDiagnostic}: "${type}" cannot wrap blocks`);
  }
  const itemType = Object.keys(schema.nodes).find(
    (name) => schema.nodes[name]?.role === "listItem",
  );
  const { parentPath, start, end } = wrapSpan(schema, doc, from, to);
  const siblings =
    parentPath.length > 0 ? blockContent(blockAt(doc, parentPath) ?? doc) : doc.content;
  const covered = siblings.slice(start, end + 1);
  const wrapper: RtBlock = {
    type,
    attrs,
    content:
      role === "list" && itemType
        ? covered.map((block) => ({
            type: itemType,
            attrs: schema.nodes[itemType]?.defaults ?? {},
            content: [block],
          }))
        : covered,
  };
  const replaceChildren = (children: readonly RtBlock[]): RtBlock[] => [
    ...children.slice(0, start),
    wrapper,
    ...children.slice(end + 1),
  ];
  const next: RtDoc =
    parentPath.length === 0
      ? { type: "doc", content: replaceChildren(doc.content) }
      : updateAt(doc, parentPath, (parent) => [
          { ...parent, content: replaceChildren(blockContent(parent)) },
        ]);
  const depth = parentPath.length;
  const map = (position: RichTextPosition): RichTextPosition => {
    const index = position.path[depth];
    if (!samePath(position.path.slice(0, depth), parentPath) || index === undefined)
      return position;
    if (index < start) return position;
    if (index > end) {
      return {
        ...position,
        path: [...parentPath, index - (end - start), ...position.path.slice(depth + 1)],
      };
    }
    const inner = role === "list" ? [index - start, 0] : [index - start];
    return {
      ...position,
      path: [...parentPath, start, ...inner, ...position.path.slice(depth + 1)],
    };
  };
  return { doc: normalizeDoc(schema, next), map };
}

/** Nearest ancestor of `position` whose type satisfies `match`. */
export function findWrapper(
  schema: RichTextSchema,
  doc: RtDoc,
  position: RichTextPosition,
  match: (type: string) => boolean,
): { path: number[]; node: RtBlock } | null {
  for (let depth = position.path.length - 1; depth >= 1; depth--) {
    const path = position.path.slice(0, depth);
    const node = blockAt(doc, path);
    if (node && match(node.type)) return { path, node };
  }
  return null;
}

/** Unwrap the wrapper at `wrapperPath`, splicing its (list items') children into its parent. */
export function liftWrapper(
  schema: RichTextSchema,
  doc: RtDoc,
  wrapperPath: readonly number[],
): { doc: RtDoc; map: (position: RichTextPosition) => RichTextPosition } {
  const wrapper = blockAt(doc, wrapperPath);
  if (!wrapper) throw new Error(`${transformDiagnostic}: no wrapper to lift`);
  const isList = roleOf(schema, wrapper.type) === "list";
  const groups = blockContent(wrapper).map((child) => (isList ? blockContent(child) : [child]));
  const next = updateAt(doc, wrapperPath, () => groups.flat());
  const depth = wrapperPath.length;
  const base = wrapperPath.at(-1) ?? 0;
  const lifted = groups.flat().length;
  const map = (position: RichTextPosition): RichTextPosition => {
    const parentPath = wrapperPath.slice(0, -1);
    if (!samePath(position.path.slice(0, depth - 1), parentPath)) return position;
    const index = position.path[depth - 1];
    if (index === undefined || index < base) return position;
    if (index > base) {
      return {
        ...position,
        path: [...parentPath, index + lifted - 1, ...position.path.slice(depth)],
      };
    }
    const child = position.path[depth] ?? 0;
    if (!isList)
      return {
        ...position,
        path: [...parentPath, base + child, ...position.path.slice(depth + 1)],
      };
    const before = groups.slice(0, child).reduce((total, group) => total + group.length, 0);
    const within = position.path[depth + 1] ?? 0;
    return {
      ...position,
      path: [...parentPath, base + before + within, ...position.path.slice(depth + 2)],
    };
  };
  return { doc: normalizeDoc(schema, next), map };
}

/** Move one list item's blocks out of its list, splitting the list around it. */
export function liftListItem(
  schema: RichTextSchema,
  doc: RtDoc,
  itemPath: readonly number[],
): { doc: RtDoc; map: (position: RichTextPosition) => RichTextPosition } {
  const listPath = itemPath.slice(0, -1);
  const list = blockAt(doc, listPath);
  const itemIndex = itemPath.at(-1) ?? 0;
  if (!list || roleOf(schema, list.type) !== "list") {
    throw new Error(`${transformDiagnostic}: ${itemPath.join(".")} is not a list item`);
  }
  const items = blockContent(list);
  const item = items[itemIndex];
  if (!item) throw new Error(`${transformDiagnostic}: missing list item`);
  const before = items.slice(0, itemIndex);
  const after = items.slice(itemIndex + 1);
  const replacement: RtBlock[] = [
    ...(before.length > 0 ? [{ ...list, content: before }] : []),
    ...blockContent(item),
    ...(after.length > 0 ? [{ ...list, content: after }] : []),
  ];
  const next = updateAt(doc, listPath, () => replacement);
  const depth = listPath.length;
  const parentPath = listPath.slice(0, -1);
  const listIndex = listPath.at(-1) ?? 0;
  const shift = before.length > 0 ? 1 : 0;
  const map = (position: RichTextPosition): RichTextPosition => {
    if (!samePath(position.path.slice(0, depth), listPath)) return position;
    const index = position.path[depth] ?? 0;
    const child = position.path[depth + 1] ?? 0;
    if (index === itemIndex) {
      return {
        ...position,
        path: [...parentPath, listIndex + shift + child, ...position.path.slice(depth + 2)],
      };
    }
    if (index > itemIndex) {
      return {
        ...position,
        path: [
          ...parentPath,
          listIndex + shift + blockContent(item).length,
          index - itemIndex - 1,
          ...position.path.slice(depth + 1),
        ],
      };
    }
    return position;
  };
  return { doc: normalizeDoc(schema, next), map };
}

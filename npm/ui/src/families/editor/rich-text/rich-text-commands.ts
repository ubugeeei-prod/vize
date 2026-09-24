import {
  blockAt,
  caret,
  comparePositions,
  inlineContent,
  isCollapsed,
  isText,
  marksBefore,
  roleOf,
  samePath,
  selectionRange,
  sliceInline,
  textblocks,
  textblockSize,
  textContent,
  updateAt,
} from "./rich-text-model.ts";
import type {
  RichTextPosition,
  RichTextSelection,
  RichTextDoc,
  RtBlock,
  RtDoc,
  RtInline,
  RtMark,
} from "./rich-text-model.ts";
import type {
  RichTextAttrs,
  RichTextAttrValue,
  RichTextMarkName,
  RichTextNodeName,
  RichTextSchema,
} from "./rich-text-schema.ts";
import { createTransaction, redo, undo } from "./rich-text-state.ts";
import type {
  RichTextCommand,
  RichTextDispatch,
  RichTextTransactionKind,
  RtState,
} from "./rich-text-state.ts";
import {
  changeMark,
  findWrapper,
  liftListItem,
  liftWrapper,
  rangeHasMark,
  replaceRange,
  setTextblockType,
  normalizeDoc,
  splitTextblock,
  wrapRange,
} from "./rich-text-transform.ts";

/** Attributes accepted when applying a node or mark: any subset of its defaults. */
type AttrsOf<Defaults> = Partial<Defaults>;

/** Attribute overrides as accepted internally (typed at the public boundary). */
type AttrsInput = { readonly [name: string]: RichTextAttrValue | undefined };

function withDefaults(defaults: RichTextAttrs, attrs: AttrsInput | undefined): RichTextAttrs {
  const merged: { [name: string]: RichTextAttrValue } = { ...defaults };
  for (const [name, value] of Object.entries(attrs ?? {})) {
    if (value !== undefined) merged[name] = value;
  }
  return merged;
}

function isCode(state: RtState, position: RichTextPosition): boolean {
  const block = blockAt(state.doc, position.path);
  return block !== undefined && state.schema.nodes[block.type]?.code === true;
}

function replaceSelection(
  state: RtState,
  inline: readonly RtInline[],
  kind: RichTextTransactionKind,
  dispatch: RichTextDispatch | undefined,
): boolean {
  const [from, to] = selectionRange(state.selection);
  const result = replaceRange(state.schema, state.doc, from, to, inline);
  dispatch?.(
    createTransaction(state, { doc: result.doc, selection: caret(result.position) }, kind),
  );
  return true;
}

/** Marks typed text receives at the current caret. */
export function activeMarks(state: RtState): readonly RtMark[] {
  if (state.storedMarks) return state.storedMarks;
  const [from] = selectionRange(state.selection);
  const block = blockAt(state.doc, from.path);
  return block ? marksBefore(state.schema, block, from.offset) : [];
}

/** Insert text at the selection with the active marks. */
export function insertText(text: string): RichTextCommand {
  return (state, dispatch) => {
    if (text.length === 0) return false;
    const [from] = selectionRange(state.selection);
    const marks = isCode(state, from) ? [] : activeMarks(state);
    return replaceSelection(state, [{ type: "text", text, marks }], "input", dispatch);
  };
}

/** Delete a non-empty selection. */
export function deleteSelection(state: RtState, dispatch?: RichTextDispatch): boolean {
  if (isCollapsed(state.selection)) return false;
  return replaceSelection(state, [], "delete", dispatch);
}

function previousBoundary(text: string, offset: number, unit: "character" | "word"): number {
  if (unit === "word") {
    const before = text.slice(0, offset);
    const match = /[\p{L}\p{N}_]+[^\p{L}\p{N}_]*$|[^\p{L}\p{N}_]+$/u.exec(before);
    return match ? offset - match[0].length : Math.max(0, offset - 1);
  }
  const code = text.charCodeAt(offset - 1);
  return code >= 0xdc00 && code <= 0xdfff && offset >= 2 ? offset - 2 : offset - 1;
}

function nextBoundary(text: string, offset: number, unit: "character" | "word"): number {
  if (unit === "word") {
    const match = /^[^\p{L}\p{N}_]*[\p{L}\p{N}_]+|^[^\p{L}\p{N}_]+/u.exec(text.slice(offset));
    return match ? offset + match[0].length : offset + 1;
  }
  const code = text.charCodeAt(offset);
  return code >= 0xd800 && code <= 0xdbff ? offset + 2 : offset + 1;
}

function plainText(state: RtState, position: RichTextPosition): string {
  const block = blockAt(state.doc, position.path);
  if (!block) return "";
  // Leaves count as one unit; represent them with a placeholder character.
  return inlineContent(block)
    .map((node) => (isText(node) ? node.text : "￼"))
    .join("");
}

/** Backspace: delete the selection, the previous character/word, or join with the previous block. */
export function deleteBackward(unit: "character" | "word" = "character"): RichTextCommand {
  return (state, dispatch) => {
    if (!isCollapsed(state.selection)) return deleteSelection(state, dispatch);
    const position = state.selection.head;
    if (position.offset > 0) {
      const from = {
        ...position,
        offset: previousBoundary(plainText(state, position), position.offset, unit),
      };
      const result = replaceRange(state.schema, state.doc, from, position, []);
      dispatch?.(
        createTransaction(state, { doc: result.doc, selection: caret(result.position) }, "delete"),
      );
      return true;
    }
    return joinBackward(state, dispatch);
  };
}

/** At the start of a textblock: lift it out of its list item/wrapper, reset its type, or merge it into the previous block. */
export function joinBackward(state: RtState, dispatch?: RichTextDispatch): boolean {
  const position = state.selection.head;
  if (position.offset !== 0 || !isCollapsed(state.selection)) return false;
  const { schema } = state;
  const entries = textblocks(schema, state.doc);
  const index = entries.findIndex((entry) => samePath(entry.path, position.path));
  const previous = index > 0 ? entries[index - 1] : undefined;
  const item = findWrapper(
    schema,
    state.doc,
    position,
    (type) => roleOf(schema, type) === "listItem",
  );
  const listPath = item?.path.slice(0, -1);
  if (
    item &&
    listPath &&
    (!previous || !samePath(previous.path.slice(0, listPath.length), listPath))
  ) {
    const lifted = liftListItem(schema, state.doc, item.path);
    dispatch?.(
      createTransaction(
        state,
        { doc: lifted.doc, selection: caret(lifted.map(position)) },
        "structure",
      ),
    );
    return true;
  }
  const wrapper = findWrapper(
    schema,
    state.doc,
    position,
    (type) => roleOf(schema, type) === "container",
  );
  if (
    wrapper &&
    (!previous || !samePath(previous.path.slice(0, wrapper.path.length), wrapper.path))
  ) {
    const lifted = liftWrapper(schema, state.doc, wrapper.path);
    dispatch?.(
      createTransaction(
        state,
        { doc: lifted.doc, selection: caret(lifted.map(position)) },
        "structure",
      ),
    );
    return true;
  }
  const block = entries[index]?.node;
  if (block && block.type !== schema.defaultBlock && !previous) {
    const doc = setTextblockType(
      schema,
      state.doc,
      position,
      position,
      schema.defaultBlock,
      schema.nodes[schema.defaultBlock]?.defaults ?? {},
    );
    dispatch?.(createTransaction(state, { doc }, "structure"));
    return true;
  }
  if (!previous) return false;
  const end = { path: previous.path, offset: textblockSize(previous.node) };
  const result = replaceRange(schema, state.doc, end, position, []);
  dispatch?.(
    createTransaction(state, { doc: result.doc, selection: caret(result.position) }, "delete"),
  );
  return true;
}

/** Delete: the selection, the next character/word, or merge the next block into this one. */
export function deleteForward(unit: "character" | "word" = "character"): RichTextCommand {
  return (state, dispatch) => {
    if (!isCollapsed(state.selection)) return deleteSelection(state, dispatch);
    const position = state.selection.head;
    const block = blockAt(state.doc, position.path);
    if (!block) return false;
    const size = textblockSize(block);
    if (position.offset < size) {
      const to = {
        ...position,
        offset: Math.min(size, nextBoundary(plainText(state, position), position.offset, unit)),
      };
      const result = replaceRange(state.schema, state.doc, position, to, []);
      dispatch?.(
        createTransaction(state, { doc: result.doc, selection: caret(result.position) }, "delete"),
      );
      return true;
    }
    const entries = textblocks(state.schema, state.doc);
    const next = entries[entries.findIndex((entry) => samePath(entry.path, position.path)) + 1];
    if (!next) return false;
    const result = replaceRange(
      state.schema,
      state.doc,
      position,
      { path: next.path, offset: 0 },
      [],
    );
    dispatch?.(
      createTransaction(state, { doc: result.doc, selection: caret(result.position) }, "delete"),
    );
    return true;
  };
}

/** Enter: newline in code blocks, leave an empty list item, or split the block. */
export function splitBlock(state: RtState, dispatch?: RichTextDispatch): boolean {
  const [from, to] = selectionRange(state.selection);
  const { schema } = state;
  if (isCode(state, from)) return insertText("\n")(state, dispatch);
  let doc: RtDoc = state.doc;
  let position = from;
  if (comparePositions(from, to) !== 0) {
    const cleared = replaceRange(schema, doc, from, to, []);
    doc = cleared.doc;
    position = cleared.position;
  }
  const block = blockAt(doc, position.path);
  const item = findWrapper(schema, doc, position, (type) => roleOf(schema, type) === "listItem");
  if (
    block &&
    item &&
    textblockSize(block) === 0 &&
    samePath(position.path.slice(0, -1), item.path)
  ) {
    const lifted = liftListItem(schema, doc, item.path);
    dispatch?.(
      createTransaction(
        state,
        { doc: lifted.doc, selection: caret(lifted.map(position)) },
        "structure",
      ),
    );
    return true;
  }
  const result = splitTextblock(schema, doc, position);
  dispatch?.(
    createTransaction(state, { doc: result.doc, selection: caret(result.position) }, "structure"),
  );
  return true;
}

/** Insert a document fragment (pasted content) at the selection. */
export function insertFragment(fragment: RtDoc): RichTextCommand {
  return (state, dispatch) => {
    const { schema } = state;
    const [from, to] = selectionRange(state.selection);
    const blocks = fragment.content;
    const first = blocks[0];
    if (!first) return false;
    if (isCode(state, from)) {
      return replaceSelection(
        state,
        [{ type: "text", text: textContent(fragment), marks: [] }],
        "paste",
        dispatch,
      );
    }
    const isTextblock = (block: RtBlock): boolean => roleOf(schema, block.type) === "textblock";
    if (blocks.length === 1 && isTextblock(first)) {
      return replaceSelection(state, inlineContent(first), "paste", dispatch);
    }
    const cleared = replaceRange(schema, state.doc, from, to, []);
    const split = splitTextblock(schema, cleared.doc, cleared.position);
    const firstPath = cleared.position.path;
    const head = isTextblock(first) ? inlineContent(first) : [];
    const last = blocks.at(-1);
    const tail = blocks.length > 1 && last && isTextblock(last) ? inlineContent(last) : null;
    const middle = blocks.slice(isTextblock(first) ? 1 : 0, tail ? -1 : undefined);
    let doc = updateAt(split.doc, firstPath, (node) => [
      { ...node, content: [...inlineContent(node), ...head] },
      ...middle,
    ]);
    const secondPath = [...split.position.path];
    if (samePath(secondPath.slice(0, -1), firstPath.slice(0, -1))) {
      secondPath[secondPath.length - 1] = (secondPath.at(-1) ?? 0) + middle.length;
    }
    let caretAt: RichTextPosition = { path: secondPath, offset: 0 };
    if (tail) {
      doc = updateAt(doc, secondPath, (node) => [
        { ...node, content: [...tail, ...inlineContent(node)] },
      ]);
      caretAt = {
        path: secondPath,
        offset: tail.reduce((total, node) => total + (isText(node) ? node.text.length : 1), 0),
      };
    }
    const normalized = normalizeDoc(schema, doc);
    dispatch?.(createTransaction(state, { doc: normalized, selection: caret(caretAt) }, "paste"));
    return true;
  };
}

/** Select the whole document. */
export function selectAll(state: RtState, dispatch?: RichTextDispatch): boolean {
  const entries = textblocks(state.schema, state.doc);
  const first = entries[0];
  const last = entries.at(-1);
  if (!first || !last) return false;
  const selection: RichTextSelection = {
    anchor: { path: first.path, offset: 0 },
    head: { path: last.path, offset: textblockSize(last.node) },
  };
  dispatch?.(createTransaction(state, { selection }, "selection"));
  return true;
}

/** Whether the selection (or stored marks at a caret) carries a mark type. */
export function isMarkActive(state: RtState, type: string): boolean {
  const [from, to] = selectionRange(state.selection);
  if (comparePositions(from, to) === 0)
    return activeMarks(state).some((mark) => mark.type === type);
  return rangeHasMark(state.schema, state.doc, from, to, type);
}

/** Whether every selected textblock has `type` (and matching `attrs`). */
export function isBlockActive(state: RtState, type: string, attrs: RichTextAttrs = {}): boolean {
  const [from, to] = selectionRange(state.selection);
  const entries = textblocks(state.schema, state.doc);
  const start = entries.findIndex((entry) => samePath(entry.path, from.path));
  const end = entries.findIndex((entry) => samePath(entry.path, to.path));
  return entries
    .slice(start, end + 1)
    .every(
      (entry) =>
        entry.node.type === type &&
        Object.entries(attrs).every(([key, value]) => entry.node.attrs[key] === value),
    );
}

/** Whether the selection sits inside a wrapper of `type`. */
export function isWrappedIn(state: RtState, type: string): boolean {
  return (
    findWrapper(
      state.schema,
      state.doc,
      state.selection.head,
      (candidate) => candidate === type,
    ) !== null
  );
}

/** Typed command set bound to one schema. */
export interface RichTextCommands<Schema extends RichTextSchema> {
  readonly insertText: (text: string) => RichTextCommand;
  readonly deleteSelection: RichTextCommand;
  readonly deleteBackward: (unit?: "character" | "word") => RichTextCommand;
  readonly deleteForward: (unit?: "character" | "word") => RichTextCommand;
  readonly splitBlock: RichTextCommand;
  readonly selectAll: RichTextCommand;
  readonly undo: RichTextCommand;
  readonly redo: RichTextCommand;
  /** Add or remove a mark; at a caret it toggles the stored marks for the next input. */
  readonly toggleMark: <Name extends RichTextMarkName<Schema>>(
    name: Name,
    attrs?: AttrsOf<Schema["marks"][Name]["defaults"]>,
  ) => RichTextCommand;
  /** Apply a mark (replacing one of the same type) across the selection. */
  readonly setMark: <Name extends RichTextMarkName<Schema>>(
    name: Name,
    attrs?: AttrsOf<Schema["marks"][Name]["defaults"]>,
  ) => RichTextCommand;
  /** Remove a mark type across the selection. */
  readonly removeMark: (name: RichTextMarkName<Schema>) => RichTextCommand;
  /** Change the selected textblocks' type. */
  readonly setBlockType: <Name extends RichTextNodeName<Schema, "textblock">>(
    name: Name,
    attrs?: AttrsOf<Schema["nodes"][Name]["defaults"]>,
  ) => RichTextCommand;
  /** Set the block type, or reset to the default block when already active. */
  readonly toggleBlockType: <Name extends RichTextNodeName<Schema, "textblock">>(
    name: Name,
    attrs?: AttrsOf<Schema["nodes"][Name]["defaults"]>,
  ) => RichTextCommand;
  /** Wrap the selected blocks in a container or list. */
  readonly wrapIn: <Name extends RichTextNodeName<Schema, "container" | "list">>(
    name: Name,
    attrs?: AttrsOf<Schema["nodes"][Name]["defaults"]>,
  ) => RichTextCommand;
  /** Wrap, or unwrap when already inside that wrapper type. */
  readonly toggleWrap: <Name extends RichTextNodeName<Schema, "container" | "list">>(
    name: Name,
    attrs?: AttrsOf<Schema["nodes"][Name]["defaults"]>,
  ) => RichTextCommand;
  /** Insert an inline leaf (image, hard break) at the selection. */
  readonly insertInline: <Name extends RichTextNodeName<Schema, "inline">>(
    name: Name,
    attrs?: AttrsOf<Schema["nodes"][Name]["defaults"]>,
  ) => RichTextCommand;
  /** Link the selection to `href` (or insert `text` as a link at a caret); `null` unlinks. */
  readonly setLink: (href: string | null, text?: string) => RichTextCommand;
  /** Insert a document (for example parsed paste content) at the selection. */
  readonly insertContent: (content: RichTextDoc<Schema>) => RichTextCommand;
}

/** Commands addressed by plain type names (used by keymaps and input rules). */
export interface RichTextUntypedCommands {
  readonly insertText: (text: string) => RichTextCommand;
  readonly deleteSelection: RichTextCommand;
  readonly deleteBackward: (unit?: "character" | "word") => RichTextCommand;
  readonly deleteForward: (unit?: "character" | "word") => RichTextCommand;
  readonly splitBlock: RichTextCommand;
  readonly selectAll: RichTextCommand;
  readonly undo: RichTextCommand;
  readonly redo: RichTextCommand;
  readonly toggleMark: (name: string, attrs?: AttrsInput) => RichTextCommand;
  readonly setMark: (name: string, attrs?: AttrsInput) => RichTextCommand;
  readonly removeMark: (name: string) => RichTextCommand;
  readonly setBlockType: (name: string, attrs?: AttrsInput) => RichTextCommand;
  readonly toggleBlockType: (name: string, attrs?: AttrsInput) => RichTextCommand;
  readonly wrapIn: (name: string, attrs?: AttrsInput) => RichTextCommand;
  readonly toggleWrap: (name: string, attrs?: AttrsInput) => RichTextCommand;
  readonly insertInline: (name: string, attrs?: AttrsInput) => RichTextCommand;
  readonly setLink: (href: string | null, text?: string) => RichTextCommand;
  readonly insertContent: (content: RtDoc) => RichTextCommand;
}

/** Create the typed command set for a schema. */
export function createRichTextCommands<Schema extends RichTextSchema>(
  schema: Schema,
): RichTextCommands<Schema> {
  return createUntypedRichTextCommands(schema);
}

/**
 * Commands that take plain type names; they return `false` for names the
 * schema does not declare. Prefer {@link createRichTextCommands} in app code.
 */
export function createUntypedRichTextCommands(schema: RichTextSchema): RichTextUntypedCommands {
  const hasMark = (name: string): boolean => Object.hasOwn(schema.marks, name);
  const hasNode = (name: string, ...roles: string[]): boolean =>
    Object.hasOwn(schema.nodes, name) && roles.includes(schema.nodes[name]?.role ?? "");
  const markDefaults = (name: string): RichTextAttrs => schema.marks[name]?.defaults ?? {};
  const nodeDefaults = (name: string): RichTextAttrs => schema.nodes[name]?.defaults ?? {};
  const linkType = Object.keys(schema.marks).find((name) => name === "link");

  const setMark =
    (name: string, attrs?: AttrsInput): RichTextCommand =>
    (state, dispatch) => {
      const [from, to] = selectionRange(state.selection);
      if (!hasMark(name) || comparePositions(from, to) === 0) return false;
      const mark = { type: name, attrs: withDefaults(markDefaults(name), attrs) };
      const doc = changeMark(state.schema, state.doc, from, to, mark, true);
      dispatch?.(createTransaction(state, { doc }, "format"));
      return true;
    };

  const removeMark =
    (name: string): RichTextCommand =>
    (state, dispatch) => {
      const [from, to] = selectionRange(state.selection);
      if (comparePositions(from, to) === 0) {
        const marks = activeMarks(state);
        if (!marks.some((mark) => mark.type === name)) return false;
        dispatch?.(
          createTransaction(
            state,
            { storedMarks: marks.filter((mark) => mark.type !== name) },
            "format",
          ),
        );
        return true;
      }
      const doc = changeMark(state.schema, state.doc, from, to, { type: name, attrs: {} }, false);
      dispatch?.(createTransaction(state, { doc }, "format"));
      return true;
    };

  const toggleMark =
    (name: string, attrs?: AttrsInput): RichTextCommand =>
    (state, dispatch) => {
      const [from] = selectionRange(state.selection);
      if (!hasMark(name) || isCode(state, from)) return false;
      if (isCollapsed(state.selection)) {
        const marks = activeMarks(state);
        const active = marks.some((mark) => mark.type === name);
        const storedMarks = active
          ? marks.filter((mark) => mark.type !== name)
          : [...marks, { type: name, attrs: withDefaults(markDefaults(name), attrs) }];
        dispatch?.(createTransaction(state, { storedMarks }, "format"));
        return true;
      }
      return isMarkActive(state, name)
        ? removeMark(name)(state, dispatch)
        : setMark(name, attrs)(state, dispatch);
    };

  const setBlockType =
    (name: string, attrs?: AttrsInput): RichTextCommand =>
    (state, dispatch) => {
      const [from, to] = selectionRange(state.selection);
      if (!hasNode(name, "textblock")) return false;
      const resolved = withDefaults(nodeDefaults(name), attrs);
      if (isBlockActive(state, name, resolved)) return false;
      const doc = setTextblockType(state.schema, state.doc, from, to, name, resolved);
      dispatch?.(createTransaction(state, { doc }, "structure"));
      return true;
    };

  const wrapIn =
    (name: string, attrs?: AttrsInput): RichTextCommand =>
    (state, dispatch) => {
      if (!hasNode(name, "container", "list")) return false;
      const [from, to] = selectionRange(state.selection);
      const wrapped = wrapRange(
        state.schema,
        state.doc,
        from,
        to,
        name,
        withDefaults(nodeDefaults(name), attrs),
      );
      dispatch?.(
        createTransaction(
          state,
          {
            doc: wrapped.doc,
            selection: {
              anchor: wrapped.map(state.selection.anchor),
              head: wrapped.map(state.selection.head),
            },
          },
          "structure",
        ),
      );
      return true;
    };

  const unwrap =
    (name: string): RichTextCommand =>
    (state, dispatch) => {
      const wrapper = findWrapper(
        state.schema,
        state.doc,
        state.selection.head,
        (type) => type === name,
      );
      if (!wrapper) return false;
      const lifted = liftWrapper(state.schema, state.doc, wrapper.path);
      dispatch?.(
        createTransaction(
          state,
          {
            doc: lifted.doc,
            selection: {
              anchor: lifted.map(state.selection.anchor),
              head: lifted.map(state.selection.head),
            },
          },
          "structure",
        ),
      );
      return true;
    };

  const insertInline =
    (name: string, attrs?: AttrsInput): RichTextCommand =>
    (state, dispatch) => {
      const [from] = selectionRange(state.selection);
      if (!hasNode(name, "inline") || isCode(state, from)) return false;
      const leaf: RtInline = {
        type: name,
        attrs: withDefaults(nodeDefaults(name), attrs),
        marks: activeMarks(state),
      };
      return replaceSelection(state, [leaf], "input", dispatch);
    };

  const setLink =
    (href: string | null, text?: string): RichTextCommand =>
    (state, dispatch) => {
      if (!linkType) return false;
      if (href === null) return removeMark(linkType)(state, dispatch);
      if (!isCollapsed(state.selection)) return setMark(linkType, { href })(state, dispatch);
      const [from] = selectionRange(state.selection);
      if (!text || isCode(state, from)) return false;
      const marks = [
        ...activeMarks(state).filter((mark) => mark.type !== linkType),
        { type: linkType, attrs: withDefaults(markDefaults(linkType), { href }) },
      ];
      return replaceSelection(state, [{ type: "text", text, marks }], "input", dispatch);
    };

  return {
    insertText,
    deleteSelection,
    deleteBackward,
    deleteForward,
    splitBlock,
    selectAll,
    undo,
    redo,
    toggleMark,
    setMark,
    removeMark,
    setBlockType,
    toggleBlockType: (name, attrs) => (state, dispatch) =>
      isBlockActive(state, name, withDefaults(nodeDefaults(name), attrs)) &&
      name !== schema.defaultBlock
        ? setBlockType(schema.defaultBlock)(state, dispatch)
        : setBlockType(name, attrs)(state, dispatch),
    wrapIn,
    toggleWrap: (name, attrs) => (state, dispatch) =>
      isWrappedIn(state, name)
        ? unwrap(name)(state, dispatch)
        : wrapIn(name, attrs)(state, dispatch),
    insertInline,
    setLink,
    insertContent: insertFragment,
  };
}

/** Text before the caret in its textblock (leaves become U+FFFC). */
export function textBeforeCaret(state: RtState): string {
  const head = state.selection.head;
  const block = blockAt(state.doc, head.path);
  if (!block) return "";
  return sliceInline(inlineContent(block), 0, head.offset)
    .map((node) => (isText(node) ? node.text : "￼"))
    .join("");
}

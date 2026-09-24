import {
  assertRichTextDoc,
  toRtDoc,
  caret,
  comparePositions,
  emptyRichTextDoc,
  textblocks,
} from "./rich-text-model.ts";
import type {
  RichTextDoc,
  RichTextPosition,
  RichTextSelection,
  RtDoc,
  RtMark,
} from "./rich-text-model.ts";
import type { RichTextSchema } from "./rich-text-schema.ts";
import { clampPosition } from "./rich-text-transform.ts";

/** Why a transaction happened; drives undo grouping. */
export type RichTextTransactionKind =
  | "delete"
  | "format"
  | "history"
  | "input"
  | "paste"
  | "selection"
  | "structure";

/** One undo/redo snapshot (validated when restored). */
export interface RichTextHistoryEntry {
  readonly doc: RtDoc;
  readonly selection: RichTextSelection;
}

/** Snapshot history; documents are immutable, so snapshots share structure. */
export interface RichTextHistory {
  readonly done: readonly RichTextHistoryEntry[];
  readonly undone: readonly RichTextHistoryEntry[];
  /** Kind and time of the last recorded change, for grouping typing into one step. */
  readonly last: { readonly kind: RichTextTransactionKind; readonly time: number } | null;
}

/** Immutable editor state typed by its schema. */
export interface RichTextState<Schema extends RichTextSchema = RichTextSchema> {
  readonly schema: Schema;
  readonly doc: RichTextDoc<Schema>;
  readonly selection: RichTextSelection;
  /** Marks applied to the next typed text (after toggling a mark at a caret). */
  readonly storedMarks: readonly RtMark[] | null;
  readonly history: RichTextHistory;
}

/**
 * Schema-agnostic state that commands and transactions work on. Every typed
 * {@link RichTextState} of a concrete schema is assignable to it.
 */
export interface RtState {
  readonly schema: RichTextSchema;
  readonly doc: RtDoc;
  readonly selection: RichTextSelection;
  readonly storedMarks: readonly RtMark[] | null;
  readonly history: RichTextHistory;
}

/** Immutable record of one state change. */
export interface RichTextTransaction {
  readonly before: RtState;
  readonly state: RtState;
  readonly kind: RichTextTransactionKind;
  readonly docChanged: boolean;
}

/** Receives transactions produced by commands. */
export type RichTextDispatch = (transaction: RichTextTransaction) => void;

/**
 * A command checks applicability without `dispatch` and applies itself with it,
 * returning whether it applies. Commands work on the schema-agnostic state
 * view, so one command type serves every schema (typed states are assignable);
 * type safety comes from the typed command factories.
 */
export type RichTextCommand = (state: RtState, dispatch?: RichTextDispatch) => boolean;

/** Options for {@link createRichTextState}. */
export interface RichTextStateOptions<Schema extends RichTextSchema> {
  /** Initial document. @default one empty default block */
  readonly doc?: RichTextDoc<Schema>;
  /** Initial selection. @default caret at the document start */
  readonly selection?: RichTextSelection;
  /** Maximum undo depth. @default 100 */
  readonly historyDepth?: number;
}

/** Options for {@link createTransaction}. */
export interface RichTextTransactionOptions {
  /** Record the change for undo. @default true when the document changed */
  readonly addToHistory?: boolean;
  /** Timestamp used for undo grouping. @default Date.now() */
  readonly time?: number;
  /** Keep typed input within this many milliseconds in one undo step. @default 500 */
  readonly groupDelay?: number;
  /** Maximum undo depth. @default 100 */
  readonly historyDepth?: number;
}

const emptyHistory = Object.freeze({ done: [], undone: [], last: null });

/** Create an editor state; the document is validated against the schema. */
export function createRichTextState<Schema extends RichTextSchema>(
  schema: Schema,
  options: RichTextStateOptions<Schema> = {},
): RichTextState<Schema> {
  const doc = options.doc ? assertRichTextDoc(schema, options.doc) : emptyRichTextDoc(schema);
  const first = textblocks(schema, doc)[0];
  const start: RichTextPosition = { path: first?.path ?? [0], offset: 0 };
  return Object.freeze({
    schema,
    doc,
    selection: options.selection ? clampSelection(schema, doc, options.selection) : caret(start),
    storedMarks: null,
    history: emptyHistory,
  });
}

/** Clamp both ends of a selection into the document. */
export function clampSelection(
  schema: RichTextSchema,
  doc: RtDoc,
  selection: RichTextSelection,
): RichTextSelection {
  return {
    anchor: clampPosition(schema, doc, selection.anchor),
    head: clampPosition(schema, doc, selection.head),
  };
}

function record(
  before: RtState,
  kind: RichTextTransactionKind,
  options: RichTextTransactionOptions,
): RichTextHistory {
  const time = options.time ?? Date.now();
  const history = before.history;
  const grouped =
    kind === "input" &&
    history.last?.kind === "input" &&
    time - history.last.time <= (options.groupDelay ?? 500) &&
    history.done.length > 0;
  const done = grouped
    ? history.done
    : [...history.done, { doc: before.doc, selection: before.selection }].slice(
        -(options.historyDepth ?? 100),
      );
  return { done, undone: [], last: { kind, time } };
}

/**
 * Build a transaction from a state and the changed parts. The resulting
 * document is re-validated, so a transaction can never produce a document the
 * schema rejects.
 */
export function createTransaction(
  before: RtState,
  change: {
    readonly doc?: RtDoc;
    readonly selection?: RichTextSelection;
    readonly storedMarks?: readonly RtMark[] | null;
    readonly history?: RichTextHistory;
  },
  kind: RichTextTransactionKind,
  options: RichTextTransactionOptions = {},
): RichTextTransaction {
  const docChanged = change.doc !== undefined && change.doc !== before.doc;
  const doc = docChanged && change.doc ? toRtDoc(before.schema, change.doc) : before.doc;
  const selection = clampSelection(before.schema, doc, change.selection ?? before.selection);
  const selectionMoved =
    comparePositions(selection.anchor, before.selection.anchor) !== 0 ||
    comparePositions(selection.head, before.selection.head) !== 0;
  const storedMarks =
    change.storedMarks !== undefined
      ? change.storedMarks
      : docChanged || selectionMoved
        ? null
        : before.storedMarks;
  const history =
    change.history ??
    (docChanged && (options.addToHistory ?? true) ? record(before, kind, options) : before.history);
  const state: RtState = Object.freeze({
    schema: before.schema,
    doc,
    selection,
    storedMarks,
    history,
  });
  return Object.freeze({ before, state, kind, docChanged });
}

/** Re-type a state produced by commands for its schema (the document is re-validated, cached). */
export function toRichTextState<Schema extends RichTextSchema>(
  schema: Schema,
  state: RtState,
): RichTextState<Schema> {
  return Object.freeze({
    schema,
    doc: assertRichTextDoc(schema, state.doc),
    selection: state.selection,
    storedMarks: state.storedMarks,
    history: state.history,
  });
}

/** Erase a typed state for schema-agnostic code (the document is re-validated, cached). */
export function toRtState<Schema extends RichTextSchema>(state: RichTextState<Schema>): RtState {
  return {
    schema: state.schema,
    doc: toRtDoc(state.schema, state.doc),
    selection: state.selection,
    storedMarks: state.storedMarks,
    history: state.history,
  };
}

/** Undo the last recorded change. */
export function undo(state: RtState, dispatch?: RichTextDispatch): boolean {
  const entry = state.history.done.at(-1);
  if (!entry) return false;
  dispatch?.(
    createTransaction(
      state,
      {
        doc: entry.doc,
        selection: entry.selection,
        history: {
          done: state.history.done.slice(0, -1),
          undone: [...state.history.undone, { doc: state.doc, selection: state.selection }],
          last: null,
        },
      },
      "history",
    ),
  );
  return true;
}

/** Redo the last undone change. */
export function redo(state: RtState, dispatch?: RichTextDispatch): boolean {
  const entry = state.history.undone.at(-1);
  if (!entry) return false;
  dispatch?.(
    createTransaction(
      state,
      {
        doc: entry.doc,
        selection: entry.selection,
        history: {
          done: [...state.history.done, { doc: state.doc, selection: state.selection }],
          undone: state.history.undone.slice(0, -1),
          last: null,
        },
      },
      "history",
    ),
  );
  return true;
}

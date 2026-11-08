import { graphemeLength, graphemes, sliceGraphemes } from "../utils/text.js";

export interface TextSelection {
  readonly anchor: number;
  readonly focus: number;
}

interface TextSnapshot {
  readonly value: string;
  readonly selection: TextSelection;
}

export interface TextEditingState extends TextSnapshot {
  readonly undo: readonly TextSnapshot[];
  readonly redo: readonly TextSnapshot[];
}

const HISTORY_LIMIT = 100;

function clampPosition(value: string, position: number): number {
  return Math.max(0, Math.min(Math.trunc(position), graphemeLength(value)));
}

function snapshot(state: TextEditingState): TextSnapshot {
  return { value: state.value, selection: state.selection };
}

function normalizeSelection(value: string, selection: TextSelection): TextSelection {
  return {
    anchor: clampPosition(value, selection.anchor),
    focus: clampPosition(value, selection.focus),
  };
}

function withSnapshot(
  state: TextEditingState,
  value: string,
  selection: TextSelection,
): TextEditingState {
  const undo = [...state.undo, snapshot(state)].slice(-HISTORY_LIMIT);
  return { value, selection: normalizeSelection(value, selection), undo, redo: [] };
}

function isWordGrapheme(value: string): boolean {
  return /[\p{L}\p{N}_]/u.test(value);
}

function isWhitespace(value: string): boolean {
  return /^\s+$/u.test(value);
}

export function createTextEditingState(
  value: string,
  cursor = graphemeLength(value),
): TextEditingState {
  const position = clampPosition(value, cursor);
  return { value, selection: { anchor: position, focus: position }, undo: [], redo: [] };
}

export function selectionBounds(selection: TextSelection): readonly [number, number] {
  return selection.anchor <= selection.focus
    ? [selection.anchor, selection.focus]
    : [selection.focus, selection.anchor];
}

export function hasSelection(state: TextEditingState): boolean {
  return state.selection.anchor !== state.selection.focus;
}

export function setTextSelection(
  state: TextEditingState,
  focus: number,
  extend = false,
): TextEditingState {
  const nextFocus = clampPosition(state.value, focus);
  return {
    ...state,
    selection: {
      anchor: extend ? state.selection.anchor : nextFocus,
      focus: nextFocus,
    },
  };
}

export function selectAllText(state: TextEditingState): TextEditingState {
  return {
    ...state,
    selection: { anchor: 0, focus: graphemeLength(state.value) },
  };
}

export function previousWordBoundary(value: string, position: number): number {
  const parts = graphemes(value);
  let cursor = Math.max(0, Math.min(position, parts.length));
  while (cursor > 0 && isWhitespace(parts[cursor - 1]!)) cursor--;
  if (cursor > 0 && isWordGrapheme(parts[cursor - 1]!)) {
    while (cursor > 0 && isWordGrapheme(parts[cursor - 1]!)) cursor--;
  } else {
    while (cursor > 0 && !isWhitespace(parts[cursor - 1]!) && !isWordGrapheme(parts[cursor - 1]!)) {
      cursor--;
    }
  }
  return cursor;
}

export function nextWordBoundary(value: string, position: number): number {
  const parts = graphemes(value);
  let cursor = Math.max(0, Math.min(position, parts.length));
  if (cursor < parts.length && isWordGrapheme(parts[cursor]!)) {
    while (cursor < parts.length && isWordGrapheme(parts[cursor]!)) cursor++;
  } else if (cursor < parts.length && !isWhitespace(parts[cursor]!)) {
    while (
      cursor < parts.length &&
      !isWhitespace(parts[cursor]!) &&
      !isWordGrapheme(parts[cursor]!)
    ) {
      cursor++;
    }
  }
  while (cursor < parts.length && isWhitespace(parts[cursor]!)) cursor++;
  return cursor;
}

export function replaceTextSelection(state: TextEditingState, text: string): TextEditingState {
  const [start, end] = selectionBounds(state.selection);
  const value = `${sliceGraphemes(state.value, 0, start)}${text}${sliceGraphemes(state.value, end)}`;
  const cursor = start + graphemeLength(text);
  if (value === state.value && start === end) return state;
  return withSnapshot(state, value, { anchor: cursor, focus: cursor });
}

export function deleteTextBackward(state: TextEditingState, byWord = false): TextEditingState {
  if (hasSelection(state)) return replaceTextSelection(state, "");
  const end = state.selection.focus;
  const start = byWord ? previousWordBoundary(state.value, end) : Math.max(0, end - 1);
  if (start === end) return state;
  return replaceTextSelection({ ...state, selection: { anchor: start, focus: end } }, "");
}

export function deleteTextForward(state: TextEditingState, byWord = false): TextEditingState {
  if (hasSelection(state)) return replaceTextSelection(state, "");
  const start = state.selection.focus;
  const end = byWord
    ? nextWordBoundary(state.value, start)
    : Math.min(graphemeLength(state.value), start + 1);
  if (start === end) return state;
  return replaceTextSelection({ ...state, selection: { anchor: start, focus: end } }, "");
}

export function undoTextEdit(state: TextEditingState): TextEditingState {
  const previous = state.undo.at(-1);
  if (!previous) return state;
  return {
    ...previous,
    undo: state.undo.slice(0, -1),
    redo: [...state.redo, snapshot(state)].slice(-HISTORY_LIMIT),
  };
}

export function redoTextEdit(state: TextEditingState): TextEditingState {
  const next = state.redo.at(-1);
  if (!next) return state;
  return {
    ...next,
    undo: [...state.undo, snapshot(state)].slice(-HISTORY_LIMIT),
    redo: state.redo.slice(0, -1),
  };
}

export function replaceExternalText(state: TextEditingState, value: string): TextEditingState {
  if (value === state.value) return state;
  return {
    value,
    selection: normalizeSelection(value, state.selection),
    undo: [],
    redo: [],
  };
}

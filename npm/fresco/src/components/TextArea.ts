/**
 * TextArea Component - controlled or uncontrolled multiline text editing.
 */

import { computed, defineComponent, h, ref, shallowRef, watch, type PropType } from "vue";
import { useInput } from "../composables/useInput.js";
import {
  deleteGraphemeAt,
  deleteGraphemeBefore,
  graphemeLength,
  sliceGraphemes,
} from "../utils/text.js";
import {
  createTextEditingState,
  deleteTextBackward,
  deleteTextForward,
  hasSelection,
  nextWordBoundary,
  previousWordBoundary,
  redoTextEdit,
  replaceExternalText,
  replaceTextSelection,
  selectAllText,
  selectionBounds,
  setTextSelection,
  undoTextEdit,
  type TextEditingState,
} from "./text-editing.js";
import {
  lineBoundary,
  moveTextAreaCursorVertically,
  positionToTextAreaPoint,
  textAreaPointToPosition,
} from "./textarea-editing.js";

export interface TextAreaProps {
  /** Controlled text value. */
  modelValue?: string;
  /** Initial value used when modelValue is omitted. */
  defaultValue?: string;
  /** Placeholder text. */
  placeholder?: string;
  /** Number of retained visible rows. */
  rows?: number;
  /** Whether the textarea owns keyboard input. */
  focused?: boolean;
  /** Prevent editing and render dimmed text. */
  disabled?: boolean;
  /** Show one-based line numbers. */
  lineNumbers?: boolean;
  /** Border style. */
  border?: "single" | "double" | "rounded" | "none";
  /** Foreground color. */
  fg?: string;
  /** Placeholder foreground color. */
  placeholderFg?: string;
  /** Line number foreground color. */
  lineNumberFg?: string;
  /** Controlled cursor line, zero based. */
  cursorLine?: number;
  /** Controlled cursor column, in graphemes. */
  cursorColumn?: number;
  /** Glyph rendered at the focused cursor. */
  cursor?: string;
  /** Validate the current value; a string becomes the error message. */
  validate?: (value: string) => boolean | string | null | undefined;
}

export const TextArea = defineComponent({
  name: "TextArea",
  props: {
    modelValue: { type: String, default: undefined },
    defaultValue: { type: String, default: "" },
    placeholder: String,
    rows: { type: Number, default: 5 },
    focused: { type: Boolean, default: false },
    disabled: { type: Boolean, default: false },
    lineNumbers: { type: Boolean, default: false },
    border: {
      type: String as PropType<TextAreaProps["border"]>,
      default: "single",
    },
    fg: String,
    placeholderFg: { type: String, default: "gray" },
    lineNumberFg: { type: String, default: "gray" },
    cursorLine: Number,
    cursorColumn: Number,
    cursor: { type: String, default: "│" },
    validate: Function as PropType<TextAreaProps["validate"]>,
  },
  emits: [
    "update:modelValue",
    "update:cursorLine",
    "update:cursorColumn",
    "submit",
    "cancel",
    "selectionChange",
    "copy",
    "cut",
    "validationChange",
    "compositionstart",
    "compositionupdate",
    "compositionend",
  ],
  setup(props, { emit }) {
    const initialValue = props.modelValue ?? props.defaultValue;
    const initialCursor =
      props.cursorLine !== undefined || props.cursorColumn !== undefined
        ? textAreaPointToPosition(initialValue, props.cursorLine ?? 0, props.cursorColumn ?? 0)
        : graphemeLength(initialValue);
    const editing = shallowRef(createTextEditingState(initialValue, initialCursor));
    const internalValue = computed(() => editing.value.value);
    const cursorPosition = computed(() => editing.value.selection.focus);
    const isActive = computed(() => props.focused && !props.disabled);
    const isComposing = ref(false);
    const compositionAnchor = ref(cursorPosition.value);
    const preedit = ref("");
    const preeditCursor = ref(0);
    const preferredColumn = ref(positionToTextAreaPoint(initialValue, initialCursor).column);
    const viewportStart = ref(0);

    const rowCount = computed(() => {
      const rows = Math.floor(props.rows);
      return Number.isSafeInteger(rows) && rows > 0 ? rows : 1;
    });
    const validation = computed(() => {
      const result = props.validate?.(internalValue.value);
      return {
        valid: result !== false && typeof result !== "string",
        message: typeof result === "string" ? result : undefined,
      };
    });

    watch(validation, ({ valid, message }) => emit("validationChange", valid, message), {
      immediate: true,
    });
    watch(
      () => props.modelValue,
      (next) => {
        if (next !== undefined) editing.value = replaceExternalText(editing.value, next);
      },
    );
    watch([() => props.cursorLine, () => props.cursorColumn], ([line, column]) => {
      if (line === undefined && column === undefined) return;
      const current = positionToTextAreaPoint(internalValue.value, cursorPosition.value);
      editing.value = setTextSelection(
        editing.value,
        textAreaPointToPosition(
          internalValue.value,
          line ?? current.line,
          column ?? current.column,
        ),
      );
    });

    const displayValue = computed(() => {
      if (!isComposing.value) return internalValue.value;
      return [
        sliceGraphemes(internalValue.value, 0, compositionAnchor.value),
        preedit.value,
        sliceGraphemes(internalValue.value, compositionAnchor.value),
      ].join("");
    });
    const displayCursor = computed(() =>
      isComposing.value
        ? compositionAnchor.value + preeditCursor.value
        : editing.value.selection.focus,
    );
    const cursorPoint = computed(() =>
      positionToTextAreaPoint(displayValue.value, displayCursor.value),
    );
    const allLines = computed(() => displayValue.value.split("\n"));

    watch(
      [() => cursorPoint.value.line, rowCount],
      ([line, rows]) => {
        if (line < viewportStart.value) viewportStart.value = line;
        else if (line >= viewportStart.value + rows) viewportStart.value = line - rows + 1;
      },
      { immediate: true },
    );

    const visibleLines = computed(() =>
      Array.from({ length: rowCount.value }, (_, offset) => {
        const line = viewportStart.value + offset;
        return { line, content: allLines.value[line] ?? "" };
      }),
    );

    const applyEditing = (next: TextEditingState, preservePreferredColumn = false) => {
      const previous = editing.value;
      if (next === previous) return;
      editing.value = next;
      if (next.value !== previous.value) emit("update:modelValue", next.value);
      if (
        next.selection.anchor !== previous.selection.anchor ||
        next.selection.focus !== previous.selection.focus
      ) {
        const [start, end] = selectionBounds(next.selection);
        const point = positionToTextAreaPoint(next.value, next.selection.focus);
        emit("selectionChange", start, end);
        emit("update:cursorLine", point.line);
        emit("update:cursorColumn", point.column);
        if (!preservePreferredColumn) preferredColumn.value = point.column;
      }
    };

    const insertText = (text: string) => applyEditing(replaceTextSelection(editing.value, text));
    const moveHorizontal = (delta: -1 | 1, extend: boolean, byWord: boolean) => {
      const [start, end] = selectionBounds(editing.value.selection);
      const current = editing.value.selection.focus;
      const target =
        hasSelection(editing.value) && !extend
          ? delta < 0
            ? start
            : end
          : byWord
            ? delta < 0
              ? previousWordBoundary(internalValue.value, current)
              : nextWordBoundary(internalValue.value, current)
            : current + delta;
      applyEditing(setTextSelection(editing.value, target, extend));
    };
    const moveVertical = (lines: number, extend: boolean) => {
      applyEditing(
        setTextSelection(
          editing.value,
          moveTextAreaCursorVertically(
            internalValue.value,
            editing.value.selection.focus,
            lines,
            preferredColumn.value,
          ),
          extend,
        ),
        true,
      );
    };

    const updatePreedit = (text: string, cursor = graphemeLength(text)) => {
      if (!isComposing.value) {
        isComposing.value = true;
        compositionAnchor.value = cursorPosition.value;
        emit("compositionstart");
      }
      preedit.value = text;
      preeditCursor.value = Math.max(0, Math.min(cursor, graphemeLength(text)));
      emit("compositionupdate", preedit.value, preeditCursor.value);
    };
    const finishComposition = (text = preedit.value) => {
      if (!isComposing.value) return;
      editing.value = setTextSelection(editing.value, compositionAnchor.value);
      isComposing.value = false;
      preedit.value = "";
      preeditCursor.value = 0;
      if (text) insertText(text);
      emit("compositionend", text);
    };
    const cancelComposition = () => {
      if (!isComposing.value) return false;
      isComposing.value = false;
      preedit.value = "";
      preeditCursor.value = 0;
      editing.value = setTextSelection(editing.value, compositionAnchor.value);
      emit("compositionend", "");
      return true;
    };
    const movePreedit = (delta: -1 | 1) => {
      if (!isComposing.value) return false;
      const next = preeditCursor.value + delta;
      if (next < 0 || next > graphemeLength(preedit.value)) return true;
      preeditCursor.value = next;
      emit("compositionupdate", preedit.value, next);
      return true;
    };
    const deletePreedit = (forward: boolean) => {
      if (!isComposing.value) return false;
      if (forward) {
        preedit.value = deleteGraphemeAt(preedit.value, preeditCursor.value);
      } else if (preeditCursor.value > 0) {
        preedit.value = deleteGraphemeBefore(preedit.value, preeditCursor.value);
        preeditCursor.value--;
      }
      emit("compositionupdate", preedit.value, preeditCursor.value);
      return true;
    };

    useInput({
      isActive,
      captureCtrlC: true,
      handler: (_input, key) => {
        if (!key.return) return;
        if (isComposing.value) {
          finishComposition();
        } else if (key.ctrl || key.meta) {
          emit("submit", internalValue.value);
        } else {
          insertText("\n");
        }
      },
      onChar: (character) => {
        if (!isComposing.value) insertText(character);
      },
      onKey: (key, modifiers) => {
        const byWord = modifiers.ctrl || modifiers.alt;
        if (key === "left") {
          if (!movePreedit(-1)) moveHorizontal(-1, modifiers.shift, byWord);
        } else if (key === "right") {
          if (!movePreedit(1)) moveHorizontal(1, modifiers.shift, byWord);
        } else if (key === "up") {
          if (!isComposing.value) moveVertical(-1, modifiers.shift);
        } else if (key === "down") {
          if (!isComposing.value) moveVertical(1, modifiers.shift);
        } else if (key === "pageup") {
          if (!isComposing.value) moveVertical(-rowCount.value, modifiers.shift);
        } else if (key === "pagedown") {
          if (!isComposing.value) moveVertical(rowCount.value, modifiers.shift);
        } else if (key === "backspace") {
          if (!deletePreedit(false)) applyEditing(deleteTextBackward(editing.value, byWord));
        } else if (key === "delete") {
          if (!deletePreedit(true)) applyEditing(deleteTextForward(editing.value, byWord));
        } else if (key === "home") {
          const target = modifiers.ctrl
            ? 0
            : lineBoundary(internalValue.value, cursorPosition.value, "start");
          applyEditing(setTextSelection(editing.value, target, modifiers.shift));
        } else if (key === "end") {
          const target = modifiers.ctrl
            ? graphemeLength(internalValue.value)
            : lineBoundary(internalValue.value, cursorPosition.value, "end");
          applyEditing(setTextSelection(editing.value, target, modifiers.shift));
        } else if (key === "tab") {
          insertText("\t");
        } else if (key === "a" && modifiers.ctrl) {
          applyEditing(selectAllText(editing.value));
        } else if (key === "z" && modifiers.ctrl) {
          applyEditing(modifiers.shift ? redoTextEdit(editing.value) : undoTextEdit(editing.value));
        } else if (key === "y" && modifiers.ctrl) {
          applyEditing(redoTextEdit(editing.value));
        } else if ((key === "c" || key === "x") && (modifiers.ctrl || modifiers.meta)) {
          if (!hasSelection(editing.value)) return;
          const [start, end] = selectionBounds(editing.value.selection);
          const selected = sliceGraphemes(internalValue.value, start, end);
          emit(key === "c" ? "copy" : "cut", selected);
          if (key === "x") applyEditing(replaceTextSelection(editing.value, ""));
        }
      },
      onEscape: () => {
        if (!cancelComposition()) emit("cancel");
      },
      onCompositionStart: () => updatePreedit("", 0),
      onCompositionUpdate: updatePreedit,
      onCompositionEnd: finishComposition,
    });

    return () => {
      const lineNumberWidth = String(Math.max(allLines.value.length, 1)).length;
      const showPlaceholder = !displayValue.value && props.placeholder && !props.focused;
      const children = visibleLines.value.map(({ line, content }) => {
        const isCursorLine = props.focused && line === cursorPoint.value.line;
        const parts = [];

        if (props.lineNumbers) {
          parts.push(
            h("text", {
              key: `ln-${line}`,
              text: `${String(line + 1).padStart(lineNumberWidth)} │ `,
              fg: props.lineNumberFg,
              dim: !isCursorLine,
            }),
          );
        }

        const placeholder = showPlaceholder && line === 0;
        const renderedContent = placeholder
          ? props.placeholder
          : isCursorLine
            ? `${sliceGraphemes(content, 0, cursorPoint.value.column)}${props.cursor}${sliceGraphemes(content, cursorPoint.value.column)}`
            : content || " ";
        parts.push(
          h("text", {
            key: `content-${line}`,
            text: renderedContent,
            fg: placeholder ? props.placeholderFg : props.fg,
            dim: props.disabled,
            bold: isCursorLine,
          }),
        );

        return h(
          "box",
          {
            key: `line-${line}`,
            style: { flexDirection: "row" },
            bg: isCursorLine ? "gray" : undefined,
          },
          parts,
        );
      });

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          invalid: !validation.value.valid,
          validationMessage: validation.value.message,
          style: {
            flexDirection: "column",
            padding: props.border !== "none" ? 1 : 0,
            height: String(rowCount.value + (props.border !== "none" ? 2 : 0)),
          },
        },
        children,
      );
    };
  },
});

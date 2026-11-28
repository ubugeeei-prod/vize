/* eslint-disable script/one-component-per-file -- this module defines more than one component */
/**
 * TextInput Component - Text input with builtin cursor management and IME support
 */

import { computed, defineComponent, h, ref, shallowRef, watch, type PropType } from "vue";
import { useInput } from "../composables/useInput.js";
import type { FrescoDimension, FrescoRenderStyle } from "../protocol.js";
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

/** Controlled or initial single-line text, focus, validation, and editing events. */
export interface TextInputProps {
  /** Text used initially and synchronized when a defined v-model value changes.
   * @default undefined
   */
  modelValue?: string;
  /** Initial text when modelValue is omitted; later changes do not reset editing.
   * @default ""
   */
  defaultValue?: string;
  /** Placeholder text.
   * @default ""
   */
  placeholder?: string;
  /** Whether input is focused when the focused alias is omitted.
   * @default false
   */
  focus?: boolean;
  /** Focus alias; a defined value takes precedence over focus, including false.
   * @default undefined
   */
  focused?: boolean;
  /** Password mode that masks displayed input.
   * @default false
   */
  mask?: boolean;
  /** Character used to mask input.
   * @default "*"
   */
  maskChar?: string;
  /** Input width.
   * @default undefined
   */
  width?: FrescoDimension;
  /** Foreground color.
   * @default undefined
   */
  fg?: string;
  /** Background color.
   * @default undefined
   */
  bg?: string;
  /** Validate text; false or any string is invalid, and a string is its message.
   * @default undefined
   */
  validate?: (value: string) => boolean | string | null | undefined;
  /** Called when local editing changes text.
   * @default undefined
   */
  "onUpdate:modelValue"?: (value: string) => void;
  /** Called on Enter when no IME composition is active.
   * @default undefined
   */
  onSubmit?: (value: string) => void;
  /** Called on Escape when no IME composition is active.
   * @default undefined
   */
  onCancel?: () => void;
  /** Called with sorted grapheme bounds when the selection changes.
   * @default undefined
   */
  onSelectionChange?: (start: number, end: number) => void;
  /** Called with copied selection text; does not itself write a system clipboard.
   * @default undefined
   */
  onCopy?: (value: string) => void;
  /** Called with cut selection text; does not itself write a system clipboard.
   * @default undefined
   */
  onCut?: (value: string) => void;
  /** Called immediately and whenever the current validation result changes.
   * @default undefined
   */
  onValidationChange?: (valid: boolean, message?: string) => void;
  /** Listener for the start of IME composition.
   * @default undefined
   */
  onCompositionStart?: () => void;
  /** Listener for IME composition updates.
   * @default undefined
   */
  onCompositionUpdate?: (text: string, cursor: number) => void;
  /** Listener for the end of IME composition.
   * @default undefined
   */
  onCompositionEnd?: (text: string) => void;
}

/** Edit single-line terminal text with grapheme-aware selection and IME support. */
export const TextInput = defineComponent({
  name: "TextInput",
  props: {
    modelValue: {
      type: String,
      default: undefined,
    },
    defaultValue: {
      type: String,
      default: "",
    },
    placeholder: {
      type: String,
      default: "",
    },
    focus: {
      type: Boolean,
      default: false,
    },
    focused: {
      type: Boolean,
      default: undefined,
    },
    mask: {
      type: Boolean,
      default: false,
    },
    maskChar: {
      type: String,
      default: "*",
    },
    width: [Number, String] as PropType<FrescoDimension>,
    fg: String,
    bg: String,
    validate: Function as PropType<TextInputProps["validate"]>,
  },
  emits: [
    "update:modelValue",
    "submit",
    "cancel",
    "compositionstart",
    "compositionupdate",
    "compositionend",
    "selectionChange",
    "copy",
    "cut",
    "validationChange",
  ],
  setup(props, { emit }) {
    const initialValue = props.modelValue ?? props.defaultValue;
    const editing = shallowRef(createTextEditingState(initialValue));
    const internalValue = computed(() => editing.value.value);
    const cursorPos = computed(() => editing.value.selection.focus);
    const isComposing = ref(false);
    const compositionAnchor = ref(cursorPos.value);
    const preedit = ref("");
    const preeditCursor = ref(0);

    const focused = computed(() => props.focused ?? props.focus);
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

    // Sync with v-model
    watch(
      () => props.modelValue,
      (newValue) => {
        if (newValue === undefined) return;
        editing.value = replaceExternalText(editing.value, newValue);
      },
    );

    const applyEditing = (next: TextEditingState) => {
      const previous = editing.value;
      if (next === previous) return;
      editing.value = next;
      if (next.value !== previous.value) emit("update:modelValue", next.value);
      if (
        next.selection.anchor !== previous.selection.anchor ||
        next.selection.focus !== previous.selection.focus
      ) {
        emit("selectionChange", ...selectionBounds(next.selection));
      }
    };

    // Insert text at cursor position
    const insertText = (text: string) => {
      applyEditing(replaceTextSelection(editing.value, text));
    };

    // Delete character before cursor (Backspace)
    const deleteBack = (byWord = false) => {
      applyEditing(deleteTextBackward(editing.value, byWord));
    };

    // Delete character at cursor (Delete)
    const deleteForward = (byWord = false) => {
      applyEditing(deleteTextForward(editing.value, byWord));
    };

    const moveLeft = (extend = false, byWord = false) => {
      const [start] = selectionBounds(editing.value.selection);
      const target =
        hasSelection(editing.value) && !extend
          ? start
          : byWord
            ? previousWordBoundary(internalValue.value, cursorPos.value)
            : cursorPos.value - 1;
      applyEditing(setTextSelection(editing.value, target, extend));
    };

    const moveRight = (extend = false, byWord = false) => {
      const [, end] = selectionBounds(editing.value.selection);
      const target =
        hasSelection(editing.value) && !extend
          ? end
          : byWord
            ? nextWordBoundary(internalValue.value, cursorPos.value)
            : cursorPos.value + 1;
      applyEditing(setTextSelection(editing.value, target, extend));
    };

    // Move cursor to start
    const moveToStart = (extend = false) => {
      applyEditing(setTextSelection(editing.value, 0, extend));
    };

    // Move cursor to end
    const moveToEnd = (extend = false) => {
      applyEditing(setTextSelection(editing.value, graphemeLength(internalValue.value), extend));
    };

    const updatePreedit = (text: string, cursor = graphemeLength(text)) => {
      if (!isComposing.value) {
        isComposing.value = true;
        compositionAnchor.value = cursorPos.value;
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
      if (text) {
        insertText(text);
      }
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

    const deletePreeditBack = () => {
      if (!isComposing.value || preeditCursor.value === 0) return false;

      preedit.value = deleteGraphemeBefore(preedit.value, preeditCursor.value);
      preeditCursor.value--;
      emit("compositionupdate", preedit.value, preeditCursor.value);
      return true;
    };

    const deletePreeditForward = () => {
      if (!isComposing.value || preeditCursor.value >= graphemeLength(preedit.value)) {
        return false;
      }

      preedit.value = deleteGraphemeAt(preedit.value, preeditCursor.value);
      emit("compositionupdate", preedit.value, preeditCursor.value);
      return true;
    };

    const movePreeditLeft = () => {
      if (!isComposing.value || preeditCursor.value === 0) return false;

      preeditCursor.value--;
      emit("compositionupdate", preedit.value, preeditCursor.value);
      return true;
    };

    const movePreeditRight = () => {
      if (!isComposing.value || preeditCursor.value >= graphemeLength(preedit.value)) {
        return false;
      }

      preeditCursor.value++;
      emit("compositionupdate", preedit.value, preeditCursor.value);
      return true;
    };

    const movePreeditToStart = () => {
      if (!isComposing.value) return false;

      preeditCursor.value = 0;
      emit("compositionupdate", preedit.value, preeditCursor.value);
      return true;
    };

    const movePreeditToEnd = () => {
      if (!isComposing.value) return false;

      preeditCursor.value = graphemeLength(preedit.value);
      emit("compositionupdate", preedit.value, preeditCursor.value);
      return true;
    };

    const displayValue = computed(() => {
      if (!isComposing.value) return internalValue.value;
      return [
        sliceGraphemes(internalValue.value, 0, compositionAnchor.value),
        preedit.value,
        sliceGraphemes(internalValue.value, compositionAnchor.value),
      ].join("");
    });

    const displayCursor = computed(() => {
      if (!isComposing.value) return cursorPos.value;
      return compositionAnchor.value + preeditCursor.value;
    });

    // Use focus prop to control input handling
    const isActive = computed(() => focused.value);

    // Handle keyboard input when focused
    useInput({
      isActive,
      captureCtrlC: true,
      onChar: (char) => {
        if (isComposing.value) return;
        insertText(char);
      },
      onKey: (key, modifiers) => {
        const modifiedWord = modifiers.ctrl || modifiers.alt;
        if (key === "left") {
          if (!movePreeditLeft()) moveLeft(modifiers.shift, modifiedWord);
        } else if (key === "right") {
          if (!movePreeditRight()) moveRight(modifiers.shift, modifiedWord);
        } else if (key === "backspace") {
          if (!deletePreeditBack()) deleteBack(modifiedWord);
        } else if (key === "delete") {
          if (!deletePreeditForward()) deleteForward(modifiedWord);
        } else if (key === "home") {
          if (!movePreeditToStart()) moveToStart(modifiers.shift);
        } else if (key === "end") {
          if (!movePreeditToEnd()) moveToEnd(modifiers.shift);
        } else if (key === "a" && modifiers.ctrl) {
          if (!movePreeditToEnd()) applyEditing(selectAllText(editing.value));
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
      onSubmit: () => {
        if (isComposing.value) {
          finishComposition();
          return;
        }
        emit("submit", internalValue.value);
      },
      onEscape: () => {
        if (cancelComposition()) return;
        emit("cancel");
      },
      onCompositionStart: () => {
        isComposing.value = true;
        compositionAnchor.value = cursorPos.value;
        preedit.value = "";
        preeditCursor.value = 0;
        emit("compositionstart");
      },
      onCompositionUpdate: updatePreedit,
      onCompositionEnd: finishComposition,
    });

    return () => {
      const style: FrescoRenderStyle = {};
      const [selectionStart, selectionEnd] = selectionBounds(editing.value.selection);
      if (props.width !== undefined) {
        style.width = String(props.width);
      }

      return h("input", {
        value: displayValue.value,
        placeholder: props.placeholder,
        focused: focused.value,
        cursor: displayCursor.value,
        selectionStart: isComposing.value ? displayCursor.value : selectionStart,
        selectionEnd: isComposing.value ? displayCursor.value : selectionEnd,
        invalid: !validation.value.valid,
        validationMessage: validation.value.message,
        mask: props.mask,
        maskChar: props.maskChar,
        style,
        fg: props.fg,
        bg: props.bg,
      });
    };
  },
});

/**
 * Password input variant
 */
export const PasswordInput = defineComponent({
  name: "PasswordInput",
  props: {
    modelValue: {
      type: String,
      default: "",
    },
    placeholder: {
      type: String,
      default: "Enter password...",
    },
    focus: Boolean,
    width: [Number, String] as PropType<FrescoDimension>,
    fg: String,
    bg: String,
  },
  emits: ["update:modelValue", "submit", "cancel"],
  setup(props, { emit }) {
    return () =>
      h(TextInput, {
        ...props,
        mask: true,
        "onUpdate:modelValue": (v: string) => emit("update:modelValue", v),
        onSubmit: (v: string) => emit("submit", v),
        onCancel: () => emit("cancel"),
      });
  },
});

/**
 * TextInput Component - Text input with builtin cursor management and IME support
 */

import { computed, defineComponent, h, ref, watch, type PropType } from "@vue/runtime-core";
import { useInput } from "../composables/useInput.js";
import {
  deleteGraphemeAt,
  deleteGraphemeBefore,
  graphemeLength,
  insertAtGrapheme,
  sliceGraphemes,
} from "../utils/text.js";

export interface TextInputProps {
  /** Input value (v-model) */
  modelValue?: string;
  /** Placeholder text */
  placeholder?: string;
  /** Whether input is focused */
  focus?: boolean;
  /** Whether input is focused (Ink-style alias used by native nodes) */
  focused?: boolean;
  /** Password mode (mask input) */
  mask?: boolean;
  /** Mask character */
  maskChar?: string;
  /** Input width */
  width?: number | string;
  /** Foreground color */
  fg?: string;
  /** Background color */
  bg?: string;
  /** Called when value changes */
  "onUpdate:modelValue"?: (value: string) => void;
  /** Called when submitted (Enter) */
  onSubmit?: (value: string) => void;
  /** Called when escape is pressed */
  onCancel?: () => void;
  /** Called when IME composition starts */
  onCompositionStart?: () => void;
  /** Called when IME composition updates */
  onCompositionUpdate?: (text: string, cursor: number) => void;
  /** Called when IME composition ends */
  onCompositionEnd?: (text: string) => void;
}

export const TextInput = defineComponent({
  name: "TextInput",
  props: {
    modelValue: {
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
    width: [Number, String] as PropType<number | string>,
    fg: String,
    bg: String,
  },
  emits: [
    "update:modelValue",
    "submit",
    "cancel",
    "compositionstart",
    "compositionupdate",
    "compositionend",
  ],
  setup(props, { emit }) {
    const internalValue = ref(props.modelValue);
    const cursorPos = ref(graphemeLength(props.modelValue));
    const isComposing = ref(false);
    const compositionAnchor = ref(cursorPos.value);
    const preedit = ref("");
    const preeditCursor = ref(0);

    const focused = computed(() => props.focused ?? props.focus);

    // Sync with v-model
    watch(
      () => props.modelValue,
      (newValue) => {
        internalValue.value = newValue;
        // Keep cursor at end if value changes externally
        const length = graphemeLength(newValue);
        if (cursorPos.value > length) {
          cursorPos.value = length;
        }
      },
    );

    // Update value and emit
    const updateValue = (value: string) => {
      internalValue.value = value;
      emit("update:modelValue", value);
    };

    // Insert text at cursor position
    const insertText = (text: string) => {
      updateValue(insertAtGrapheme(internalValue.value, cursorPos.value, text));
      cursorPos.value += graphemeLength(text);
    };

    // Delete character before cursor (Backspace)
    const deleteBack = () => {
      if (cursorPos.value > 0) {
        updateValue(deleteGraphemeBefore(internalValue.value, cursorPos.value));
        cursorPos.value--;
      }
    };

    // Delete character at cursor (Delete)
    const deleteForward = () => {
      if (cursorPos.value < graphemeLength(internalValue.value)) {
        updateValue(deleteGraphemeAt(internalValue.value, cursorPos.value));
      }
    };

    // Move cursor left
    const moveLeft = () => {
      if (cursorPos.value > 0) {
        cursorPos.value--;
      }
    };

    // Move cursor right
    const moveRight = () => {
      if (cursorPos.value < graphemeLength(internalValue.value)) {
        cursorPos.value++;
      }
    };

    // Move cursor to start
    const moveToStart = () => {
      cursorPos.value = 0;
    };

    // Move cursor to end
    const moveToEnd = () => {
      cursorPos.value = graphemeLength(internalValue.value);
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

      cursorPos.value = compositionAnchor.value;
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
      cursorPos.value = compositionAnchor.value;
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
      onChar: (char) => {
        if (isComposing.value) return;
        insertText(char);
      },
      onArrow: (direction) => {
        if (direction === "left") {
          if (!movePreeditLeft()) moveLeft();
        }
        if (direction === "right") {
          if (!movePreeditRight()) moveRight();
        }
      },
      onKey: (key, modifiers) => {
        if (key === "backspace") {
          if (!deletePreeditBack()) deleteBack();
        } else if (key === "delete") {
          if (!deletePreeditForward()) deleteForward();
        } else if (key === "home") {
          if (!movePreeditToStart()) moveToStart();
        } else if (key === "end") {
          if (!movePreeditToEnd()) moveToEnd();
        } else if (key === "a" && modifiers.ctrl) {
          // Ctrl+A - select all (move to end for now)
          if (!movePreeditToEnd()) moveToEnd();
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
      const style: Record<string, unknown> = {};
      if (props.width !== undefined) {
        style.width = String(props.width);
      }

      return h("input", {
        value: displayValue.value,
        placeholder: props.placeholder,
        focused: focused.value,
        cursor: displayCursor.value,
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
    width: [Number, String] as PropType<number | string>,
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

/**
 * Confirm Component - Confirmation dialog
 */

import { computed, defineComponent, h, ref } from "vue";
import { useInput } from "../composables/useInput.js";

export interface ConfirmProps {
  /** Confirmation message */
  message: string;
  /** Confirm button text */
  confirmText?: string;
  /** Cancel button text */
  cancelText?: string;
  /** Whether confirm is initially selected */
  defaultConfirm?: boolean;
  /** Whether the confirmation owns keyboard input */
  focused?: boolean;
  /** Confirm button foreground color */
  confirmFg?: string;
  /** Cancel button foreground color */
  cancelFg?: string;
  /** Selected button foreground color */
  selectedFg?: string;
}

export const Confirm = defineComponent({
  name: "Confirm",
  props: {
    message: {
      type: String,
      required: true,
    },
    confirmText: {
      type: String,
      default: "Yes",
    },
    cancelText: {
      type: String,
      default: "No",
    },
    defaultConfirm: {
      type: Boolean,
      default: true,
    },
    focused: {
      type: Boolean,
      default: false,
    },
    confirmFg: {
      type: String,
      default: "green",
    },
    cancelFg: {
      type: String,
      default: "red",
    },
    selectedFg: {
      type: String,
      default: "cyan",
    },
  },
  emits: ["confirm", "cancel", "select"],
  setup(props, { emit }) {
    const isConfirmSelected = ref(props.defaultConfirm);

    const toggle = () => {
      isConfirmSelected.value = !isConfirmSelected.value;
    };
    const activate = () => {
      emit("select", isConfirmSelected.value);
      emit(isConfirmSelected.value ? "confirm" : "cancel");
    };

    useInput({
      isActive: computed(() => props.focused),
      onArrow: toggle,
      onKey: (key) => {
        if (key === "tab" || key === "backtab") toggle();
      },
      onChar: (character) => {
        if (character === " ") activate();
      },
      onSubmit: activate,
      onEscape: () => emit("cancel"),
    });

    return () => {
      return h(
        "box",
        {
          style: { flex_direction: "column" },
        },
        [
          // Message
          h("text", { key: "message" }, props.message),
          // Buttons
          h(
            "box",
            {
              key: "buttons",
              style: {
                flex_direction: "row",
                gap: 2,
                margin_top: 1,
              },
            },
            [
              h(
                "text",
                {
                  key: "confirm",
                  fg: isConfirmSelected.value ? props.selectedFg : props.confirmFg,
                  bold: isConfirmSelected.value,
                },
                `[${props.confirmText}]`,
              ),
              h(
                "text",
                {
                  key: "cancel",
                  fg: !isConfirmSelected.value ? props.selectedFg : props.cancelFg,
                  bold: !isConfirmSelected.value,
                },
                `[${props.cancelText}]`,
              ),
            ],
          ),
        ],
      );
    };
  },
});

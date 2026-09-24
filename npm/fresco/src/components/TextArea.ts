/**
 * TextArea Component - Multiline text input
 */

import { defineComponent, h, type PropType, computed } from "vue";

export interface TextAreaProps {
  /** Text value */
  modelValue?: string;
  /** Placeholder text */
  placeholder?: string;
  /** Number of visible rows */
  rows?: number;
  /** Whether the textarea is focused */
  focused?: boolean;
  /** Whether the textarea is disabled */
  disabled?: boolean;
  /** Show line numbers */
  lineNumbers?: boolean;
  /** Border style */
  border?: "single" | "double" | "rounded" | "none";
  /** Foreground color */
  fg?: string;
  /** Placeholder foreground color */
  placeholderFg?: string;
  /** Line number foreground color */
  lineNumberFg?: string;
  /** Cursor line */
  cursorLine?: number;
  /** Cursor column */
  cursorColumn?: number;
}

export const TextArea = defineComponent({
  name: "TextArea",
  props: {
    modelValue: {
      type: String,
      default: "",
    },
    placeholder: String,
    rows: {
      type: Number,
      default: 5,
    },
    focused: {
      type: Boolean,
      default: false,
    },
    disabled: {
      type: Boolean,
      default: false,
    },
    lineNumbers: {
      type: Boolean,
      default: false,
    },
    border: {
      type: String as PropType<TextAreaProps["border"]>,
      default: "single",
    },
    fg: String,
    placeholderFg: {
      type: String,
      default: "gray",
    },
    lineNumberFg: {
      type: String,
      default: "gray",
    },
    cursorLine: {
      type: Number,
      default: 0,
    },
    cursorColumn: {
      type: Number,
      default: 0,
    },
  },
  emits: ["update:modelValue"],
  setup(props) {
    const lines = computed(() => {
      const text = props.modelValue || "";
      const textLines = text.split("\n");

      // Pad to minimum rows
      while (textLines.length < props.rows) {
        textLines.push("");
      }

      return textLines.slice(0, props.rows);
    });

    const showPlaceholder = computed(
      () => !props.modelValue && props.placeholder && !props.focused,
    );

    return () => {
      const lineNumWidth = String(lines.value.length).length;

      const children = lines.value.map((line, index) => {
        const isCursorLine = props.focused && index === props.cursorLine;
        const parts = [];

        // Line number
        if (props.lineNumbers) {
          parts.push(
            h(
              "text",
              {
                key: `ln-${index}`,
                fg: props.lineNumberFg,
                dim: !isCursorLine,
              },
              `${String(index + 1).padStart(lineNumWidth)} │ `,
            ),
          );
        }

        // Line content
        let content = line || (showPlaceholder.value && index === 0 ? props.placeholder : " ");

        parts.push(
          h(
            "text",
            {
              key: `content-${index}`,
              fg: showPlaceholder.value && index === 0 ? props.placeholderFg : props.fg,
              dim: props.disabled,
              bold: isCursorLine,
            },
            content,
          ),
        );

        return h(
          "box",
          {
            key: `line-${index}`,
            style: { flex_direction: "row" },
            bg: isCursorLine ? "gray" : undefined,
          },
          parts,
        );
      });

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          style: {
            flex_direction: "column",
            padding: props.border !== "none" ? 1 : 0,
            height: String(props.rows + (props.border !== "none" ? 2 : 0)),
          },
        },
        children,
      );
    };
  },
});

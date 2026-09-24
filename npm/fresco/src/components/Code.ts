/**
 * Code Component - Code block display
 */

import { defineComponent, h, type PropType } from "vue";

export interface CodeProps {
  /** Code content */
  code: string;
  /** Language (for future syntax highlighting) */
  language?: string;
  /** Show line numbers */
  lineNumbers?: boolean;
  /** Starting line number */
  startLine?: number;
  /** Highlight specific lines */
  highlightLines?: number[];
  /** Border style */
  border?: "single" | "double" | "rounded" | "none";
  /** Code foreground color */
  fg?: string;
  /** Line number foreground color */
  lineNumberFg?: string;
  /** Highlight line background */
  highlightBg?: string;
}

export const Code = defineComponent({
  name: "Code",
  props: {
    code: {
      type: String,
      required: true,
    },
    language: String,
    lineNumbers: {
      type: Boolean,
      default: true,
    },
    startLine: {
      type: Number,
      default: 1,
    },
    highlightLines: {
      type: Array as PropType<number[]>,
      default: () => [],
    },
    border: {
      type: String as PropType<CodeProps["border"]>,
      default: "single",
    },
    fg: {
      type: String,
      default: "white",
    },
    lineNumberFg: {
      type: String,
      default: "gray",
    },
    highlightBg: {
      type: String,
      default: "blue",
    },
  },
  setup(props) {
    return () => {
      const lines = props.code.split("\n");
      const maxLineNum = props.startLine + lines.length - 1;
      const lineNumWidth = String(maxLineNum).length;

      const children = lines.map((line, index) => {
        const lineNum = props.startLine + index;
        const isHighlighted = props.highlightLines?.includes(lineNum);

        const parts = [];

        if (props.lineNumbers) {
          parts.push(
            h(
              "text",
              {
                key: `ln-${lineNum}`,
                fg: props.lineNumberFg,
              },
              `${String(lineNum).padStart(lineNumWidth)} │ `,
            ),
          );
        }

        parts.push(
          h(
            "text",
            {
              key: `code-${lineNum}`,
              fg: props.fg,
              bg: isHighlighted ? props.highlightBg : undefined,
            },
            line || " ",
          ),
        );

        return h(
          "box",
          {
            key: `line-${lineNum}`,
            style: { flex_direction: "row" },
          },
          parts,
        );
      });

      // Add language label if provided
      if (props.language) {
        children.unshift(
          h(
            "text",
            {
              key: "lang",
              dim: true,
              style: { margin_bottom: 1 },
            },
            `// ${props.language}`,
          ),
        );
      }

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          style: {
            flex_direction: "column",
            padding: props.border !== "none" ? 1 : 0,
          },
        },
        children,
      );
    };
  },
});

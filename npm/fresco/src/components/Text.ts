/**
 * Text Component - Text display
 */

import { defineComponent, h, type PropType } from "@vue/runtime-core";
import { stringifyChildren } from "../utils/text.js";

export type TextWrap =
  | boolean
  | "wrap"
  | "end"
  | "middle"
  | "truncate"
  | "truncate-start"
  | "truncate-middle"
  | "truncate-end";

export interface TextProps {
  /** Text content (alternative to slot) */
  content?: string;
  /** Ink-compatible text wrapping/truncation mode */
  wrap?: TextWrap;
  /** Foreground color (Fresco alias) */
  fg?: string;
  /** Foreground color (Ink alias) */
  color?: string;
  /** Background color (Fresco alias) */
  bg?: string;
  /** Background color (Ink alias) */
  backgroundColor?: string;
  /** Bold text */
  bold?: boolean;
  /** Dim text (Fresco alias) */
  dim?: boolean;
  /** Dim text (Ink alias) */
  dimColor?: boolean;
  /** Italic text */
  italic?: boolean;
  /** Underlined text */
  underline?: boolean;
  /** Strikethrough text */
  strikethrough?: boolean;
  /** Inverse background/foreground colors */
  inverse?: boolean;
  /** Accessibility label, accepted for Ink API parity */
  "aria-label"?: string;
  /** Hide from screen readers, accepted for Ink API parity */
  "aria-hidden"?: boolean;
}

export const Text = defineComponent({
  name: "Text",
  props: {
    content: String,
    wrap: [Boolean, String] as PropType<TextProps["wrap"]>,
    fg: String,
    color: String,
    bg: String,
    backgroundColor: String,
    bold: Boolean,
    dim: Boolean,
    dimColor: Boolean,
    italic: Boolean,
    underline: Boolean,
    strikethrough: Boolean,
    inverse: Boolean,
    "aria-label": String,
    "aria-hidden": Boolean,
  },
  setup(props, { slots }) {
    return () => {
      const text = props.content ?? stringifyChildren(slots.default?.());

      return h("text", {
        text,
        wrap: props.wrap,
        fg: props.fg ?? props.color,
        bg: props.bg ?? props.backgroundColor,
        bold: props.bold,
        dim: props.dim || props.dimColor,
        italic: props.italic,
        underline: props.underline,
        strikethrough: props.strikethrough,
        inverse: props.inverse,
        "aria-label": props["aria-label"],
        "aria-hidden": props["aria-hidden"],
      });
    };
  },
});

/**
 * Convenience components for common text styles
 */

export const ErrorText = defineComponent({
  name: "ErrorText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "red", ...props }, slots);
  },
});

export const WarningText = defineComponent({
  name: "WarningText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "yellow", ...props }, slots);
  },
});

export const SuccessText = defineComponent({
  name: "SuccessText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "green", ...props }, slots);
  },
});

export const InfoText = defineComponent({
  name: "InfoText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "blue", ...props }, slots);
  },
});

export const MutedText = defineComponent({
  name: "MutedText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { dim: true, ...props }, slots);
  },
});

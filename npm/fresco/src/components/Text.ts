/**
 * Text Component - Text display
 */

import { defineComponent, h, type PropType } from "@vue/runtime-core";
import { useIsScreenReaderEnabled } from "../composables/useIsScreenReaderEnabled.js";
import type { FrescoAppearance } from "../protocol.js";
import { stringifyChildren } from "../utils/text.js";

export type TextWrap =
  | boolean
  | "wrap"
  | "hard"
  | "truncate"
  | "truncate-start"
  | "truncate-middle"
  | "truncate-end"
  | "end"
  | "middle";

export interface TextProps extends FrescoAppearance {
  /** Text content (alternative to slot) */
  content?: string;
  /** Ink-compatible text wrapping/truncation mode */
  wrap?: TextWrap;
  /** Foreground color (Ink alias) */
  color?: string;
  /** Background color (Ink alias) */
  backgroundColor?: string;
  /** Dim text (Ink alias) */
  dimColor?: boolean;
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
    blink: Boolean,
    hidden: Boolean,
    // Declared camelCase so the runtime props object (which Vue camelizes)
    // matches these keys; templates and h() may still pass "aria-label" etc.
    ariaLabel: String,
    ariaHidden: Boolean,
  },
  setup(props, { slots }) {
    const isScreenReaderEnabled = useIsScreenReaderEnabled();

    return () => {
      if (isScreenReaderEnabled && props.ariaHidden) return null;

      const text =
        isScreenReaderEnabled && props.ariaLabel
          ? props.ariaLabel
          : (props.content ?? stringifyChildren(slots.default?.()));

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
        ...(props.blink ? { blink: true } : {}),
        ...(props.hidden ? { hidden: true } : {}),
        "aria-label": props.ariaLabel,
        "aria-hidden": props.ariaHidden,
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

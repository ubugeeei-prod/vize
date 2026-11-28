/* eslint-disable script/one-component-per-file -- this module defines more than one component */
/**
 * Text Component - Text display
 */

import { defineComponent, h, type PropType } from "vue";
import { useIsScreenReaderEnabled } from "../composables/useIsScreenReaderEnabled.js";
import type { FrescoAppearance } from "../protocol.js";
import { stringifyChildren } from "../utils/text.js";

/** Wrapping and truncation policies accepted by terminal text nodes. */
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

/** Text content, wrapping policy, and terminal appearance with compatibility aliases. */
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

/** Render terminal text from content or the default slot. */
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

/** Render red terminal text for errors. */
export const ErrorText = defineComponent({
  name: "ErrorText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "red", ...props }, slots);
  },
});

/** Render yellow terminal text for warnings. */
export const WarningText = defineComponent({
  name: "WarningText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "yellow", ...props }, slots);
  },
});

/** Render green terminal text for successful outcomes. */
export const SuccessText = defineComponent({
  name: "SuccessText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "green", ...props }, slots);
  },
});

/** Render blue terminal text for informational output. */
export const InfoText = defineComponent({
  name: "InfoText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { fg: "blue", ...props }, slots);
  },
});

/** Render terminal text with dim appearance. */
export const MutedText = defineComponent({
  name: "MutedText",
  props: {
    content: String,
  },
  setup(props, { slots }) {
    return () => h(Text, { dim: true, ...props }, slots);
  },
});

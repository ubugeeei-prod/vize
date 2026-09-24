/**
 * ProgressBar Component - Progress indicator
 */

import { defineComponent, h, computed, type PropType } from "vue";
import { Box } from "./Box.js";
import { Text } from "./Text.js";

export interface ProgressBarProps {
  /** Progress value (0-100) */
  value: number;
  /** Total width in characters */
  width?: number;
  /** Show percentage label */
  showLabel?: boolean;
  /** Label position */
  labelPosition?: "left" | "right" | "inside";
  /** Filled character */
  filledChar?: string;
  /** Empty character */
  emptyChar?: string;
  /** Left border character */
  leftBorder?: string;
  /** Right border character */
  rightBorder?: string;
  /** Filled color */
  filledFg?: string;
  /** Empty color */
  emptyFg?: string;
}

export const ProgressBar = defineComponent({
  name: "ProgressBar",
  props: {
    value: {
      type: Number,
      required: true,
      validator: (v: number) => v >= 0 && v <= 100,
    },
    width: {
      type: Number,
      default: 20,
    },
    showLabel: {
      type: Boolean,
      default: true,
    },
    labelPosition: {
      type: String as PropType<"left" | "right" | "inside">,
      default: "right",
    },
    filledChar: {
      type: String,
      default: "█",
    },
    emptyChar: {
      type: String,
      default: "░",
    },
    leftBorder: {
      type: String,
      default: "",
    },
    rightBorder: {
      type: String,
      default: "",
    },
    filledFg: {
      type: String,
      default: "green",
    },
    emptyFg: {
      type: String,
      default: "gray",
    },
  },
  setup(props) {
    const normalizedValue = computed(() => Math.max(0, Math.min(100, props.value)));

    const filledWidth = computed(() => Math.round((normalizedValue.value / 100) * props.width));

    const emptyWidth = computed(() => props.width - filledWidth.value);

    const label = computed(() => `${Math.round(normalizedValue.value)}%`);

    return () => {
      const filled = props.filledChar.repeat(filledWidth.value);
      const empty = props.emptyChar.repeat(emptyWidth.value);

      const barContent = [
        props.leftBorder && h(Text, {}, () => props.leftBorder),
        h(Text, { fg: props.filledFg }, () => filled),
        h(Text, { fg: props.emptyFg }, () => empty),
        props.rightBorder && h(Text, {}, () => props.rightBorder),
      ].filter(Boolean);

      if (!props.showLabel) {
        return h(Box, { flexDirection: "row" }, () => barContent);
      }

      const labelElement = h(Text, { dim: true }, () => label.value);

      switch (props.labelPosition) {
        case "left":
          return h(Box, { flexDirection: "row", gap: 1 }, () => [labelElement, ...barContent]);
        case "inside":
          // For inside, we'd need more complex rendering
          // For now, show on right
          return h(Box, { flexDirection: "row", gap: 1 }, () => [...barContent, labelElement]);
        case "right":
        default:
          return h(Box, { flexDirection: "row", gap: 1 }, () => [...barContent, labelElement]);
      }
    };
  },
});

/**
 * Indeterminate progress bar (animated)
 */
export const IndeterminateProgressBar = defineComponent({
  name: "IndeterminateProgressBar",
  props: {
    width: {
      type: Number,
      default: 20,
    },
    fg: {
      type: String,
      default: "cyan",
    },
  },
  setup(props) {
    // This would need animation support
    // For now, show a static pattern
    return () => {
      const pattern = "▓▒░░░░░░░░░░░░░░░░░░";
      const display = pattern.slice(0, props.width);

      return h(Text, { fg: props.fg }, () => display);
    };
  },
});

/**
 * Stack Component - Horizontal/Vertical stack layout helper
 */

import { defineComponent, h, type PropType } from "vue";

export interface StackProps {
  /** Stack direction */
  direction?: "horizontal" | "vertical";
  /** Gap between children */
  gap?: number;
  /** Align items */
  align?: "start" | "center" | "end" | "stretch";
  /** Justify content */
  justify?: "start" | "center" | "end" | "between" | "around" | "evenly";
  /** Wrap children */
  wrap?: boolean;
}

const ALIGN_MAP: Record<string, string> = {
  start: "flex-start",
  center: "center",
  end: "flex-end",
  stretch: "stretch",
};

const JUSTIFY_MAP: Record<string, string> = {
  start: "flex-start",
  center: "center",
  end: "flex-end",
  between: "space-between",
  around: "space-around",
  evenly: "space-evenly",
};

export const Stack = defineComponent({
  name: "Stack",
  props: {
    direction: {
      type: String as PropType<"horizontal" | "vertical">,
      default: "vertical",
    },
    gap: {
      type: Number,
      default: 0,
    },
    align: {
      type: String as PropType<StackProps["align"]>,
      default: "stretch",
    },
    justify: {
      type: String as PropType<StackProps["justify"]>,
      default: "start",
    },
    wrap: {
      type: Boolean,
      default: false,
    },
  },
  setup(props, { slots }) {
    return () => {
      return h(
        "box",
        {
          style: {
            flex_direction: props.direction === "horizontal" ? "row" : "column",
            gap: props.gap,
            align_items: ALIGN_MAP[props.align ?? "stretch"],
            justify_content: JUSTIFY_MAP[props.justify ?? "start"],
            flex_wrap: props.wrap ? "wrap" : "nowrap",
          },
        },
        slots.default?.(),
      );
    };
  },
});

// Convenience components
export const HStack = defineComponent({
  name: "HStack",
  props: {
    gap: { type: Number, default: 1 },
    align: String as PropType<StackProps["align"]>,
    justify: String as PropType<StackProps["justify"]>,
  },
  setup(props, { slots }) {
    return () => h(Stack, { direction: "horizontal", ...props }, slots.default);
  },
});

export const VStack = defineComponent({
  name: "VStack",
  props: {
    gap: { type: Number, default: 0 },
    align: String as PropType<StackProps["align"]>,
    justify: String as PropType<StackProps["justify"]>,
  },
  setup(props, { slots }) {
    return () => h(Stack, { direction: "vertical", ...props }, slots.default);
  },
});

/**
 * Tooltip Component - Tooltip overlay
 */

import { defineComponent, h, type PropType, type VNode } from "vue";

export type TooltipPosition = "top" | "bottom" | "left" | "right";

export interface TooltipProps {
  /** Tooltip text */
  text: string;
  /** Whether tooltip is visible */
  visible?: boolean;
  /** Tooltip position */
  position?: TooltipPosition;
  /** Border style */
  border?: "single" | "rounded" | "none";
  /** Background color */
  bg?: string;
  /** Foreground color */
  fg?: string;
}

export const Tooltip = defineComponent({
  name: "Tooltip",
  props: {
    text: {
      type: String,
      required: true,
    },
    visible: {
      type: Boolean,
      default: true,
    },
    position: {
      type: String as PropType<TooltipPosition>,
      default: "top",
    },
    border: {
      type: String as PropType<TooltipProps["border"]>,
      default: "rounded",
    },
    bg: {
      type: String,
      default: "white",
    },
    fg: {
      type: String,
      default: "black",
    },
  },
  setup(props, { slots }) {
    return () => {
      const content = slots.default?.();

      if (!props.visible) {
        return h("box", {}, content);
      }

      const tooltip = h(
        "box",
        {
          key: "tooltip",
          border: props.border === "none" ? undefined : props.border,
          bg: props.bg,
          fg: props.fg,
          style: {
            padding_left: 1,
            padding_right: 1,
          },
        },
        [h("text", { fg: props.fg, bg: props.bg }, props.text)],
      );

      const children: VNode[] = [];

      switch (props.position) {
        case "top":
          children.push(tooltip);
          children.push(h("box", { key: "content" }, content));
          break;
        case "bottom":
          children.push(h("box", { key: "content" }, content));
          children.push(tooltip);
          break;
        case "left":
          return h("box", { style: { flex_direction: "row" } }, [
            tooltip,
            h("box", { key: "content" }, content),
          ]);
        case "right":
          return h("box", { style: { flex_direction: "row" } }, [
            h("box", { key: "content" }, content),
            tooltip,
          ]);
      }

      return h("box", { style: { flex_direction: "column" } }, children);
    };
  },
});

/**
 * Badge Component - Status badge/tag
 */

import { defineComponent, h, type PropType } from "vue";

export type BadgeVariant = "default" | "success" | "warning" | "error" | "info";

export interface BadgeProps {
  /** Badge text */
  label: string;
  /** Badge variant */
  variant?: BadgeVariant;
  /** Custom foreground color (overrides variant) */
  fg?: string;
  /** Custom background color (overrides variant) */
  bg?: string;
  /** Show border */
  border?: boolean;
}

const VARIANT_COLORS: Record<BadgeVariant, { fg: string; bg?: string }> = {
  default: { fg: "white" },
  success: { fg: "green" },
  warning: { fg: "yellow" },
  error: { fg: "red" },
  info: { fg: "cyan" },
};

export const Badge = defineComponent({
  name: "Badge",
  props: {
    label: {
      type: String,
      required: true,
    },
    variant: {
      type: String as PropType<BadgeVariant>,
      default: "default",
    },
    fg: String,
    bg: String,
    border: {
      type: Boolean,
      default: false,
    },
  },
  setup(props) {
    return () => {
      const colors = VARIANT_COLORS[props.variant];
      const fg = props.fg ?? colors.fg;
      const bg = props.bg ?? colors.bg;

      if (props.border) {
        return h(
          "box",
          {
            border: "single",
            fg,
            bg,
            style: { padding_left: 1, padding_right: 1 },
          },
          [h("text", { fg, bg }, props.label)],
        );
      }

      return h(
        "text",
        {
          fg,
          bg,
          bold: true,
        },
        `[${props.label}]`,
      );
    };
  },
});

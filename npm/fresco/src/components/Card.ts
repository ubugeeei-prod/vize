/**
 * Card Component - Container card with optional header and footer
 */

import { defineComponent, h, type PropType, type VNode } from "vue";

export interface CardProps {
  /** Card title */
  title?: string;
  /** Card subtitle */
  subtitle?: string;
  /** Footer text */
  footer?: string;
  /** Border style */
  border?: "single" | "double" | "rounded" | "heavy" | "none";
  /** Padding */
  padding?: number;
  /** Title foreground color */
  titleFg?: string;
  /** Border foreground color */
  borderFg?: string;
  /** Background color */
  bg?: string;
}

export const Card = defineComponent({
  name: "Card",
  props: {
    title: String,
    subtitle: String,
    footer: String,
    border: {
      type: String as PropType<CardProps["border"]>,
      default: "rounded",
    },
    padding: {
      type: Number,
      default: 1,
    },
    titleFg: {
      type: String,
      default: "white",
    },
    borderFg: String,
    bg: String,
  },
  setup(props, { slots }) {
    return () => {
      const children: VNode[] = [];

      // Header
      if (props.title || props.subtitle) {
        const headerContent: VNode[] = [];

        if (props.title) {
          headerContent.push(h("text", { fg: props.titleFg, bold: true }, props.title));
        }

        if (props.subtitle) {
          headerContent.push(
            h("text", { dim: true }, props.title ? ` - ${props.subtitle}` : props.subtitle),
          );
        }

        children.push(
          h(
            "box",
            {
              key: "header",
              style: {
                flex_direction: "row",
                margin_bottom: 1,
              },
            },
            headerContent,
          ),
        );
      }

      // Content
      children.push(
        h(
          "box",
          {
            key: "content",
            style: { flex_grow: 1 },
          },
          slots.default?.(),
        ),
      );

      // Footer
      if (props.footer || slots.footer) {
        children.push(
          h(
            "box",
            {
              key: "footer",
              style: { margin_top: 1 },
            },
            slots.footer?.() ?? [h("text", { dim: true }, props.footer)],
          ),
        );
      }

      return h(
        "box",
        {
          border: props.border === "none" ? undefined : props.border,
          fg: props.borderFg,
          bg: props.bg,
          style: {
            flex_direction: "column",
            padding: props.padding,
          },
        },
        children,
      );
    };
  },
});

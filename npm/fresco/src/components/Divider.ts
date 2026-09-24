/**
 * Divider Component - Horizontal or vertical divider line
 */

import { defineComponent, h, type PropType } from "vue";

export interface DividerProps {
  /** Divider direction */
  direction?: "horizontal" | "vertical";
  /** Divider character */
  char?: string;
  /** Title in the middle of the divider */
  title?: string;
  /** Foreground color */
  fg?: string;
  /** Title foreground color */
  titleFg?: string;
}

export const Divider = defineComponent({
  name: "Divider",
  props: {
    direction: {
      type: String as PropType<"horizontal" | "vertical">,
      default: "horizontal",
    },
    char: String,
    title: String,
    fg: {
      type: String,
      default: "gray",
    },
    titleFg: String,
  },
  setup(props) {
    return () => {
      const dividerChar = props.char ?? (props.direction === "horizontal" ? "─" : "│");

      if (props.direction === "vertical") {
        return h(
          "text",
          {
            fg: props.fg,
          },
          dividerChar,
        );
      }

      // Horizontal divider
      if (props.title) {
        return h(
          "box",
          {
            style: { flex_direction: "row", align_items: "center" },
          },
          [
            h("text", { fg: props.fg }, dividerChar.repeat(3)),
            h(
              "text",
              {
                fg: props.titleFg ?? props.fg,
                bold: true,
              },
              ` ${props.title} `,
            ),
            h("text", { fg: props.fg }, dividerChar.repeat(3)),
          ],
        );
      }

      // Simple divider - width is handled by parent container
      return h(
        "text",
        {
          fg: props.fg,
        },
        dividerChar.repeat(40),
      );
    };
  },
});

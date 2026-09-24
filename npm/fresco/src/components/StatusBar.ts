/**
 * StatusBar Component - Status bar (typically at bottom of screen)
 */

import { defineComponent, h, type PropType, type VNode } from "vue";

export interface StatusBarItem {
  key: string;
  content: string;
  fg?: string;
  bg?: string;
  bold?: boolean;
  align?: "left" | "right";
}

export interface StatusBarProps {
  /** Status bar items */
  items: StatusBarItem[];
  /** Background color */
  bg?: string;
  /** Foreground color */
  fg?: string;
  /** Separator between items */
  separator?: string;
}

export const StatusBar = defineComponent({
  name: "StatusBar",
  props: {
    items: {
      type: Array as PropType<StatusBarItem[]>,
      required: true,
    },
    bg: {
      type: String,
      default: "blue",
    },
    fg: {
      type: String,
      default: "white",
    },
    separator: {
      type: String,
      default: " │ ",
    },
  },
  setup(props) {
    return () => {
      const leftItems = props.items.filter((item) => item.align !== "right");
      const rightItems = props.items.filter((item) => item.align === "right");

      const renderItems = (items: StatusBarItem[]): VNode[] => {
        const result: VNode[] = [];
        items.forEach((item, index) => {
          if (index > 0) {
            result.push(
              h(
                "text",
                { key: `sep-${item.key}`, fg: props.fg, bg: props.bg, dim: true },
                props.separator,
              ),
            );
          }
          result.push(
            h(
              "text",
              {
                key: item.key,
                fg: item.fg ?? props.fg,
                bg: item.bg ?? props.bg,
                bold: item.bold,
              },
              item.content,
            ),
          );
        });
        return result;
      };

      return h(
        "box",
        {
          bg: props.bg,
          style: {
            flex_direction: "row",
            justify_content: "space-between",
            width: "100%",
            padding_left: 1,
            padding_right: 1,
          },
        },
        [
          h("box", { key: "left", style: { flex_direction: "row" } }, renderItems(leftItems)),
          h("box", { key: "right", style: { flex_direction: "row" } }, renderItems(rightItems)),
        ],
      );
    };
  },
});

/**
 * KeyHint Component - Display keyboard shortcut hints
 */

import { defineComponent, h, type PropType } from "vue";

export interface KeyBinding {
  keys: string[];
  description: string;
}

export interface KeyHintProps {
  /** Key bindings to display */
  bindings: KeyBinding[];
  /** Layout direction */
  direction?: "horizontal" | "vertical";
  /** Key foreground color */
  keyFg?: string;
  /** Key background color */
  keyBg?: string;
  /** Description foreground color */
  descFg?: string;
  /** Separator between key and description */
  separator?: string;
}

export const KeyHint = defineComponent({
  name: "KeyHint",
  props: {
    bindings: {
      type: Array as PropType<KeyBinding[]>,
      required: true,
    },
    direction: {
      type: String as PropType<"horizontal" | "vertical">,
      default: "horizontal",
    },
    keyFg: {
      type: String,
      default: "black",
    },
    keyBg: {
      type: String,
      default: "white",
    },
    descFg: {
      type: String,
      default: "gray",
    },
    separator: {
      type: String,
      default: " ",
    },
  },
  setup(props) {
    return () => {
      const children = props.bindings.map((binding, index) => {
        const keyParts = binding.keys
          .map((key, keyIndex) => [
            h(
              "text",
              {
                key: `key-${keyIndex}`,
                fg: props.keyFg,
                bg: props.keyBg,
                bold: true,
              },
              ` ${key} `,
            ),
            keyIndex < binding.keys.length - 1 ? h("text", { key: `plus-${keyIndex}` }, "+") : null,
          ])
          .flat()
          .filter(Boolean);

        return h(
          "box",
          {
            key: `binding-${index}`,
            style: {
              flex_direction: "row",
              margin_right: props.direction === "horizontal" ? 2 : 0,
            },
          },
          [
            ...keyParts,
            h("text", {}, props.separator),
            h("text", { fg: props.descFg }, binding.description),
          ],
        );
      });

      return h(
        "box",
        {
          style: {
            flex_direction: props.direction === "horizontal" ? "row" : "column",
            flex_wrap: "wrap",
          },
        },
        children,
      );
    };
  },
});

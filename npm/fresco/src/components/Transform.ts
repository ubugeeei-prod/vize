/**
 * Transform Component - transforms stringified child output before rendering.
 */

import { defineComponent, h, type PropType } from "@vue/runtime-core";
import { stringifyChildren } from "../utils/text.js";

export interface TransformProps {
  /** Screen-reader-specific label, accepted for Ink API parity */
  accessibilityLabel?: string;
  /** Transform each rendered line */
  transform: (children: string, index: number) => string;
}

export const Transform = defineComponent({
  name: "Transform",
  props: {
    accessibilityLabel: String,
    transform: {
      type: Function as PropType<TransformProps["transform"]>,
      required: true,
    },
  },
  setup(props, { slots }) {
    return () => {
      const text = stringifyChildren(slots.default?.());
      const transformed = text
        .split("\n")
        .map((line, index) => props.transform(line, index))
        .join("\n");

      return h("text", {
        text: transformed,
        "aria-label": props.accessibilityLabel,
      });
    };
  },
});

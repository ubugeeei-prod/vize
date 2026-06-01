/**
 * Newline Component - inserts one or more newline characters inside Text.
 */

import { defineComponent, h } from "@vue/runtime-core";

export interface NewlineProps {
  /** Number of newlines to insert */
  count?: number;
}

export const Newline = defineComponent({
  name: "Newline",
  props: {
    count: {
      type: Number,
      default: 1,
    },
  },
  setup(props) {
    return () => h("text", { text: "\n".repeat(Math.max(0, props.count)) });
  },
});

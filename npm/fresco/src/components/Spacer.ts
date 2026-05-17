/**
 * Spacer Component - flexible empty space along the parent main axis.
 */

import { defineComponent, h } from "@vue/runtime-core";

export const Spacer = defineComponent({
  name: "Spacer",
  setup() {
    return () =>
      h("box", {
        style: {
          flexGrow: 1,
          flexShrink: 1,
        },
      });
  },
});

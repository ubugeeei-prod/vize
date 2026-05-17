/**
 * Static Component - renders stable items above the live area.
 *
 * Fresco keeps this component declarative for Vue. The native renderer does not
 * yet persist historical frames separately, so current items are rendered in
 * order with the same child function API as Ink.
 */

import { defineComponent, h, type PropType } from "@vue/runtime-core";

export interface StaticProps<T = unknown> {
  /** Items to render */
  items: T[];
  /** Optional container style */
  style?: Record<string, unknown>;
  /** Render item callback */
  children?: (item: T, index: number) => unknown;
}

export const Static = defineComponent({
  name: "Static",
  props: {
    items: {
      type: Array as PropType<unknown[]>,
      default: () => [],
    },
    style: Object as PropType<Record<string, unknown>>,
  },
  setup(props, { slots }) {
    return () => {
      const renderItem = slots.default;

      return h(
        "box",
        {
          style: {
            flexDirection: "column",
            ...props.style,
          },
        },
        props.items.flatMap((item, index) => renderItem?.({ item, index }) ?? []),
      );
    };
  },
});

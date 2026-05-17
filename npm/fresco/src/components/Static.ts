/**
 * Static Component - renders stable items above the live area.
 *
 * The app renderer treats this component specially: newly added items are
 * promoted into a persistent output region above the live frame.
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
          internal_static: true,
          style: {
            position: "absolute",
            flexDirection: "column",
            ...props.style,
          },
        },
        props.items.map((item, index) =>
          h(
            "box",
            {
              key: index,
              internal_static_item: true,
              style: {
                flexDirection: "column",
              },
            },
            renderItem?.({ item, index }) ?? [],
          ),
        ),
      );
    };
  },
});

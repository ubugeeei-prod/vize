/**
 * Grid Component - Grid layout helper
 */

import { defineComponent, h, type VNode } from "vue";

/** Column count and terminal-cell spacing for a fixed-column grid. */
export interface GridProps {
  /** Number of columns
   * @default 2
   */
  columns?: number;
  /** Gap between cells
   * @default 1
   */
  gap?: number;
  /** Row gap
   * @default undefined
   */
  rowGap?: number;
  /** Column gap
   * @default undefined
   */
  columnGap?: number;
}

/** Arrange default-slot nodes into rows with a fixed number of columns. */
export const Grid = defineComponent({
  name: "Grid",
  props: {
    columns: {
      type: Number,
      default: 2,
    },
    gap: {
      type: Number,
      default: 1,
    },
    rowGap: Number,
    columnGap: Number,
  },
  setup(props, { slots }) {
    return () => {
      const children = slots.default?.() ?? [];
      const flatChildren = Array.isArray(children) ? children.flat() : [children];

      const rows: VNode[][] = [];
      let currentRow: VNode[] = [];

      flatChildren.forEach((child, index) => {
        currentRow.push(child);
        if (currentRow.length === props.columns || index === flatChildren.length - 1) {
          rows.push([...currentRow]);
          currentRow = [];
        }
      });

      const rowGap = props.rowGap ?? props.gap;
      const columnGap = props.columnGap ?? props.gap;

      return h(
        "box",
        {
          style: {
            flex_direction: "column",
            gap: rowGap,
          },
        },
        rows.map((row, rowIndex) =>
          h(
            "box",
            {
              key: `row-${rowIndex}`,
              style: {
                flex_direction: "row",
                gap: columnGap,
              },
            },
            row.map((cell, cellIndex) =>
              h(
                "box",
                {
                  key: `cell-${rowIndex}-${cellIndex}`,
                  style: { flex_grow: 1 },
                },
                [cell],
              ),
            ),
          ),
        ),
      );
    };
  },
});

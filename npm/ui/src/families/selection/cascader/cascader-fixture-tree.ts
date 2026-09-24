import { h } from "vue";
import type { VNode } from "vue";

import CascaderColumn from "./cascader-column.vue";
import CascaderContent from "./cascader-content.vue";
import CascaderItem from "./cascader-item.vue";
import CascaderRoot from "./cascader-root.vue";
import CascaderTrigger from "./cascader-trigger.vue";
import CascaderValue from "./cascader-value.vue";
import type { CascaderSlotState } from "./cascader-types.ts";

/** Fixture node shared by tests and runtime fixtures. */
export interface Place {
  readonly value: string;
  readonly label: string;
  readonly children?: readonly Place[];
  readonly disabled?: boolean;
}

export const places: readonly Place[] = Object.freeze([
  {
    children: [
      {
        children: [
          { label: "Shibuya", value: "shibuya" },
          { label: "Shinjuku", value: "shinjuku" },
        ],
        label: "Tokyo",
        value: "tokyo",
      },
      { children: [{ label: "Kita", value: "kita" }], label: "Osaka", value: "osaka" },
    ],
    label: "Japan",
    value: "jp",
  },
  { disabled: true, label: "Mars", value: "mars" },
  {
    children: [
      { label: "Paris", value: "paris" },
      { label: "Lyon", value: "lyon" },
    ],
    label: "France",
    value: "fr",
  },
]);

/** Props every fixture Cascader shares. */
export const cascaderFixtureProps = Object.freeze({
  by: "value",
  itemDisabled: (place: Place) => place.disabled === true,
  itemText: (place: Place) => place.label,
  options: places,
});

/** Default slot rendering trigger, value, and one column per root column. */
export function cascaderFixtureSlot(contentProps: Record<string, unknown> = {}) {
  return (state: CascaderSlotState<Place>): VNode[] => [
    h(CascaderTrigger, { ariaLabel: "Place" }, () => h(CascaderValue)),
    h(CascaderContent, contentProps, () =>
      state.columns.map((column) =>
        h(CascaderColumn, { key: column.level, level: column.level }, () =>
          column.options.map((place) =>
            h(CascaderItem<Place>, { key: place.value, value: place }, () => place.label),
          ),
        ),
      ),
    ),
  ];
}

/** Render a Cascader over `places` with columns built from root slot state. */
export function renderCascaderTree(
  props: Record<string, unknown> = {},
  contentProps: Record<string, unknown> = {},
): VNode {
  return h(
    CascaderRoot<Place>,
    { ...cascaderFixtureProps, ...props },
    { default: cascaderFixtureSlot(contentProps) },
  );
}

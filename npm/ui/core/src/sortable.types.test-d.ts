/** Compile-only assertions for the public sortable contract. */

import { ref } from "vue";
import type { HTMLAttributes, ShallowRef } from "vue";

import type { DragSourceProps } from "./drag-and-drop.ts";
import {
  createSortable,
  type SortableIndicatorState,
  type SortableOrientation,
  type SortablePosition,
} from "./sortable.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

export const orientations: readonly SortableOrientation[] = ["grid", "horizontal", "vertical"];
// @ts-expect-error the orientation union is closed.
export const invalidOrientation: SortableOrientation = "stack";

export const positions: readonly SortablePosition[] = ["after", "before", "inside"];
// @ts-expect-error the position union is closed.
export const invalidPosition: SortablePosition = "around";

const disabled = ref(false);
export const controller = createSortable({
  orientation: () => "grid",
  columns: 2,
  direction: "rtl",
  nesting: true,
  isDisabled: disabled,
  onSortCommit(event) {
    const fromIndex: number = event.fromIndex;
    const overKey: string | null = event.overKey;
    void fromIndex;
    void overKey;
    if (event.position === "inside") {
      const receiving: string | null = event.overKey;
      void receiving;
    }
  },
});

export const item = controller.registerItem({
  key: "alpha",
  element: () => null,
  label: () => "Alpha",
});

type _SortingIsReadonly = Expect<Equal<typeof controller.isSorting, Readonly<ShallowRef<boolean>>>>;
type _IndicatorIsNullable = Expect<
  Equal<typeof controller.indicator, Readonly<ShallowRef<SortableIndicatorState | null>>>
>;
type _ItemPropsAreExact = Expect<Equal<typeof item.itemProps, Readonly<DragSourceProps>>>;

export const vueAttributes: HTMLAttributes = item.itemProps;
// @ts-expect-error consumers cannot mutate readonly reactive state.
controller.isSorting.value = true;
// @ts-expect-error columns must resolve to a number.
createSortable({ columns: "2" });
// @ts-expect-error the direction union is closed.
createSortable({ direction: "auto" });
// @ts-expect-error item registration requires an element accessor.
controller.registerItem({ key: "floating" });

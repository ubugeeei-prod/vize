/** Compile-only assertions proving ListboxGrid value inference. */

import {
  ListboxGrid,
  ListboxGridItem,
  type GridMove,
  type ListboxGridItemSlotState,
  type ListboxGridModelValue,
  type ListboxGridState,
} from "./listbox-grid.ts";
import { moveInGrid } from "./listbox-grid-model.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Color {
  readonly hex: string;
  readonly name: string;
}

declare const colors: readonly Color[];
declare const color: Color;

type _Single = Expect<Equal<ListboxGridModelValue<Color, false>, Color | null>>;
type _Multiple = Expect<Equal<ListboxGridModelValue<Color, true>, readonly Color[]>>;
type _State = Expect<Equal<ListboxGridState, "disabled" | "empty" | "selected">>;
type _Moves = Expect<
  Equal<
    GridMove,
    | "down"
    | "first"
    | "last"
    | "left"
    | "page-down"
    | "page-up"
    | "right"
    | "row-end"
    | "row-start"
    | "up"
  >
>;

ListboxGrid({
  items: colors,
  by: "hex",
  "onUpdate:modelValue": (value) => {
    type _SingleEmit = Expect<Equal<typeof value, Color | null>>;
  },
});

ListboxGrid({
  items: colors,
  multiple: true,
  "onUpdate:modelValue": (value) => {
    type _MultipleEmit = Expect<Equal<typeof value, readonly Color[]>>;
  },
  formValue: (value) => value.hex,
});

// @ts-expect-error `by` keys must exist on the value type.
ListboxGrid({ items: colors, by: "rgb" });

// @ts-expect-error multiple mode requires an array model.
ListboxGrid({ items: colors, multiple: true, modelValue: color });

type ItemContext = NonNullable<ReturnType<typeof ListboxGridItem<Color>>["__ctx"]>;
type _ItemSlots = Expect<
  Equal<
    Parameters<NonNullable<ItemContext["slots"]["default"]>>[0],
    ListboxGridItemSlotState<Color>
  >
>;

const next: number | null = moveInGrid("down", { columns: 4, count: 10, index: 0 });
void next;

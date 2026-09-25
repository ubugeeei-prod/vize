/** Compile-only assertions proving TransferList value inference. */

import {
  TransferListItem,
  TransferListRoot,
  transferItems,
  type TransferListActionKind,
  type TransferListItemSlotState,
  type TransferListSlotState,
} from "./transfer-list.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Role {
  readonly slug: string;
  readonly title: string;
}

declare const roles: readonly Role[];

type _Actions = Expect<
  Equal<
    TransferListActionKind,
    | "move-all-to-source"
    | "move-all-to-target"
    | "move-selected-to-source"
    | "move-selected-to-target"
  >
>;

TransferListRoot({
  items: roles,
  by: "slug",
  itemText: (role) => role.title,
  filter: (role, query) => role.title.includes(query),
  "onUpdate:modelValue": (value) => {
    type _Target = Expect<Equal<typeof value, readonly Role[]>>;
  },
  onChange: (value, previous, moved, direction) => {
    type _Moved = Expect<Equal<typeof moved, readonly Role[]>>;
    type _Direction = Expect<Equal<typeof direction, "to-source" | "to-target">>;
    void value;
    void previous;
  },
});

// @ts-expect-error `by` keys must exist on the item type.
TransferListRoot({ items: roles, by: "id" });

// @ts-expect-error the model holds items of the same type.
TransferListRoot({ items: roles, modelValue: ["admin"] });

// @ts-expect-error items are required.
TransferListRoot({});

type RootContext = NonNullable<ReturnType<typeof TransferListRoot<Role>>["__ctx"]>;
type _RootSlots = Expect<
  Equal<Parameters<NonNullable<RootContext["slots"]["default"]>>[0], TransferListSlotState<Role>>
>;
type ItemContext = NonNullable<ReturnType<typeof TransferListItem<Role>>["__ctx"]>;
type _ItemSlots = Expect<
  Equal<
    Parameters<NonNullable<ItemContext["slots"]["default"]>>[0],
    TransferListItemSlotState<Role>
  >
>;

const result = transferItems<string>({
  direction: "to-target",
  equals: (left, right) => left === right,
  items: ["a"],
  max: 1,
  moving: ["a"],
  orderMode: "append",
  target: [],
});
type _Result = Expect<Equal<typeof result.moved, readonly string[]>>;

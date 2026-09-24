/** Compile-only assertions proving Select value inference. */

import {
  SelectItem,
  SelectRoot,
  SelectVirtualizer,
  fromSelectList,
  toSelectList,
  type SelectBy,
  type SelectItemSlotState,
  type SelectModelValue,
  type SelectPosition,
  type SelectRootExpose,
  type SelectSlotState,
  type SelectState,
  type SelectValueKey,
  type SelectVirtualItemSlotState,
} from "./select.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface User {
  readonly id: number;
  readonly name: string;
}

declare const users: readonly User[];
declare const user: User;

type _SingleModel = Expect<Equal<SelectModelValue<User, false>, User | null>>;
type _MultipleModel = Expect<Equal<SelectModelValue<User, true>, readonly User[]>>;
type _KeysOfObjects = Expect<Equal<SelectValueKey<User>, "id" | "name">>;
type _PrimitivesHaveNoKeys = Expect<Equal<SelectValueKey<string>, never>>;
type _ByAcceptsKeyOrComparator = Expect<
  Equal<SelectBy<User>, "id" | "name" | ((left: User, right: User) => boolean)>
>;
type _State = Expect<Equal<SelectState, "closed" | "open">>;
type _Position = Expect<Equal<SelectPosition, "item-aligned" | "popper">>;

// `items` decides T; omitting `multiple` selects the single `T | null` model.
SelectRoot({
  items: users,
  by: "id",
  itemText: (value) => {
    type _ItemTextReceivesT = Expect<Equal<typeof value, User>>;
    return value.name;
  },
  "onUpdate:modelValue": (value) => {
    type _SingleEmit = Expect<Equal<typeof value, User | null>>;
  },
  onChange: (value, previous, event) => {
    type _ChangeValue = Expect<Equal<typeof value, User | null>>;
    type _ChangePrevious = Expect<Equal<typeof previous, User | null>>;
    type _ChangeEvent = Expect<Equal<typeof event, Event>>;
  },
});

// `multiple: true` switches the model to a readonly array.
SelectRoot({
  items: users,
  multiple: true,
  defaultValue: [user],
  "onUpdate:modelValue": (value) => {
    type _MultipleEmit = Expect<Equal<typeof value, readonly User[]>>;
  },
});

// The model alone can infer T.
SelectRoot({
  modelValue: 3,
  "onUpdate:modelValue": (value) => {
    type _NumberEmit = Expect<Equal<typeof value, number | null>>;
  },
});

SelectRoot({ items: users, by: (left, right) => left.id === right.id });
SelectRoot({ items: ["a", "b"] as const, modelValue: "a" });

// @ts-expect-error `by` keys must exist on the value type.
SelectRoot({ items: users, by: "email" });

// @ts-expect-error multiple mode requires an array model.
SelectRoot({ items: users, multiple: true, modelValue: user });

// @ts-expect-error single mode rejects an array model.
SelectRoot({ items: users, multiple: false, modelValue: users });

// @ts-expect-error the model must match the items' value type.
SelectRoot({ items: users, modelValue: "ada" });

// @ts-expect-error primitive values have no comparison keys.
SelectRoot({ items: [1, 2, 3], by: "toFixed" });

type ItemContext = NonNullable<ReturnType<typeof SelectItem<User>>["__ctx"]>;
type _ItemValueProp = Expect<Equal<ItemContext["props"]["value"], User>>;
type _ItemSlotState = Expect<
  Equal<Parameters<NonNullable<ItemContext["slots"]["default"]>>[0], SelectItemSlotState<User>>
>;

type VirtualContext = NonNullable<ReturnType<typeof SelectVirtualizer<User>>["__ctx"]>;
type _VirtualSlot = Expect<
  Equal<
    Parameters<NonNullable<VirtualContext["slots"]["default"]>>[0],
    SelectVirtualItemSlotState<User>
  >
>;

type RootContext = NonNullable<ReturnType<typeof SelectRoot<User, true>>["__ctx"]>;
type _RootSlotState = Expect<
  Equal<Parameters<NonNullable<RootContext["slots"]["default"]>>[0], SelectSlotState<User>>
>;

declare const exposed: SelectRootExpose<User>;
type _ExposeSelected = Expect<Equal<typeof exposed.selected, readonly User[]>>;
type _ExposeSelect = Expect<Equal<Parameters<typeof exposed.select>[0], User>>;

// @ts-expect-error SelectItem requires a value.
SelectItem({});

const single: User | null = fromSelectList<User, false>([user], false);
const many: readonly User[] = fromSelectList<User, true>([user], true);
const list: readonly User[] = toSelectList<User>(user, false, Object.is);

void single;
void many;
void list;

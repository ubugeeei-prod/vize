/** Compile-only assertions proving Cascader path inference. */

import {
  CascaderItem,
  CascaderRoot,
  findCascaderPath,
  type CascaderColumnState,
  type CascaderItemSlotState,
  type CascaderModelValue,
  type CascaderSlotState,
} from "./cascader.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Category {
  readonly slug: string;
  readonly title: string;
  readonly children?: readonly Category[];
}

declare const categories: readonly Category[];
declare const category: Category;

type _SinglePath = Expect<Equal<CascaderModelValue<Category, false>, readonly Category[]>>;
type _MultiplePaths = Expect<
  Equal<CascaderModelValue<Category, true>, readonly (readonly Category[])[]>
>;

CascaderRoot({
  options: categories,
  by: "slug",
  itemText: (node) => node.title,
  "onUpdate:modelValue": (value) => {
    type _Single = Expect<Equal<typeof value, readonly Category[]>>;
  },
  loadChildren: async (node, { signal }) => {
    type _Node = Expect<Equal<typeof node, Category>>;
    type _Signal = Expect<Equal<typeof signal, AbortSignal>>;
    return node.children ?? [];
  },
});

CascaderRoot({
  options: categories,
  multiple: true,
  defaultValue: [[category]],
  "onUpdate:modelValue": (value) => {
    type _Multiple = Expect<Equal<typeof value, readonly (readonly Category[])[]>>;
  },
});

// @ts-expect-error `by` keys must exist on the node type.
CascaderRoot({ options: categories, by: "id" });

// @ts-expect-error single mode takes one path, not a list of paths.
CascaderRoot({ options: categories, modelValue: [[category]] });

// @ts-expect-error loaders must resolve nodes of the same type.
CascaderRoot({ options: categories, loadChildren: async () => ["x"] });

// @ts-expect-error options are required.
CascaderRoot({});

type RootContext = NonNullable<ReturnType<typeof CascaderRoot<Category>>["__ctx"]>;
type RootSlot = Parameters<NonNullable<RootContext["slots"]["default"]>>[0];
type _RootSlot = Expect<Equal<RootSlot, CascaderSlotState<Category>>>;
type _Columns = Expect<Equal<RootSlot["columns"], readonly CascaderColumnState<Category>[]>>;

type ItemContext = NonNullable<ReturnType<typeof CascaderItem<Category>>["__ctx"]>;
type _ItemSlot = Expect<
  Equal<
    Parameters<NonNullable<ItemContext["slots"]["default"]>>[0],
    CascaderItemSlotState<Category>
  >
>;

const found = findCascaderPath(categories, (node) => node.slug === "shoes");
type _Found = Expect<Equal<typeof found, readonly Category[] | null>>;

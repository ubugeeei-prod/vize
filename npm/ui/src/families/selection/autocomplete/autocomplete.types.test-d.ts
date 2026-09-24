/** Compile-only assertions proving Autocomplete value inference. */

import {
  AutocompleteRoot,
  createWebStorageHistory,
  type AutocompleteHistoryStorage,
  type AutocompleteSlotState,
} from "./autocomplete.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Address {
  readonly placeId: string;
  readonly formatted: string;
}

declare const addresses: readonly Address[];

AutocompleteRoot({
  loadItems: async (query, { signal }) => {
    void query;
    void signal;
    return addresses;
  },
  by: "placeId",
  itemText: (value) => value.formatted,
  "onUpdate:modelValue": (value) => {
    type _Chosen = Expect<Equal<typeof value, Address | null>>;
  },
  "onUpdate:history": (history) => {
    type _History = Expect<Equal<typeof history, readonly Address[]>>;
  },
  onSubmit: (text, value) => {
    type _Text = Expect<Equal<typeof text, string>>;
    type _Value = Expect<Equal<typeof value, Address | null>>;
  },
});

AutocompleteRoot({ items: ["vue", "vite"], fromText: (text) => text });

// @ts-expect-error fromText must build the item type.
AutocompleteRoot({ items: addresses, fromText: (text: string) => text });

// @ts-expect-error `by` keys must exist on the item type.
AutocompleteRoot({ items: addresses, by: "id" });

const storage: AutocompleteHistoryStorage<Address> = createWebStorageHistory<Address>({
  key: "recent-addresses",
  parse: () => null,
});
void storage;

type RootContext = NonNullable<ReturnType<typeof AutocompleteRoot<Address>>["__ctx"]>;
type _Slots = Expect<
  Equal<Parameters<NonNullable<RootContext["slots"]["default"]>>[0], AutocompleteSlotState<Address>>
>;

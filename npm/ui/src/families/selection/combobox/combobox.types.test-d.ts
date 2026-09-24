/** Compile-only assertions proving Combobox value inference. */

import {
  ComboboxChip,
  ComboboxRoot,
  containsComboboxFilter,
  type ComboboxAutocomplete,
  type ComboboxChipSlotState,
  type ComboboxFilter,
  type ComboboxLoadStatus,
  type ComboboxModelValue,
  type ComboboxRootExpose,
  type ComboboxSlotState,
} from "./combobox.ts";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends <Value>() => Value extends Right ? 1 : 2
    ? true
    : false;
type Expect<Condition extends true> = Condition;

interface Repo {
  readonly id: number;
  readonly fullName: string;
}

declare const repos: readonly Repo[];
declare const repo: Repo;

type _Autocomplete = Expect<Equal<ComboboxAutocomplete, "both" | "inline" | "list" | "none">>;
type _Status = Expect<Equal<ComboboxLoadStatus, "error" | "idle" | "loading" | "success">>;
type _Model = Expect<Equal<ComboboxModelValue<Repo, true>, readonly Repo[]>>;

ComboboxRoot({
  items: repos,
  by: "id",
  filter: (value, query, text) => {
    type _FilterValue = Expect<Equal<typeof value, Repo>>;
    return text.includes(query);
  },
  "onUpdate:modelValue": (value) => {
    type _Single = Expect<Equal<typeof value, Repo | null>>;
  },
  "onUpdate:inputValue": (text) => {
    type _Text = Expect<Equal<typeof text, string>>;
  },
});

// The loader decides T and `multiple: true` switches the model to an array.
ComboboxRoot({
  multiple: true,
  loadItems: async (query, { signal }) => {
    type _Signal = Expect<Equal<typeof signal, AbortSignal>>;
    void query;
    return repos;
  },
  "onUpdate:modelValue": (value) => {
    type _Multiple = Expect<Equal<typeof value, readonly Repo[]>>;
  },
});

// createOption must build the value type from text, and `create` reports it.
ComboboxRoot({
  items: repos,
  createOption: (text) => ({ id: -1, fullName: text }),
  onCreate: (value, text) => {
    type _Created = Expect<Equal<typeof value, Repo>>;
    type _CreatedText = Expect<Equal<typeof text, string>>;
  },
});

ComboboxRoot({ items: ["a", "b"], filter: false, autocomplete: "both", strict: false });
ComboboxRoot({ items: repos, filter: containsComboboxFilter });

// @ts-expect-error createOption must return the item type.
ComboboxRoot({ items: repos, createOption: (text: string) => text });

// @ts-expect-error filter functions receive the item type.
ComboboxRoot({ items: repos, filter: (value: string) => value.length > 0 });

// @ts-expect-error multiple mode requires an array model.
ComboboxRoot({ items: repos, multiple: true, modelValue: repo });

// @ts-expect-error autocomplete is a closed union.
ComboboxRoot({ items: repos, autocomplete: "fuzzy" });

type RootContext = NonNullable<ReturnType<typeof ComboboxRoot<Repo, true>>["__ctx"]>;
type _RootSlots = Expect<
  Equal<Parameters<NonNullable<RootContext["slots"]["default"]>>[0], ComboboxSlotState<Repo>>
>;

type ChipContext = NonNullable<ReturnType<typeof ComboboxChip<Repo>>["__ctx"]>;
type _ChipSlots = Expect<
  Equal<Parameters<NonNullable<ChipContext["slots"]["default"]>>[0], ComboboxChipSlotState<Repo>>
>;

declare const exposed: ComboboxRootExpose<Repo>;
type _ExposeSelected = Expect<Equal<typeof exposed.selected, readonly Repo[]>>;

const filter: ComboboxFilter<Repo> = (value, query) => value.fullName.startsWith(query);
void filter;

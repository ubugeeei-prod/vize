import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { createContext } from "../../../context.ts";
import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import type { CompositeItemProps } from "../../foundations/composite-navigation/composite-navigation.ts";
import type {
  ListboxDirection,
  ListboxOrientation,
  ListboxSelectionMode,
  ListboxState,
  ListboxValue,
} from "./listbox-types.ts";

export interface ListboxCollectionValue {
  readonly id: ComputedRef<string>;
  readonly value: string;
}

export interface ListboxItemRegistrationInput {
  readonly value: string;
  readonly id: ComputedRef<string>;
  readonly element?: MaybeRefOrGetter<Element | null | undefined>;
  readonly textValue?: MaybeRefOrGetter<string | null | undefined>;
  readonly disabled?: MaybeRefOrGetter<boolean | undefined>;
  readonly order?: MaybeRefOrGetter<number | undefined>;
}

/** Shared state and collection hooks for Listbox compound items. */
export interface ListboxContextValue {
  readonly id: ComputedRef<string>;
  readonly value: ComputedRef<ListboxValue>;
  readonly selectedValues: ComputedRef<ReadonlySet<string>>;
  readonly activeValue: ComputedRef<string | null>;
  readonly disabled: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly invalid: ComputedRef<boolean>;
  readonly selectionMode: ComputedRef<ListboxSelectionMode>;
  readonly orientation: ComputedRef<ListboxOrientation>;
  readonly direction: ComputedRef<ListboxDirection>;
  readonly state: ComputedRef<ListboxState>;
  readonly registerItem: (input: ListboxItemRegistrationInput) => CollectionRegistration<string>;
  readonly getItemProps: (value: string) => Readonly<CompositeItemProps>;
  readonly setActiveValue: (value: string | null) => boolean;
  readonly selectValue: (value: string, nativeEvent: Event | null) => boolean;
  readonly toggleValue: (value: string, nativeEvent: Event | null) => boolean;
  readonly focus: (options?: FocusOptions) => void;
}

export const listboxContext = createContext<ListboxContextValue>("Listbox");

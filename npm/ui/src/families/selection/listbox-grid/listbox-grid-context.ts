import type { ComputedRef, MaybeRefOrGetter } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistration } from "../../foundations/collection/collection.ts";

/** Registration input for one grid option. */
export interface ListboxGridItemRegistration<T> {
  readonly id: ComputedRef<string>;
  readonly value: T;
  readonly element: MaybeRefOrGetter<Element | null | undefined>;
  readonly textValue: MaybeRefOrGetter<string | null | undefined>;
  readonly disabled: MaybeRefOrGetter<boolean | undefined>;
}

/**
 * Shared state for ListboxGrid options. Value-accepting members are methods
 * so a root typed over `T` stays assignable to the `unknown` context.
 */
export interface ListboxGridContextValue<T> {
  readonly activeId: ComputedRef<string | null>;
  readonly columns: ComputedRef<number>;
  readonly disabled: ComputedRef<boolean>;
  readonly multiple: ComputedRef<boolean>;
  register(input: ListboxGridItemRegistration<T>): CollectionRegistration<string>;
  indexOf(id: string): number;
  isSelected(value: T): boolean;
  activate(id: string): void;
  choose(value: T, event: Event | null): boolean;
  focus(): void;
}

export const listboxGridContext = createContext<ListboxGridContextValue<unknown>>("ListboxGrid");

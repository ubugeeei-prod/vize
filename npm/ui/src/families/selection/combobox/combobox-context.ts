import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import type { ComboboxAutocomplete, ComboboxLoadStatus } from "./combobox-types.ts";

/**
 * Combobox-only state layered on the shared Select popup context.
 *
 * Structural popup parts (content, item, group, label, separator, viewport,
 * virtualizer, indicator) read the Select context that `ComboboxRoot` also
 * provides; the parts below read this context for input, chip, and status
 * behavior. Value-accepting members are methods for the same variance reason
 * documented on the Select context.
 */
export interface ComboboxContextValue<T> {
  readonly inputId: ComputedRef<string>;
  readonly listboxId: ComputedRef<string>;
  readonly inputValue: ComputedRef<string>;
  readonly query: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
  readonly readonly: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly invalid: ComputedRef<boolean>;
  readonly multiple: ComputedRef<boolean>;
  readonly autocomplete: ComputedRef<ComboboxAutocomplete>;
  readonly activeDescendant: ComputedRef<string | undefined>;
  readonly empty: ComputedRef<boolean>;
  readonly status: ComputedRef<ComboboxLoadStatus>;
  readonly canCreate: ComputedRef<boolean>;
  readonly createActive: ComputedRef<boolean>;
  readonly inputElement: ShallowRef<HTMLInputElement | null>;
  readonly anchorElement: ShallowRef<HTMLElement | null>;
  readonly toggleElement: ShallowRef<HTMLElement | null>;
  readonly labelledby: ShallowRef<string | undefined>;
  readonly nativeRequired: ComputedRef<boolean>;
  onInput(event: Event): void;
  onKeydown(event: KeyboardEvent): void;
  onFocus(event: FocusEvent): void;
  onBlur(event: FocusEvent): void;
  setOpen(open: boolean, event: Event | null): boolean;
  remove(value: T, event?: Event | null): boolean;
  textOf(value: T): string;
  isValueDisabled(value: T): boolean;
  create(event: Event | null): boolean;
  registerCreateOption(input: {
    readonly id: ComputedRef<string>;
    readonly element: () => Element | null;
  }): CollectionRegistration<string>;
  highlightCreateOption(): void;
}

export const comboboxContext = createContext<ComboboxContextValue<unknown>>("Combobox");

/** State shared by one `ComboboxChip` with its remove button. */
export interface ComboboxChipContextValue {
  readonly text: ComputedRef<string>;
  readonly disabled: ComputedRef<boolean>;
  remove(event?: Event | null): boolean;
}

export const comboboxChipContext = createContext<ComboboxChipContextValue>("ComboboxChip");

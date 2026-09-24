import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import type { SelectOptionRegistration, SelectVirtualAdapter } from "./select-collection.ts";
import type { SelectDirection, SelectState } from "./select-types.ts";

/**
 * Family owning a popup part. Structural parts are shared by Select and
 * Combobox and publish `data-vize-ui="<prefix>-<part>"` for their owner.
 */
export type SelectPartPrefix = "combobox" | "select";

/**
 * Shared state for Select compound parts.
 *
 * The root is generic over its value type `T`; parts are generic
 * independently, so values cross this boundary as `unknown` and are compared
 * with the root-owned equality. Members that accept values are declared as
 * methods so a root typed over `T` stays assignable to the `unknown`
 * context without casts; every value a part hands back originated from the
 * root's model, its `items`, or a part authored for that root.
 */
export interface SelectContextValue<T> {
  readonly partPrefix: SelectPartPrefix;
  readonly baseId: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly listboxId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<SelectState>;
  readonly disabled: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly invalid: ComputedRef<boolean>;
  readonly multiple: ComputedRef<boolean>;
  readonly direction: ComputedRef<SelectDirection>;
  readonly placeholder: ComputedRef<string | undefined>;
  readonly selected: ComputedRef<readonly T[]>;
  readonly selectedText: ComputedRef<readonly string[]>;
  readonly activeDescendant: ComputedRef<string | undefined>;
  readonly items: ComputedRef<readonly T[] | undefined>;
  readonly activeKey: ComputedRef<string | null>;
  readonly triggerElement: ShallowRef<HTMLElement | null>;
  readonly contentElement: ShallowRef<HTMLElement | null>;
  readonly viewportElement: ShallowRef<HTMLElement | null>;
  readonly referenceElement: ComputedRef<Element | null>;
  readonly listboxLabelledby: ComputedRef<string | undefined>;
  readonly dismissBranches: ComputedRef<readonly Element[]>;
  isSelected(value: T): boolean;
  isItemVisible(value: T, text: string): boolean;
  choose(value: T, event: Event | null): boolean;
  setOpen(open: boolean, event: Event | null): boolean;
  onTriggerKeydown(event: KeyboardEvent): void;
  registerItem(input: SelectOptionRegistration<T>): CollectionRegistration<string>;
  highlight(key: string | null): boolean;
  setVirtualAdapter(adapter: SelectVirtualAdapter | null): () => void;
  rememberText(value: T, text: string): void;
  textOf(value: T): string;
  isValueDisabled(value: T): boolean;
  anchorElement(): Element | null;
}

export const selectContext = createContext<SelectContextValue<unknown>>("Select");

/** State shared by one `SelectItem` with its indicator. */
export interface SelectItemContextValue {
  readonly selected: ComputedRef<boolean>;
  readonly active: ComputedRef<boolean>;
  readonly disabled: ComputedRef<boolean>;
}

export const selectItemContext = createContext<SelectItemContextValue>("SelectItem");

/** Label wiring shared by one `SelectGroup` with its `SelectLabel`. */
export interface SelectGroupContextValue {
  readonly labelId: ComputedRef<string>;
}

export const selectGroupContext = createContext<SelectGroupContextValue>("SelectGroup");

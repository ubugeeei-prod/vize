import type { ComputedRef, ShallowRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { CascaderColumnState, CascaderState } from "./cascader-types.ts";

/**
 * Shared state for Cascader parts.
 *
 * The root is generic over its node type `T`; parts are generic
 * independently, so nodes cross this boundary as `unknown` and are compared
 * with the root-owned equality. Node-accepting members are methods so a root
 * typed over `T` stays assignable to the `unknown` context without casts.
 */
export interface CascaderContextValue<T> {
  readonly baseId: ComputedRef<string>;
  readonly triggerId: ComputedRef<string>;
  readonly contentId: ComputedRef<string>;
  readonly open: ComputedRef<boolean>;
  readonly state: ComputedRef<CascaderState>;
  readonly disabled: ComputedRef<boolean>;
  readonly required: ComputedRef<boolean>;
  readonly multiple: ComputedRef<boolean>;
  readonly placeholder: ComputedRef<string | undefined>;
  readonly selectedText: ComputedRef<readonly string[]>;
  readonly activeDescendant: ComputedRef<string | undefined>;
  readonly columnIds: ComputedRef<readonly string[]>;
  readonly columns: ComputedRef<readonly CascaderColumnState<T>[]>;
  readonly triggerElement: ShallowRef<HTMLElement | null>;
  readonly contentElement: ShallowRef<HTMLElement | null>;
  setOpen(open: boolean, event: Event | null): boolean;
  onTriggerKeydown(event: KeyboardEvent): void;
  columnId(level: number): string;
  optionId(level: number, value: T): string;
  textOf(value: T): string;
  isBranch(value: T): boolean;
  isLoading(value: T): boolean;
  isDisabled(value: T): boolean;
  isExpanded(level: number, value: T): boolean;
  isActive(level: number, value: T): boolean;
  isSelectedEnd(level: number, value: T): boolean;
  isPartial(level: number, value: T): boolean;
  pressItem(level: number, value: T, event: Event): void;
  hoverItem(level: number, value: T): void;
}

export const cascaderContext = createContext<CascaderContextValue<unknown>>("Cascader");

/** Level shared by one `CascaderColumn` with its items. */
export interface CascaderColumnContextValue {
  readonly level: ComputedRef<number>;
}

export const cascaderColumnContext = createContext<CascaderColumnContextValue>("CascaderColumn");

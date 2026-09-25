import type { ComputedRef } from "vue";

import { createContext } from "../../foundations/context/context.ts";
import type { TransferListAction, TransferListSide } from "./transfer-list-model.ts";

/** Per-side state shared with panels, search fields, and items. */
export interface TransferListSideState<T> {
  readonly all: ComputedRef<readonly T[]>;
  readonly visible: ComputedRef<readonly T[]>;
  readonly checked: ComputedRef<readonly T[]>;
  readonly query: ComputedRef<string>;
}

/**
 * Shared TransferList state. Value-accepting members are methods so a root
 * typed over `T` stays assignable to the `unknown` context.
 */
export interface TransferListContextValue<T> {
  readonly baseId: ComputedRef<string>;
  panelId(side: TransferListSide): string;
  readonly disabled: ComputedRef<boolean>;
  readonly full: ComputedRef<boolean>;
  side(side: TransferListSide): TransferListSideState<T>;
  isChecked(side: TransferListSide, value: T): boolean;
  isItemDisabled(value: T): boolean;
  textOf(value: T): string;
  setChecked(side: TransferListSide, values: readonly T[]): void;
  toggleChecked(side: TransferListSide, value: T): void;
  setQuery(side: TransferListSide, query: string): void;
  canRun(action: TransferListAction): boolean;
  run(action: TransferListAction, event: Event | null): readonly T[];
  moveValues(side: TransferListSide, values: readonly T[], event: Event | null): readonly T[];
  registerPanel(side: TransferListSide, focus: () => void): () => void;
  focusPanel(side: TransferListSide): void;
}

export const transferListContext = createContext<TransferListContextValue<unknown>>("TransferList");

/** Per-panel state shared with items and the empty state. */
export interface TransferListPanelContextValue {
  readonly side: TransferListSide;
  readonly activeKey: ComputedRef<string | null>;
  readonly empty: ComputedRef<boolean>;
  readonly query: ComputedRef<string>;
  register(input: {
    readonly id: ComputedRef<string>;
    readonly value: unknown;
    readonly element: () => Element | null;
    readonly textValue: () => string;
    readonly disabled: () => boolean;
  }): () => void;
  activate(key: string): void;
  focus(): void;
}

export const transferListPanelContext =
  createContext<TransferListPanelContextValue>("TransferListPanel");

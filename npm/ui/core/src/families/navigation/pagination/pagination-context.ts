import type { ComputedRef } from "vue";

import { createContext } from "../../../context.ts";
import type {
  PaginationControlState,
  PaginationPageState,
  PaginationState,
} from "./pagination-types.ts";
import type { PaginationRangeItem } from "./pagination-range.ts";

/** Shared state and actions for the Pagination compound components. */
export interface PaginationContextValue {
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly page: ComputedRef<number>;
  readonly pageCount: ComputedRef<number>;
  readonly disabled: ComputedRef<boolean>;
  readonly state: ComputedRef<PaginationState>;
  readonly range: ComputedRef<readonly PaginationRangeItem[]>;
  readonly previousPage: ComputedRef<number | null>;
  readonly nextPage: ComputedRef<number | null>;
  readonly canPrevious: ComputedRef<boolean>;
  readonly canNext: ComputedRef<boolean>;
  readonly previousId: ComputedRef<string>;
  readonly nextId: ComputedRef<string>;
  readonly getPageId: (page: number) => string;
  readonly getPageLabel: (page: number, current: boolean) => string;
  readonly getPageState: (page: number, disabled: boolean) => PaginationPageState;
  readonly getPreviousState: (disabled: boolean) => PaginationControlState;
  readonly getNextState: (disabled: boolean) => PaginationControlState;
  readonly setPage: (page: number, event?: Event | null) => boolean;
  readonly goPrevious: (event?: Event | null) => boolean;
  readonly goNext: (event?: Event | null) => boolean;
  readonly focusCurrent: (options?: FocusOptions) => void;
}

export const paginationContext = createContext<PaginationContextValue>("Pagination");

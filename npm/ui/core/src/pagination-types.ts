import type { PrimitiveElement } from "./primitive.ts";
import type { PaginationRangeItem } from "./pagination-range.ts";

/** State exposed by the Pagination root, list, and root slot. */
export type PaginationState = "active" | "disabled";

/** State exposed by numbered page controls and page-aware list items. */
export type PaginationPageState = "current" | "disabled" | "idle";

/** State exposed by previous and next controls. */
export type PaginationControlState = "disabled" | "idle";

/** Position exposed by a non-interactive pagination ellipsis. */
export type PaginationEllipsisPosition = "start" | "end";

/** Public props accepted by the Pagination root primitive. */
export interface PaginationRootProps {
  /**
   * Consumer-owned pagination id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /** Total number of pages. Runtime values below one resolve to one page. @default required */
  readonly pageCount: number;

  /**
   * Controlled current page. `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: number;

  /**
   * Initial page for uncontrolled use and the page restored by reset.
   *
   * @default 1
   */
  readonly defaultValue?: number;

  /**
   * Disable all page controls.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible landmark label mirrored to `aria-label`.
   *
   * @default "Pagination"
   */
  readonly label?: string;

  /**
   * Number of pages kept on each side of the current page in `range`.
   *
   * @default 1
   */
  readonly siblingCount?: number;

  /**
   * Number of pages always kept at each boundary in `range`.
   *
   * @default 1
   */
  readonly boundaryCount?: number;
}

/** State exposed by the Pagination root default slot. */
export interface PaginationSlotState {
  /** Current page clamped into the available page range. */
  readonly page: number;

  /** Total page count normalized to at least one page. */
  readonly pageCount: number;

  /** Whether every page control is disabled by the root. */
  readonly disabled: boolean;

  /** Whether a previous page exists and controls are enabled. */
  readonly canPrevious: boolean;

  /** Whether a next page exists and controls are enabled. */
  readonly canNext: boolean;

  /** Previous page number, or `null` while on the first page. */
  readonly previousPage: number | null;

  /** Next page number, or `null` while on the last page. */
  readonly nextPage: number | null;

  /** Deterministic compact range for rendering page and ellipsis items. */
  readonly range: readonly PaginationRangeItem[];

  /** Stable state token for styling and tests. */
  readonly state: PaginationState;
}

/** State exposed by PaginationList's default slot. */
export interface PaginationListSlotState extends PaginationSlotState {
  /** Deterministic id assigned to the list. */
  readonly listId: string;
}

/** State exposed by PaginationItem's default slot. */
export interface PaginationItemSlotState {
  /** Page represented by the item, or `undefined` for structural items. */
  readonly page: number | undefined;

  /** Whether the item represents the current page. */
  readonly current: boolean;

  /** Whether the item or root is disabled. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: PaginationPageState;
}

/** State exposed by a numbered PaginationPage control. */
export interface PaginationPageSlotState {
  /** Page selected by this control. */
  readonly page: number;

  /** Whether this control represents the current page. */
  readonly current: boolean;

  /** Whether this control is disabled by itself, the root, or page bounds. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: PaginationPageState;
}

/** State exposed by PaginationPrevious and PaginationNext controls. */
export interface PaginationControlSlotState {
  /** Target page selected by this control, or `null` at a boundary. */
  readonly targetPage: number | null;

  /** Whether this control is disabled by itself, the root, or page bounds. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: PaginationControlState;
}

/** State exposed by PaginationEllipsis. */
export interface PaginationEllipsisSlotState {
  /** Ellipsis position relative to the current compact range. */
  readonly position: PaginationEllipsisPosition;

  /** Ellipses are never interactive. */
  readonly disabled: true;
}

/** Public instance exposed by PaginationRoot. */
export interface PaginationRootExpose extends PaginationSlotState {
  /** Rendered landmark element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Root-owned base id for the Pagination family. */
  readonly id: string;

  /** Id wired to PaginationList. */
  readonly listId: string;

  /** Move focus to the current page control when it is rendered. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a current-page update and report whether it differs. */
  readonly setPage: (page: number, event?: Event | null) => boolean;

  /** Request the previous page and report whether it differs. */
  readonly goPrevious: (event?: Event | null) => boolean;

  /** Request the next page and report whether it differs. */
  readonly goNext: (event?: Event | null) => boolean;

  /** Restore the default page and report whether it differs. */
  readonly reset: () => boolean;
}

/** Public instance exposed by PaginationList. */
export interface PaginationListExpose extends PaginationListSlotState {
  /** Rendered list element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Move focus to the current page control when it is rendered. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by PaginationItem. */
export interface PaginationItemExpose extends PaginationItemSlotState {
  /** Rendered list item element or component instance. */
  readonly element: PrimitiveElement | null;
}

/** Public instance exposed by PaginationPage. */
export interface PaginationPageExpose extends PaginationPageSlotState {
  /** Rendered native page button. */
  readonly element: HTMLButtonElement | null;

  /** Deterministic id assigned to the page control. */
  readonly id: string;

  /** Move focus to the page button. */
  readonly focus: (options?: FocusOptions) => void;

  /** Select this page and report whether the current page changed. */
  readonly select: () => boolean;
}

/** Public instance exposed by PaginationPrevious and PaginationNext. */
export interface PaginationControlExpose extends PaginationControlSlotState {
  /** Rendered native navigation button. */
  readonly element: HTMLButtonElement | null;

  /** Deterministic id assigned to the control. */
  readonly id: string;

  /** Move focus to the navigation button. */
  readonly focus: (options?: FocusOptions) => void;

  /** Select the target page and report whether the current page changed. */
  readonly select: () => boolean;
}

/** Public instance exposed by PaginationEllipsis. */
export interface PaginationEllipsisExpose extends PaginationEllipsisSlotState {
  /** Rendered ellipsis element or component instance. */
  readonly element: PrimitiveElement | null;
}

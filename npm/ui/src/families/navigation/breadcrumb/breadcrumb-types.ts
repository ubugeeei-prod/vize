import type { PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Values accepted by `aria-current` for route-aware breadcrumb links. */
export type BreadcrumbCurrent = "date" | "location" | "page" | "step" | "time" | true;

/** State exposed by the Breadcrumb root default slot. */
export interface BreadcrumbRootSlotState {
  /** Accessible landmark label mirrored to `aria-label`. */
  readonly label: string;
}

/** State exposed by BreadcrumbItem's default slot. */
export interface BreadcrumbItemSlotState {
  /** Whether this breadcrumb item represents the current page, step, or location. */
  readonly current: boolean;
}

/** State exposed by BreadcrumbLink's default slot. */
export interface BreadcrumbLinkSlotState {
  /** Whether this link represents the current page, step, or location. */
  readonly current: boolean;

  /** Resolved `aria-current` value, or `undefined` when the link is not current. */
  readonly ariaCurrent: Exclude<BreadcrumbCurrent, true> | undefined;
}

/** State exposed by BreadcrumbSeparator's default slot. */
export interface BreadcrumbSeparatorSlotState {
  /** Separators are always decorative and hidden from assistive technology. */
  readonly decorative: true;
}

/** Public component instance state exposed by Breadcrumb. */
export interface BreadcrumbRootExpose extends BreadcrumbRootSlotState {
  /** Rendered landmark element or component instance. */
  readonly element: PrimitiveElement | null;
}

/** Public component instance state exposed by BreadcrumbList. */
export interface BreadcrumbListExpose {
  /** Rendered list element or component instance. */
  readonly element: PrimitiveElement | null;
}

/** Public component instance state exposed by BreadcrumbItem. */
export interface BreadcrumbItemExpose extends BreadcrumbItemSlotState {
  /** Rendered list item element or component instance. */
  readonly element: PrimitiveElement | null;
}

/** Public component instance state exposed by BreadcrumbLink. */
export interface BreadcrumbLinkExpose extends BreadcrumbLinkSlotState {
  /** Rendered link element or component instance. */
  readonly element: PrimitiveElement | null;

  /** Focus the rendered link when it is a native HTMLElement. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public component instance state exposed by BreadcrumbSeparator. */
export interface BreadcrumbSeparatorExpose extends BreadcrumbSeparatorSlotState {
  /** Rendered separator element or component instance. */
  readonly element: PrimitiveElement | null;
}

import type { ComponentPublicInstance } from "vue";

/** Orientation announced by {@link Separator} and exposed through data hooks. */
export type SeparatorOrientation = "horizontal" | "vertical";

/** Rendered value exposed by {@link Separator}. */
export type SeparatorElement = Element | ComponentPublicInstance;

/** Public instance state exposed by the separator component. */
export interface SeparatorExpose {
  /** Rendered host element or component instance. */
  readonly element: SeparatorElement | null;

  /** Logical axis announced by ARIA and `data-orientation`. */
  readonly orientation: SeparatorOrientation;

  /** Whether the separator is hidden from the accessibility tree. */
  readonly decorative: boolean;
}

import type { ComponentPublicInstance } from "vue";

/** Named max-inline-size preset supported by {@link Container}. */
export type ContainerSize = "xs" | "sm" | "md" | "lg" | "xl" | "full";

/** Native CSS length, percentage, keyword, or custom property accepted by {@link Container}. */
export type ContainerLength = string | number;

/** CSS-ready value published after numeric lengths are normalized. */
export type ContainerResolvedLength = string;

/** Rendered value exposed by {@link Container}. */
export type ContainerElement = Element | ComponentPublicInstance;

/** Inline style hooks applied to the rendered Container host. */
export interface ContainerStyle {
  /** Consumer-overridable max inline size hook read by the host declaration. */
  readonly "--vize-ui-container-max-inline-size": ContainerResolvedLength;

  /** Consumer-overridable inline padding hook read by the host declaration. */
  readonly "--vize-ui-container-padding-inline": ContainerResolvedLength;

  /** Native logical max inline size declaration. */
  readonly maxInlineSize: string;

  /** Native logical inline padding declaration. */
  readonly paddingInline: string;

  /** Native logical margin declaration applied only when centering is enabled. */
  readonly marginInline?: "auto";
}

/** State exposed to the default Container slot. */
export interface ContainerSlotState {
  /** Resolved named max-inline-size preset. */
  readonly size: ContainerSize;

  /** Resolved CSS max inline size value. */
  readonly maxInlineSize: ContainerResolvedLength;

  /** Resolved CSS inline padding value. */
  readonly paddingInline: ContainerResolvedLength;

  /** Whether the host uses logical auto margins. */
  readonly centered: boolean;

  /** Native logical style object applied to the host. */
  readonly style: ContainerStyle;
}

/** Resolved layout state published by {@link Container}. */
export type ContainerResolvedLayout = ContainerSlotState;

/** Public instance state exposed by the Container primitive. */
export interface ContainerExpose extends ContainerSlotState {
  /** Rendered host element or component instance. */
  readonly element: ContainerElement | null;
}

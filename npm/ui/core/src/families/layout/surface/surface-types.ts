import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Common semantic native hosts documented for Surface examples. */
export type SurfaceSemanticHost = "article" | "aside" | "div" | "section";

/** Native element, custom element, or component accepted by the Surface primitive. */
export type SurfaceAs = PrimitiveAs;

/** Consumer tone hooks mirrored by {@link Surface} through optional `data-tone`. */
export type SurfaceTone =
  | "accent"
  | "danger"
  | "info"
  | "muted"
  | "neutral"
  | "success"
  | "warning";

/** Shared elevation roles mirrored by {@link Surface} through optional `data-elevation`. */
export type SurfaceElevation = "floating" | "overlay" | "raised";

/** Rendered value exposed by {@link Surface}. */
export type SurfaceElement = PrimitiveElement;

/** Public props accepted by the Surface primitive. */
export interface SurfaceProps {
  /**
   * Native semantic host rendered by the primitive.
   *
   * @default "section"
   */
  readonly as?: SurfaceAs;

  /**
   * Space-separated ids that label the surface.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the surface.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Optional consumer tone hook mirrored to `data-tone`; no CSS is emitted.
   *
   * @default undefined
   */
  readonly tone?: SurfaceTone;

  /**
   * Optional consumer elevation hook mirrored to `data-elevation`; no CSS is emitted.
   *
   * @default undefined
   */
  readonly elevation?: SurfaceElevation;
}

/** Normalized ARIA IDREF state rendered by Surface. */
export interface SurfaceAriaState {
  /** Normalized `aria-labelledby` value, or `undefined` when absent. */
  readonly ariaLabelledby: string | undefined;

  /** Normalized `aria-describedby` value, or `undefined` when absent. */
  readonly ariaDescribedby: string | undefined;
}

/** State exposed to the default Surface slot. */
export interface SurfaceSlotState extends SurfaceAriaState {
  /** Rendered semantic host. */
  readonly as: SurfaceAs;

  /** Optional consumer tone hook. */
  readonly tone: SurfaceTone | undefined;

  /** Optional consumer elevation hook. */
  readonly elevation: SurfaceElevation | undefined;

  /** Whether the surface has an accessible labeling reference. */
  readonly labelled: boolean;

  /** Whether the surface has an accessible description reference. */
  readonly described: boolean;
}

/** Public component instance state exposed by the Surface primitive. */
export interface SurfaceExpose extends SurfaceSlotState {
  /** Rendered host element. */
  readonly element: SurfaceElement | null;
}

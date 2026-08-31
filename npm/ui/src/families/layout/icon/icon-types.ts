import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Accessibility state emitted by the SVG icon root. */
export type IconAriaState = "decorative" | "img";

/** Consumer styling token mirrored to `data-size`; no CSS is emitted. */
export type IconSize = "xs" | "sm" | "md" | "lg" | "xl";

/** Root SVG paint tokens. Arbitrary paints belong on slotted paths or CSS. */
export type IconPaint = "currentColor" | "none";

/** Native SVG stroke-linecap values accepted by the root. */
export type IconStrokeLinecap = "butt" | "round" | "square";

/** Native SVG stroke-linejoin values accepted by the root. */
export type IconStrokeLinejoin = "arcs" | "bevel" | "miter" | "miter-clip" | "round";

/** Numeric stroke width tokens accepted by the root SVG. */
export type IconStrokeWidth =
  | 0
  | 0.5
  | 1
  | 1.5
  | 2
  | 2.5
  | 3
  | 4
  | "0"
  | "0.5"
  | "1"
  | "1.5"
  | "2"
  | "2.5"
  | "3"
  | "4";

export type IconElement = PrimitiveElement;

/** Shared structural props for Icon. */
export interface IconBaseProps {
  /**
   * SVG element, custom element, or component to render as the root.
   *
   * @default "svg"
   */
  readonly as?: PrimitiveAs;

  /**
   * Consumer-owned root id. Missing ids are not generated for decorative icons.
   *
   * @default undefined
   */
  readonly id?: string;

  /**
   * SVG viewport.
   *
   * @default "0 0 24 24"
   */
  readonly viewBox?: string;

  /**
   * Native SVG width attribute.
   *
   * @default "1em"
   */
  readonly width?: string | number;

  /**
   * Native SVG height attribute.
   *
   * @default "1em"
   */
  readonly height?: string | number;

  /**
   * Consumer styling token mirrored to `data-size`.
   *
   * @default "md"
   */
  readonly size?: IconSize;

  /**
   * Native SVG focusability.
   *
   * @default false
   */
  readonly focusable?: boolean;

  /**
   * Root fill attribute.
   *
   * @default "none"
   */
  readonly fill?: IconPaint;

  /**
   * Root stroke attribute.
   *
   * @default "currentColor"
   */
  readonly stroke?: IconPaint;

  /**
   * Root stroke-width attribute.
   *
   * @default "2"
   */
  readonly strokeWidth?: IconStrokeWidth;

  /**
   * Root stroke-linecap attribute.
   *
   * @default "round"
   */
  readonly strokeLinecap?: IconStrokeLinecap;

  /**
   * Root stroke-linejoin attribute.
   *
   * @default "round"
   */
  readonly strokeLinejoin?: IconStrokeLinejoin;
}

/** Decorative icon props. Accessible labels are intentionally unavailable here. */
export interface IconDecorativeProps {
  /**
   * Treat the icon as decorative. Icons with no accessible name are decorative by default.
   *
   * @default true when no accessible name is provided
   */
  readonly decorative?: true;

  /**
   * Force the root out of the accessibility tree.
   *
   * @default true when no accessible name is provided
   */
  readonly ariaHidden?: true;

  readonly ariaLabel?: never;
  readonly ariaLabelledby?: never;
  readonly ariaDescribedby?: never;
  readonly title?: never;
  readonly description?: never;
  readonly titleId?: never;
  readonly descriptionId?: never;
}

/** Accessible name variants for a non-decorative icon. */
export type IconAccessibleNameProps =
  | {
      /** Direct accessible name. */
      readonly ariaLabel: string;
      readonly ariaLabelledby?: string;
      readonly title?: string;
    }
  | {
      /** Element ids that provide the accessible name. */
      readonly ariaLabel?: string;
      readonly ariaLabelledby: string;
      readonly title?: string;
    }
  | {
      /** Inline SVG title used as the accessible name when no ARIA name is provided. */
      readonly ariaLabel?: string;
      readonly ariaLabelledby?: string;
      readonly title: string;
    };

/** Non-decorative icon props. */
export type IconImageProps = IconAccessibleNameProps & {
  /**
   * Keep the icon in the accessibility tree.
   *
   * @default false when an accessible name is provided
   */
  readonly decorative?: false;

  /**
   * Keep the icon in the accessibility tree.
   *
   * @default false when an accessible name is provided
   */
  readonly ariaHidden?: false;

  /**
   * Element ids that describe the icon.
   *
   * @default generated from `description` when present
   */
  readonly ariaDescribedby?: string;

  /**
   * Inline SVG description.
   *
   * @default undefined
   */
  readonly description?: string;

  /**
   * Consumer-owned id for the generated title element.
   *
   * @default generated when `title` is present
   */
  readonly titleId?: string;

  /**
   * Consumer-owned id for the generated desc element.
   *
   * @default generated when `description` is present
   */
  readonly descriptionId?: string;
};

/** Public Icon props. Empty props intentionally produce a decorative icon. */
export type IconProps = IconBaseProps & (IconDecorativeProps | IconImageProps);

/** State passed to Icon slots and exposed refs. */
export interface IconSlotState {
  readonly ariaState: IconAriaState;
  readonly decorative: boolean;
  readonly descriptionId: string | undefined;
  readonly size: IconSize;
  readonly titleId: string | undefined;
  readonly viewBox: string;
}

/** Public Icon expose contract. */
export interface IconExpose extends IconSlotState {
  readonly element: IconElement | null;
}

/** Native submission behavior for IconButton when rendered as a button. */
export type IconButtonType = "button" | "reset" | "submit";

/** Consumer styling size mirrored to `data-size`; no CSS is emitted. */
export type IconButtonSize = "sm" | "md" | "lg";

/** Consumer styling tone mirrored to `data-tone`; no CSS is emitted. */
export type IconButtonTone = "accent" | "danger" | "neutral";

/** Consumer styling variant mirrored to `data-variant`; no CSS is emitted. */
export type IconButtonVariant = "outline" | "plain" | "soft" | "solid";

/** Availability state mirrored to `data-state`. */
export type IconButtonState = "disabled" | "idle" | "loading";

export type IconButtonElement = PrimitiveElement;

/** Shared structural props for IconButton. */
export interface IconButtonBaseProps {
  /**
   * Native element, custom element, or component to render as the root.
   *
   * @default "button"
   */
  readonly as?: PrimitiveAs;

  /**
   * Whether the rendered target already implements native button semantics.
   *
   * @default true when `as` is "button"; otherwise false
   */
  readonly native?: boolean;

  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: IconButtonType;

  /**
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Announce in-progress work and prevent repeated activation while preserving focus.
   *
   * @default false
   */
  readonly loading?: boolean;

  /**
   * Consumer styling size mirrored to `data-size`.
   *
   * @default "md"
   */
  readonly size?: IconButtonSize;

  /**
   * Consumer styling tone mirrored to `data-tone`.
   *
   * @default "neutral"
   */
  readonly tone?: IconButtonTone;

  /**
   * Consumer styling variant mirrored to `data-variant`.
   *
   * @default "plain"
   */
  readonly variant?: IconButtonVariant;

  /**
   * Element ids that describe the icon button.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;
}

/** Accessible name variants required for icon-only buttons. */
export type IconButtonNameProps =
  | {
      /** Direct accessible name. */
      readonly ariaLabel: string;
      readonly ariaLabelledby?: string;
    }
  | {
      /** Element ids that provide the accessible name. */
      readonly ariaLabel?: string;
      readonly ariaLabelledby: string;
    };

/** Public IconButton props. A name is required by type. */
export type IconButtonProps = IconButtonBaseProps & IconButtonNameProps;

/** State passed to IconButton slots and exposed refs. */
export interface IconButtonSlotState {
  readonly disabled: boolean;
  readonly loading: boolean;
  readonly size: IconButtonSize;
  readonly state: IconButtonState;
  readonly tone: IconButtonTone;
  readonly unavailable: boolean;
  readonly variant: IconButtonVariant;
}

/** Public IconButton expose contract. */
export interface IconButtonExpose extends IconButtonSlotState {
  readonly element: IconButtonElement | null;
  readonly focus: (options?: FocusOptions) => void;
}

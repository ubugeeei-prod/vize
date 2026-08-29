import type { ComponentPublicInstance } from "vue";

/** Logical axis used by {@link Stack}. */
export type StackAxis = "block" | "inline";

/** Native CSS `align-items` values supported by {@link Stack}. */
export type StackAlign = "stretch" | "start" | "center" | "end" | "baseline";

/** Native CSS `justify-content` values supported by {@link Stack}. */
export type StackJustify =
  | "start"
  | "center"
  | "end"
  | "space-between"
  | "space-around"
  | "space-evenly";

/** Native CSS length, percentage, keyword, or custom property used by {@link Stack}. */
export type StackGap = string;

/** CSS `flex-direction` produced by the logical stack axis. */
export type StackFlexDirection = "column" | "column-reverse" | "row" | "row-reverse";

/** Rendered data state for {@link Stack}. */
export type StackState = "stacked";

/** Rendered value exposed by {@link Stack}. */
export type StackElement = Element | ComponentPublicInstance;

/** State exposed to the default Stack slot. */
export interface StackSlotState {
  /** Logical axis used for the flex main axis. */
  readonly axis: StackAxis;

  /** Whether the selected logical axis is reversed. */
  readonly reversed: boolean;

  /** CSS `flex-direction` applied to the rendered host. */
  readonly direction: StackFlexDirection;

  /** Native CSS `gap` value between direct children. */
  readonly gap: StackGap;

  /** Native CSS `align-items` value for the cross axis. */
  readonly align: StackAlign;

  /** Native CSS `justify-content` value for the main axis. */
  readonly justify: StackJustify;

  /** Stable state token for styling and tests. */
  readonly state: StackState;
}

/** Inline style hooks applied to the rendered Stack host. */
export interface StackStyle {
  /** Consumer-overridable gap hook read by the host `gap` declaration. */
  readonly "--vize-ui-stack-gap": StackGap;

  /** Consumer-overridable alignment hook read by the host `align-items` declaration. */
  readonly "--vize-ui-stack-align": StackAlign;

  /** Consumer-overridable justification hook read by the host `justify-content` declaration. */
  readonly "--vize-ui-stack-justify": StackJustify;

  /** Flex layout mode for a single non-wrapping axis. */
  readonly display: "flex";

  /** CSS `flex-direction` resolved from `axis` and `reversed`. */
  readonly flexDirection: StackFlexDirection;

  /** Native child spacing declaration. */
  readonly gap: string;

  /** Native cross-axis alignment declaration. */
  readonly alignItems: string;

  /** Native main-axis distribution declaration. */
  readonly justifyContent: string;
}

/** Resolved layout state published by {@link Stack}. */
export interface StackResolvedLayout extends StackSlotState {
  /** Native flexbox style object applied to the host. */
  readonly style: StackStyle;
}

/** Public instance state exposed by the Stack primitive. */
export interface StackExpose extends StackSlotState {
  /** Rendered host element or component instance. */
  readonly element: StackElement | null;
}

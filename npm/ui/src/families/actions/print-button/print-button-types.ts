import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Native submission behavior for PrintButton when rendered as a button. */
export type PrintButtonType = "button" | "reset" | "submit";

/** Stable print lifecycle state exposed for styling, slots, emits, and tests. */
export type PrintButtonState = "idle" | "printing" | "printed" | "error";

/** Async print action used by PrintButton; defaults to the platform print function. */
export type PrintButtonAction = (nativeEvent: MouseEvent) => void | Promise<void>;

/** Rendered value exposed by PrintButton. */
export type PrintButtonElement = PrimitiveElement;

/** Stable part names emitted by PrintButton. */
export type PrintButtonPart = "label" | "root";

/** Stable `data-vize-ui` values emitted by PrintButton. */
export type PrintButtonDataName = "print-button" | "print-button-label";

/** Stable data attributes emitted by PrintButton. */
export type PrintButtonDataAttribute =
  | "data-disabled"
  | "data-printing"
  | "data-state"
  | "data-vize-ui";

/** PrintButton emits no CSS custom properties; styling remains consumer-owned. */
export type PrintButtonCssCustomProperty = never;

/** Public props accepted by PrintButton. */
export interface PrintButtonProps {
  /**
   * Native element, custom element, or component to render.
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
  readonly type?: PrintButtonType;

  /**
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Idle fallback text rendered when no default slot is supplied.
   *
   * @default "Print"
   */
  readonly idleLabel?: string;

  /**
   * Busy fallback text rendered while the configured action is pending.
   *
   * @default "Printing"
   */
  readonly printingLabel?: string;

  /**
   * Success fallback text rendered when no default slot is supplied.
   *
   * @default "Printed"
   */
  readonly printedLabel?: string;

  /**
   * Failure fallback text rendered when no default slot is supplied.
   *
   * @default "Print failed"
   */
  readonly errorLabel?: string;

  /**
   * Accessible name when no visible label or `aria-labelledby` supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Space-separated ids that label the button.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Space-separated ids that describe the button.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Test, platform, or product hook for triggering print.
   *
   * @default globalThis.print
   */
  readonly action?: PrintButtonAction;
}

/** Events emitted by PrintButton. */
export interface PrintButtonEmits {
  /** Fired after the configured action completes. */
  print: [nativeEvent: MouseEvent];

  /** Fired after the configured action throws or rejects. */
  error: [error: unknown, nativeEvent: MouseEvent];
}

/** State exposed to the PrintButton default slot. */
export interface PrintButtonSlotState {
  /** Whether the control suppresses user activation. */
  readonly disabled: boolean;

  /** Whether an async print action is already in flight. */
  readonly printing: boolean;

  /** Whether activation is currently unavailable. */
  readonly unavailable: boolean;

  /** Stable print lifecycle token for styling and tests. */
  readonly state: PrintButtonState;

  /** Resolved fallback text for the current state. */
  readonly label: string;
}

/** Slots exposed by PrintButton. */
export interface PrintButtonSlots {
  /** Renders button contents with the current print state. */
  default(props: PrintButtonSlotState): unknown;
}

/** Public component instance state exposed by PrintButton. */
export interface PrintButtonExpose extends PrintButtonSlotState {
  /** Rendered root element or component instance. */
  readonly element: PrintButtonElement | null;

  /** Move focus to the rendered control. */
  readonly focus: (options?: FocusOptions) => void;
}

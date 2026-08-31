import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Native submission behavior for CopyButton when rendered as a button. */
export type CopyButtonType = "button" | "reset" | "submit";

/** Stable copy result state exposed for styling, slots, emits, and tests. */
export type CopyButtonState = "idle" | "copied" | "error";

/** Async text writer used by CopyButton; defaults to `navigator.clipboard.writeText`. */
export type CopyButtonWriter = (value: string) => void | Promise<void>;

/** Rendered value exposed by CopyButton. */
export type CopyButtonElement = PrimitiveElement;

/** Stable part names emitted by CopyButton. */
export type CopyButtonPart = "label" | "root";

/** Stable `data-vize-ui` values emitted by CopyButton. */
export type CopyButtonDataName = "copy-button" | "copy-button-label";

/** Stable data attributes emitted by CopyButton. */
export type CopyButtonDataAttribute =
  | "data-disabled"
  | "data-state"
  | "data-vize-ui"
  | "data-writing";

/** CopyButton emits no CSS custom properties; styling remains consumer-owned. */
export type CopyButtonCssCustomProperty = never;

/** Public props accepted by CopyButton. */
export interface CopyButtonProps {
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
  readonly type?: CopyButtonType;

  /**
   * Plain string copied to the system clipboard on activation.
   *
   * @default required
   */
  readonly value: string;

  /**
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Idle fallback text rendered when no default slot is supplied.
   *
   * @default "Copy"
   */
  readonly idleLabel?: string;

  /**
   * Success fallback text rendered when no default slot is supplied.
   *
   * @default "Copied"
   */
  readonly copiedLabel?: string;

  /**
   * Failure fallback text rendered when no default slot is supplied.
   *
   * @default "Copy failed"
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
   * Test or platform hook for writing clipboard text.
   *
   * @default navigator.clipboard.writeText
   */
  readonly writer?: CopyButtonWriter;
}

/** Events emitted by CopyButton. */
export interface CopyButtonEmits {
  /** Fired after the configured writer accepts the value. */
  copy: [value: string, nativeEvent: MouseEvent];

  /** Fired after the configured writer rejects the value. */
  error: [error: unknown, value: string, nativeEvent: MouseEvent];
}

/** State exposed to the CopyButton default slot. */
export interface CopyButtonSlotState {
  /** Plain string that will be passed to the writer on activation. */
  readonly value: string;

  /** Whether the control suppresses user activation. */
  readonly disabled: boolean;

  /** Whether an async write is already in flight. */
  readonly writing: boolean;

  /** Whether activation is currently unavailable. */
  readonly unavailable: boolean;

  /** Stable copy result token for styling and tests. */
  readonly state: CopyButtonState;

  /** Resolved fallback text for the current state. */
  readonly label: string;
}

/** Slots exposed by CopyButton. */
export interface CopyButtonSlots {
  /** Renders button contents with the current copy state. */
  default(props: CopyButtonSlotState): unknown;
}

/** Public component instance state exposed by CopyButton. */
export interface CopyButtonExpose extends CopyButtonSlotState {
  /** Rendered root element or component instance. */
  readonly element: CopyButtonElement | null;

  /** Move focus to the rendered control. */
  readonly focus: (options?: FocusOptions) => void;
}

import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

/** Native submission behavior for FullscreenButton when rendered as a button. */
export type FullscreenButtonType = "button" | "reset" | "submit";

/** Stable fullscreen lifecycle state exposed for styling, slots, emits, and tests. */
export type FullscreenButtonState = "idle" | "entering" | "active" | "exiting" | "error";

/** Fullscreen operation requested by one user activation. */
export type FullscreenButtonOperationType = "enter" | "exit";

/** Target element, or lazy target resolver, used when entering fullscreen. */
export type FullscreenButtonTarget =
  | Element
  | null
  | undefined
  | ((nativeEvent: MouseEvent) => Element | null | undefined);

/** Injected fullscreen adapter used by tests, SSR, and product integrations. */
export interface FullscreenButtonController {
  /**
   * Current fullscreen element when known.
   *
   * @default document.fullscreenElement
   */
  readonly getFullscreenElement?: () => Element | null | undefined;

  /** Request fullscreen for the submitted target. */
  readonly requestFullscreen: (target: Element, nativeEvent: MouseEvent) => void | Promise<void>;

  /** Exit the current fullscreen session. */
  readonly exitFullscreen: (nativeEvent: MouseEvent) => void | Promise<void>;
}

/** Submitted fullscreen operation preserved across async races. */
export interface FullscreenButtonOperation {
  /** Whether the activation requested entry or exit. */
  readonly type: FullscreenButtonOperationType;

  /** Target captured for entry, or the known fullscreen element for exit. */
  readonly target: Element | null;

  /** Controller captured at activation time. */
  readonly controller: FullscreenButtonController;
}

/** Rendered value exposed by FullscreenButton. */
export type FullscreenButtonElement = PrimitiveElement;

/** Stable part names emitted by FullscreenButton. */
export type FullscreenButtonPart = "label" | "root";

/** Stable `data-vize-ui` values emitted by FullscreenButton. */
export type FullscreenButtonDataName = "fullscreen-button" | "fullscreen-button-label";

/** Stable data attributes emitted by FullscreenButton. */
export type FullscreenButtonDataAttribute =
  | "data-active"
  | "data-disabled"
  | "data-pending"
  | "data-state"
  | "data-vize-ui";

/** FullscreenButton emits no CSS custom properties; styling remains consumer-owned. */
export type FullscreenButtonCssCustomProperty = never;

/** Public props accepted by FullscreenButton. */
export interface FullscreenButtonProps {
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
  readonly type?: FullscreenButtonType;

  /**
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Fallback text rendered when fullscreen is inactive and no default slot is supplied.
   *
   * @default "Enter fullscreen"
   */
  readonly enterLabel?: string;

  /**
   * Fallback text rendered when fullscreen is active and no default slot is supplied.
   *
   * @default "Exit fullscreen"
   */
  readonly exitLabel?: string;

  /**
   * Fallback text rendered while a fullscreen operation is pending.
   *
   * @default "Changing fullscreen"
   */
  readonly busyLabel?: string;

  /**
   * Failure fallback text rendered when no default slot is supplied.
   *
   * @default "Fullscreen failed"
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
   * Target used when entering fullscreen.
   *
   * @default document.documentElement at activation time
   */
  readonly target?: FullscreenButtonTarget;

  /**
   * Test, platform, or product hook for fullscreen requests.
   *
   * @default platform fullscreen controller
   */
  readonly controller?: FullscreenButtonController;
}

/** Events emitted by FullscreenButton. */
export interface FullscreenButtonEmits {
  /** Fired after the submitted fullscreen operation completes. */
  fullscreen: [operation: FullscreenButtonOperation, nativeEvent: MouseEvent];

  /** Fired after the submitted fullscreen operation throws or rejects. */
  error: [error: unknown, operation: FullscreenButtonOperation, nativeEvent: MouseEvent];
}

/** State exposed to the FullscreenButton default slot. */
export interface FullscreenButtonSlotState {
  /** Whether the control suppresses user activation. */
  readonly disabled: boolean;

  /** Whether fullscreen is currently considered active. */
  readonly active: boolean;

  /** Whether an async fullscreen operation is already in flight. */
  readonly pending: boolean;

  /** Pending operation type, or `null` when idle. */
  readonly operation: FullscreenButtonOperationType | null;

  /** Whether activation is currently unavailable. */
  readonly unavailable: boolean;

  /** Stable fullscreen lifecycle token for styling and tests. */
  readonly state: FullscreenButtonState;

  /** Resolved fallback text for the current state. */
  readonly label: string;
}

/** Slots exposed by FullscreenButton. */
export interface FullscreenButtonSlots {
  /** Renders button contents with the current fullscreen state. */
  default(props: FullscreenButtonSlotState): unknown;
}

/** Public component instance state exposed by FullscreenButton. */
export interface FullscreenButtonExpose extends FullscreenButtonSlotState {
  /** Rendered root element or component instance. */
  readonly element: FullscreenButtonElement | null;

  /** Move focus to the rendered control. */
  readonly focus: (options?: FocusOptions) => void;
}

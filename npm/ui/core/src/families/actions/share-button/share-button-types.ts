import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";

/** Native submission behavior for ShareButton when rendered as a button. */
export type ShareButtonType = "button" | "reset" | "submit";

/** Stable share lifecycle state exposed for styling, slots, emits, and tests. */
export type ShareButtonState = "idle" | "sharing" | "shared" | "error";

/** Closed payload accepted by the Web Share API and ShareButton actions. */
export interface ShareButtonPayload {
  /** Optional share title. */
  readonly title?: string;

  /** Optional share body text. */
  readonly text?: string;

  /** Optional URL to share. */
  readonly url?: string;

  /** Optional files to share when the platform supports file sharing. */
  readonly files?: File[];
}

/** Async share action used by ShareButton; defaults to `navigator.share`. */
export type ShareButtonAction = (
  payload: ShareButtonPayload,
  nativeEvent: MouseEvent,
) => void | Promise<void>;

/** Rendered value exposed by ShareButton. */
export type ShareButtonElement = PrimitiveElement;

/** Stable part names emitted by ShareButton. */
export type ShareButtonPart = "label" | "root";

/** Stable `data-vize-ui` values emitted by ShareButton. */
export type ShareButtonDataName = "share-button" | "share-button-label";

/** Stable data attributes emitted by ShareButton. */
export type ShareButtonDataAttribute =
  | "data-disabled"
  | "data-sharing"
  | "data-state"
  | "data-vize-ui";

/** ShareButton emits no CSS custom properties; styling remains consumer-owned. */
export type ShareButtonCssCustomProperty = never;

/** Public props accepted by ShareButton. */
export interface ShareButtonProps {
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
  readonly type?: ShareButtonType;

  /**
   * Remove the control from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Optional share title.
   *
   * @default undefined
   */
  readonly title?: string;

  /**
   * Optional share body text.
   *
   * @default undefined
   */
  readonly text?: string;

  /**
   * Optional URL to share.
   *
   * @default undefined
   */
  readonly url?: string;

  /**
   * Optional files to share when the platform supports file sharing.
   *
   * @default undefined
   */
  readonly files?: File[];

  /**
   * Idle fallback text rendered when no default slot is supplied.
   *
   * @default "Share"
   */
  readonly idleLabel?: string;

  /**
   * Busy fallback text rendered while the configured action is pending.
   *
   * @default "Sharing"
   */
  readonly sharingLabel?: string;

  /**
   * Success fallback text rendered when no default slot is supplied.
   *
   * @default "Shared"
   */
  readonly sharedLabel?: string;

  /**
   * Failure fallback text rendered when no default slot is supplied.
   *
   * @default "Share failed"
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
   * Test, platform, or product hook for sharing.
   *
   * @default navigator.share
   */
  readonly action?: ShareButtonAction;
}

/** Events emitted by ShareButton. */
export interface ShareButtonEmits {
  /** Fired after the submitted share action completes. */
  share: [payload: ShareButtonPayload, nativeEvent: MouseEvent];

  /** Fired after the submitted share action throws or rejects. */
  error: [error: unknown, payload: ShareButtonPayload, nativeEvent: MouseEvent];
}

/** State exposed to the ShareButton default slot. */
export interface ShareButtonSlotState {
  /** Payload captured for the next activation. */
  readonly payload: ShareButtonPayload;

  /** Whether the control suppresses user activation. */
  readonly disabled: boolean;

  /** Whether an async share action is already in flight. */
  readonly sharing: boolean;

  /** Whether activation is currently unavailable. */
  readonly unavailable: boolean;

  /** Stable share lifecycle token for styling and tests. */
  readonly state: ShareButtonState;

  /** Resolved fallback text for the current state. */
  readonly label: string;
}

/** Slots exposed by ShareButton. */
export interface ShareButtonSlots {
  /** Renders button contents with the current share state. */
  default(props: ShareButtonSlotState): unknown;
}

/** Public component instance state exposed by ShareButton. */
export interface ShareButtonExpose extends ShareButtonSlotState {
  /** Rendered root element or component instance. */
  readonly element: ShareButtonElement | null;

  /** Move focus to the rendered control. */
  readonly focus: (options?: FocusOptions) => void;
}

import type { MaybeRefOrGetter, ShallowRef } from "vue";

import type { PressEvent, PressPointerType, PressProps } from "../press/press-types.ts";

/** Pointing-device families eligible to start a long press. */
export type LongPressPointerType = Extract<PressPointerType, "mouse" | "pen" | "pointer" | "touch">;

/** Lifecycle notification emitted by a long-press controller. */
export type LongPressEventType = "longpress" | "longpressend" | "longpressstart";

/** Immutable snapshot of one long-press lifecycle event. */
export interface LongPressEvent extends Omit<PressEvent, "type" | "pointerType"> {
  /** Long-press lifecycle phase represented by this snapshot. */
  readonly type: LongPressEventType;

  /** Pointing-device family that initiated the interaction. */
  readonly pointerType: LongPressPointerType;
}

/** Options shared by {@link createLongPress} and {@link useLongPress}. */
export interface LongPressOptions {
  /**
   * Suppress long and short activation while retaining the bound props.
   * Reactive values are checked at start and again when the threshold elapses.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Restrict long-press recognition to one pointing-device family.
   * Keyboard and virtual activation remain available through `onPress`.
   *
   * @default undefined
   */
  readonly pointerType?: MaybeRefOrGetter<LongPressPointerType | undefined>;

  /**
   * Time in milliseconds that the primary pointer must remain down.
   * The value is resolved once per attempt and must be finite and non-negative.
   *
   * @default 500
   */
  readonly threshold?: MaybeRefOrGetter<number | undefined>;

  /**
   * Accessible explanation of the long action, for example
   * "Long press to open actions". Used as `aria-description` unless
   * `accessibilityDescriptionId` is supplied.
   */
  readonly accessibilityDescription?: MaybeRefOrGetter<string | undefined>;

  /**
   * ID of consumer-rendered descriptive content. When supplied, the host uses
   * `aria-describedby` and the inline accessibility description is omitted.
   */
  readonly accessibilityDescriptionId?: MaybeRefOrGetter<string | undefined>;

  /**
   * Preserve selectable text during the pointer attempt.
   *
   * @default false
   */
  readonly allowTextSelectionOnPress?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Prevent the compatibility mousedown default that normally moves focus.
   *
   * @default false
   */
  readonly preventFocusOnPress?: MaybeRefOrGetter<boolean | undefined>;

  /** Called after a qualifying primary pointer starts. */
  readonly onLongPressStart?: (event: LongPressEvent) => void;

  /** Called when an attempt ends, whether before or after the threshold. */
  readonly onLongPressEnd?: (event: LongPressEvent) => void;

  /** Called exactly once when the configured threshold is reached. */
  readonly onLongPress?: (event: LongPressEvent) => void;

  /**
   * Called for an ordinary short, keyboard, or virtual activation.
   * Use this as the keyboard-accessible alternative to a long-only action.
   */
  readonly onPress?: (event: PressEvent) => void;
}

/** Attributes and native handlers to spread onto one long-press host. */
export interface LongPressProps extends PressProps {
  readonly "aria-describedby": string | undefined;
  readonly "aria-description": string | undefined;
  readonly onContextmenu: (event: MouseEvent) => void;
}

/** Stateful long-press recognizer with explicit lifecycle ownership. */
export interface LongPressController {
  /** Whether an eligible pointer attempt is currently active. */
  readonly isPressed: Readonly<ShallowRef<boolean>>;

  /** Whether the active attempt has crossed the configured threshold. */
  readonly isLongPressed: Readonly<ShallowRef<boolean>>;

  /** Stable handlers and accessibility attributes for exactly one host. */
  readonly longPressProps: Readonly<LongPressProps>;

  /**
   * Cancel the current pending or triggered interaction.
   * Returns `true` when an interaction was canceled.
   *
   * @throws Error with `VIZE_UI_LONG_PRESS_DISPOSED` after disposal.
   */
  readonly cancel: () => boolean;

  /** Release timers, listeners, selection guards, and reactive state. */
  readonly dispose: () => void;
}

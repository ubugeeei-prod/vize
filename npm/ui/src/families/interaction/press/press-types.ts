import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Stable input family reported by a press interaction. */
export type PressPointerType = "keyboard" | "mouse" | "pen" | "pointer" | "touch" | "virtual";

/** Lifecycle notification emitted by a press controller. */
export type PressEventType = "press" | "pressend" | "pressstart" | "pressup";

/** Keyboard semantics applied when the host is not natively activatable. */
export type PressKeyboardBehavior = "button" | "link" | "none";

/** Immutable, renderer-independent snapshot of one press lifecycle event. */
export interface PressEvent {
  /** Press lifecycle phase represented by this snapshot. */
  readonly type: PressEventType;

  /** Input family that initiated the interaction. */
  readonly pointerType: PressPointerType;

  /** Element whose bound press props own the interaction. */
  readonly target: Element;

  /** Native event responsible for this phase, or `null` for manual cancellation. */
  readonly originalEvent: Event | null;

  /** Viewport coordinate when supplied by pointing hardware. */
  readonly x: number | null;

  /** Viewport coordinate when supplied by pointing hardware. */
  readonly y: number | null;

  /** Modifier-key snapshots captured from the native event. */
  readonly altKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;

  /** Whether the lifecycle ended without an activation. */
  readonly isCanceled: boolean;
}

/** Options shared by {@link createPress} and {@link usePress}. */
export interface PressOptions {
  /**
   * Suppress every activation while retaining the bound props.
   *
   * Reactive values are read for every event, so disabling an interaction
   * between pointer-down and pointer-up cancels it deterministically.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Keyboard contract for hosts without native activation behavior.
   * Native buttons, links, inputs, and summaries always keep browser timing.
   *
   * @default "button"
   */
  readonly keyboardBehavior?: MaybeRefOrGetter<PressKeyboardBehavior | undefined>;

  /**
   * End the interaction permanently on its first pointer exit.
   * When false, leaving pauses pressed state and re-entering resumes it.
   *
   * @default false
   */
  readonly shouldCancelOnPointerExit?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Prevent the compatibility mousedown default that normally moves focus.
   * Keyboard and virtual activation never lose focus.
   *
   * @default false
   */
  readonly preventFocusOnPress?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Preserve selectable text during an active pointer press. When false, the
   * host's exact inline user-select declarations are restored at press end.
   *
   * @default false
   */
  readonly allowTextSelectionOnPress?: MaybeRefOrGetter<boolean | undefined>;

  /** Called after pressed state becomes active. */
  readonly onPressStart?: (event: PressEvent) => void;

  /** Called after pressed state ends or pauses at the pointer boundary. */
  readonly onPressEnd?: (event: PressEvent) => void;

  /** Called for every distinct pressed-state transition. */
  readonly onPressChange?: (isPressed: boolean) => void;

  /** Called when keyboard or pointing hardware is released over the host. */
  readonly onPressUp?: (event: PressEvent) => void;

  /** Called exactly once for a completed activation. */
  readonly onPress?: (event: PressEvent) => void;
}

/** Framework props to spread onto the element that owns a press interaction. */
export interface PressProps {
  readonly onClick: (event: MouseEvent) => void;
  readonly onDragstart: (event: DragEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onKeyup: (event: KeyboardEvent) => void;
  readonly onMousedown: (event: MouseEvent) => void;
  readonly onMousemove: (event: MouseEvent) => void;
  readonly onMouseup: (event: MouseEvent) => void;
  readonly onPointercancel: (event: PointerEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onPointermove: (event: PointerEvent) => void;
  readonly onPointerup: (event: PointerEvent) => void;
  readonly onTouchcancel: (event: TouchEvent) => void;
  readonly onTouchend: (event: TouchEvent) => void;
  readonly onTouchmove: (event: TouchEvent) => void;
  readonly onTouchstart: (event: TouchEvent) => void;
}

/** Stateful press normalizer with explicit lifecycle ownership. */
export interface PressController {
  /** Whether the active input is currently within the pressed region. */
  readonly isPressed: Readonly<ShallowRef<boolean>>;

  /** Stable handlers to merge or spread onto exactly one host element. */
  readonly pressProps: Readonly<PressProps>;

  /** Cancel the current interaction without producing an activation. */
  readonly cancel: () => boolean;

  /** Release document listeners, timers, and transient selection changes. */
  readonly dispose: () => void;
}

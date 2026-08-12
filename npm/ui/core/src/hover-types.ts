import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Pointing-device families that have a meaningful hover state. */
export type HoverPointerType = "mouse" | "pen";

/** Lifecycle notification emitted by a hover controller. */
export type HoverEventType = "hoverend" | "hoverstart";

/** Immutable snapshot of one hover lifecycle transition. */
export interface HoverEvent {
  /** Hover lifecycle phase represented by this snapshot. */
  readonly type: HoverEventType;

  /** Hover-capable pointing-device family. */
  readonly pointerType: HoverPointerType;

  /** Element whose bound hover props own the interaction. */
  readonly target: Element;

  /** Native event responsible for this transition, or `null` for manual cancellation. */
  readonly originalEvent: Event | null;

  /** Viewport coordinates when present on the native event. */
  readonly x: number | null;
  readonly y: number | null;

  /** Modifier-key snapshots captured from the native event. */
  readonly altKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;

  /** Whether hover ended due to cancellation rather than a normal boundary exit. */
  readonly isCanceled: boolean;
}

/** Options shared by {@link createHover} and {@link useHover}. */
export interface HoverOptions {
  /**
   * Suppress hover while retaining stable bound props.
   * Reactive values are resolved for every relevant native event.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Restrict recognition to one hover-capable device family.
   *
   * @default undefined
   */
  readonly pointerType?: MaybeRefOrGetter<HoverPointerType | undefined>;

  /** Called after hover state becomes active. */
  readonly onHoverStart?: (event: HoverEvent) => void;

  /** Called after hover ends or is canceled. */
  readonly onHoverEnd?: (event: HoverEvent) => void;

  /** Called for every distinct hover-state transition. */
  readonly onHoverChange?: (isHovered: boolean) => void;
}

/** Native handlers to spread onto exactly one hover host. */
export interface HoverProps {
  readonly onMouseenter: (event: MouseEvent) => void;
  readonly onMouseleave: (event: MouseEvent) => void;
  readonly onMousemove: (event: MouseEvent) => void;
  readonly onPointercancel: (event: PointerEvent) => void;
  readonly onPointerenter: (event: PointerEvent) => void;
  readonly onPointerleave: (event: PointerEvent) => void;
  readonly onPointermove: (event: PointerEvent) => void;
  readonly onTouchstart: (event: TouchEvent) => void;
}

/** Stateful hover normalizer with explicit listener ownership. */
export interface HoverController {
  /** Whether a qualifying mouse or pen currently hovers the host. */
  readonly isHovered: Readonly<ShallowRef<boolean>>;

  /** Stable native handlers to merge or spread onto one host. */
  readonly hoverProps: Readonly<HoverProps>;

  /** Cancel the current hover interaction. */
  readonly cancel: () => boolean;

  /** Release document listeners and reactive state. */
  readonly dispose: () => void;
}

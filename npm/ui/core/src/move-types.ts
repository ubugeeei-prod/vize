import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Input families normalized by the move interaction. */
export type MovePointerType = "keyboard" | "mouse" | "pen" | "pointer" | "touch";

/** Lifecycle phases emitted by a move controller. */
export type MoveEventType = "move" | "moveend" | "movestart";

interface MoveEventBase<Type extends MoveEventType> {
  /** Move lifecycle phase represented by this immutable snapshot. */
  readonly type: Type;

  /** Input family that owns this movement. */
  readonly pointerType: MovePointerType;

  /** Element whose bound props own the interaction. */
  readonly target: Element;

  /** Native event responsible for this phase, or `null` for manual cancellation. */
  readonly originalEvent: Event | null;

  /** Page coordinates when supplied by pointing hardware. */
  readonly x: number | null;
  readonly y: number | null;

  /** Modifier-key snapshots captured from the native event. */
  readonly altKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;
}

/** Snapshot emitted immediately before the first non-zero movement. */
export interface MoveStartEvent extends MoveEventBase<"movestart"> {
  readonly deltaX: 0;
  readonly deltaY: 0;
  readonly isCanceled: false;
}

/** Snapshot of one non-zero movement since the preceding native event. */
export interface MoveUpdateEvent extends MoveEventBase<"move"> {
  readonly deltaX: number;
  readonly deltaY: number;
  readonly isCanceled: false;
}

/** Snapshot emitted after a movement completes or is canceled. */
export interface MoveEndEvent extends MoveEventBase<"moveend"> {
  readonly deltaX: 0;
  readonly deltaY: 0;
  readonly isCanceled: boolean;
}

/** Discriminated lifecycle event union for exhaustive consumer handling. */
export type MoveEvent = MoveEndEvent | MoveStartEvent | MoveUpdateEvent;

/** Options shared by {@link createMove} and {@link useMove}. */
export interface MoveOptions {
  /**
   * Suppress new movement and cancel an active pointer movement on its next event.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Distance contributed by each physical arrow-key press, including repeat events.
   * The resolved value must be finite and greater than zero.
   *
   * @default 1
   */
  readonly keyboardStep?: MaybeRefOrGetter<number | undefined>;

  /** Called exactly once before the first non-zero delta in an interaction. */
  readonly onMoveStart?: (event: MoveStartEvent) => void;

  /** Called for every non-zero pointer or keyboard delta. */
  readonly onMove?: (event: MoveUpdateEvent) => void;

  /** Called after an interaction that emitted at least one move finishes. */
  readonly onMoveEnd?: (event: MoveEndEvent) => void;
}

/** Native handlers to spread onto exactly one move host. */
export interface MoveProps {
  readonly onDragstart: (event: DragEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onMousedown: (event: MouseEvent) => void;
  readonly onPointercancel: (event: PointerEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onTouchcancel: (event: TouchEvent) => void;
  readonly onTouchend: (event: TouchEvent) => void;
  readonly onTouchmove: (event: TouchEvent) => void;
  readonly onTouchstart: (event: TouchEvent) => void;
}

/** Stateful move normalizer with explicit listener and selection ownership. */
export interface MoveController {
  /** Whether a pointer or ephemeral keyboard movement is currently being emitted. */
  readonly isMoving: Readonly<ShallowRef<boolean>>;

  /** Stable native handlers to spread or merge onto one host. */
  readonly moveProps: Readonly<MoveProps>;

  /**
   * Cancel the current pointer attempt, including one that has not moved yet.
   *
   * @returns `true` when an owned pointer or keyboard transaction was settled.
   * @throws An error carrying `VIZE_UI_MOVE_DISPOSED` after disposal.
   */
  readonly cancel: () => boolean;

  /** Release listeners, selection guards, and reactive state without callbacks. */
  readonly dispose: () => void;
}

import type { MaybeRefOrGetter, ShallowRef } from "vue";

import type {
  DragAnnouncements,
  DragEndEvent,
  DragMoveEvent,
  DragPayload,
  DragStartEvent,
  DropEdge,
  DropIndicatorState,
  DropTargetEvent,
  DropTargetRect,
} from "./drag-and-drop-types.ts";

/** Edge-proximity auto-scroll configuration for pointer sessions. */
export interface DragAutoScrollOptions {
  /** Scrollable element driven while a pointer drag approaches its edges. */
  readonly container: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Viewport rectangle of the container used for edge proximity.
   *
   * @default the container's `getBoundingClientRect()`
   */
  readonly getRect?: () => DropTargetRect | DOMRectReadOnly | null | undefined;

  /**
   * Distance in CSS pixels from a container edge that engages scrolling.
   * The resolved value must be finite and greater than zero.
   *
   * @default 48
   */
  readonly threshold?: MaybeRefOrGetter<number | undefined>;

  /**
   * Maximum scroll distance in CSS pixels applied per scroll step.
   * The resolved value must be finite and greater than zero.
   *
   * @default 16
   */
  readonly speed?: MaybeRefOrGetter<number | undefined>;
}

/** Options shared by `createDragAndDrop` and `useDragAndDrop`. */
export interface DragAndDropOptions<Data = unknown> {
  /**
   * Suppress new sessions and cancel the active session on its next event.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Chebyshev distance in CSS pixels a pointer must travel before a session
   * starts. The resolved value must be finite and greater than or equal to zero.
   *
   * @default 4
   */
  readonly startDistance?: MaybeRefOrGetter<number | undefined>;

  /**
   * Edge-proximity auto-scroll for pointer sessions.
   *
   * @default undefined
   */
  readonly autoScroll?: DragAutoScrollOptions;

  /**
   * Overrides merged over the built-in English announcement builders.
   *
   * @default built-in English announcements
   */
  readonly announcements?: DragAnnouncements<Data>;

  /** Called exactly once when a session starts. */
  readonly onDragStart?: (event: DragStartEvent<Data>) => void;

  /** Called for pointer movement and keyboard target changes inside a session. */
  readonly onDragMove?: (event: DragMoveEvent<Data>) => void;

  /** Called exactly once after a session drops or cancels. */
  readonly onDragEnd?: (event: DragEndEvent<Data>) => void;
}

/** Declaration of one draggable source owned by a controller. */
export interface DragSourceOptions<Data = unknown> {
  /** Stable key unique among this controller's sources. */
  readonly key: string;

  /** Dragged item element used for geometry; the handle receives `sourceProps`. */
  readonly element?: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Typed payload resolved when a session starts.
   *
   * @default undefined
   */
  readonly payload?: MaybeRefOrGetter<DragPayload<Data> | undefined>;

  /**
   * Human-readable name used by announcements.
   *
   * @default the source key
   */
  readonly label?: MaybeRefOrGetter<string | undefined>;

  /**
   * Suppress new sessions from this source.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Whether Enter or Space on the handle starts a keyboard session.
   *
   * @default true
   */
  readonly keyboard?: MaybeRefOrGetter<boolean | undefined>;
}

/** Native handlers to spread onto exactly one drag handle per source. */
export interface DragSourceProps {
  readonly onDragstart: (event: DragEvent) => void;
  readonly onFocusout: (event: FocusEvent) => void;
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onMousedown: (event: MouseEvent) => void;
  readonly onPointerdown: (event: PointerEvent) => void;
  readonly onTouchstart: (event: TouchEvent) => void;
}

/** Live registration owned by one drag source. */
export interface DragSourceRegistration {
  /** Key this registration was created with. */
  readonly key: string;

  /** Whether this source owns the active session. */
  readonly isDragging: Readonly<ShallowRef<boolean>>;

  /** Stable native handlers to spread or merge onto one drag handle. */
  readonly sourceProps: Readonly<DragSourceProps>;

  /** Remove the source; its active session is canceled first. */
  readonly dispose: () => void;
}

/** Declaration of one drop target owned by a controller. */
export interface DropTargetOptions<Data = unknown> {
  /** Stable key unique among this controller's targets. */
  readonly key: string;

  /** Element that owns this target's geometry and keyboard order. */
  readonly element: MaybeRefOrGetter<Element | null | undefined>;

  /**
   * Drop edges this target resolves, in preference order for keyboard sessions.
   *
   * @default ["inside"]
   */
  readonly edges?: MaybeRefOrGetter<readonly DropEdge[] | undefined>;

  /**
   * Viewport rectangle used for hit testing and indicator geometry.
   *
   * @default the element's `getBoundingClientRect()`
   */
  readonly getRect?: () => DropTargetRect | DOMRectReadOnly | null | undefined;

  /**
   * Whether this target accepts the session payload.
   *
   * @default every payload is accepted
   */
  readonly accepts?: (payload: DragPayload<Data> | null) => boolean;

  /**
   * Remove this target from hit testing and keyboard navigation.
   *
   * @default false
   */
  readonly isDisabled?: MaybeRefOrGetter<boolean | undefined>;

  /**
   * Human-readable name used by announcements.
   *
   * @default the target key
   */
  readonly label?: MaybeRefOrGetter<string | undefined>;

  /** Called when the drag enters this target. */
  readonly onEnter?: (event: DropTargetEvent<Data>) => void;

  /** Called when the drag moves inside this target or its edge changes. */
  readonly onMove?: (event: DropTargetEvent<Data>) => void;

  /** Called when the drag leaves this target without dropping. */
  readonly onLeave?: (event: DropTargetEvent<Data>) => void;

  /** Called when the session drops on this target. */
  readonly onDrop?: (event: DropTargetEvent<Data>) => void;
}

/** Live registration owned by one drop target. */
export interface DropTargetRegistration {
  /** Key this registration was created with. */
  readonly key: string;

  /** Whether the active session is currently over this target. */
  readonly isOver: Readonly<ShallowRef<boolean>>;

  /** Remove the target from hit testing and keyboard navigation. */
  readonly dispose: () => void;
}

/** Stateful drag-and-drop coordinator with explicit ownership and disposal. */
export interface DragAndDropController<Data = unknown> {
  /** Whether a pointer or keyboard session is active. */
  readonly isDragging: Readonly<ShallowRef<boolean>>;

  /** Key of the source that owns the active session, or `null`. */
  readonly sourceKey: Readonly<ShallowRef<string | null>>;

  /** Key of the target currently under the active session, or `null`. */
  readonly targetKey: Readonly<ShallowRef<string | null>>;

  /** Indicator state for the target currently under the active session. */
  readonly indicator: Readonly<ShallowRef<DropIndicatorState | null>>;

  /** Register one draggable source. Duplicate keys are rejected. */
  readonly registerSource: (options: DragSourceOptions<Data>) => DragSourceRegistration;

  /** Register one drop target. Duplicate keys are rejected. */
  readonly registerTarget: (options: DropTargetOptions<Data>) => DropTargetRegistration;

  /** Speak one message through the controller's assertive live region. */
  readonly announce: (message: string) => void;

  /**
   * Cancel the active session, including one that has not started yet.
   *
   * @returns `true` when an owned session or armed pointer was settled.
   * @throws An error carrying `VIZE_UI_DRAG_AND_DROP_DISPOSED` after disposal.
   */
  readonly cancel: () => boolean;

  /** Release listeners, the live region, and reactive state without callbacks. */
  readonly dispose: () => void;
}

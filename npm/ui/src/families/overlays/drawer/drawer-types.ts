import type {
  DialogAutoFocusEvent,
  DialogCloseExpose,
  DialogDescriptionExpose,
  DialogTitleExpose,
  DialogTriggerExpose,
} from "../dialog/dialog-types.ts";
import type {
  DismissableLayerEscapeKeyDownEvent,
  DismissableLayerPointerDownOutsideEvent,
} from "../dismissable-layer/dismissable-layer.ts";

/** Viewport edge the drawer is attached to and slides out of. */
export type DrawerSide = "bottom" | "left" | "right" | "top";

/**
 * One resting position of the drawer.
 *
 * A number in `(0, 1]` is the fraction of the drawer's own size left visible;
 * a `${number}px` string is an absolute visible size in CSS pixels.
 */
export type DrawerSnapPoint = number | `${number}px`;

/** Open state mirrored to the Drawer data contract. */
export type DrawerState = "closed" | "open";

/** What ended a drag gesture. */
export type DrawerDragOutcome = "cancel" | "dismiss" | "snap";

/** Preventable Drawer auto-focus lifecycle event. */
export type DrawerAutoFocusEvent = DialogAutoFocusEvent;

/** Preventable Escape lifecycle event emitted by DrawerContent. */
export type DrawerEscapeKeyDownEvent = DismissableLayerEscapeKeyDownEvent;

/** Preventable outside pointer event emitted by non-modal DrawerContent. */
export type DrawerPointerDownOutsideEvent = DismissableLayerPointerDownOutsideEvent;

/** Why the drawer requested dismissal. */
export type DrawerDismissReason = "backdrop" | "escape-key" | "pointer-down-outside";

/** Notification emitted after an unprevented dismissal request. */
export interface DrawerDismissEvent {
  /** User action that requested dismissal. */
  readonly reason: DrawerDismissReason;

  /** Native event that caused the request, when one exists. */
  readonly originalEvent: Event | null;
}

/** Preventable notification emitted when the backdrop of a modal drawer is pressed. */
export interface DrawerBackdropPointerDownEvent {
  /** Native pointer event targeting the `<dialog>` backdrop. */
  readonly originalEvent: PointerEvent | MouseEvent;

  /** Whether a listener cancelled dismissal. */
  readonly defaultPrevented: boolean;

  /** Keep the drawer open. */
  readonly preventDefault: () => void;
}

/** Summary of a completed drag gesture. */
export interface DrawerDragEndEvent {
  /** Whether the gesture snapped, dismissed, or was cancelled by the platform. */
  readonly outcome: DrawerDragOutcome;

  /** Signed travel in px toward the dismiss direction. */
  readonly distance: number;

  /** Release velocity in px/ms toward the dismiss direction. */
  readonly velocity: number;

  /** Snap point the drawer rests on afterwards (`null` without snap points or on dismiss). */
  readonly snapPoint: DrawerSnapPoint | null;
}

/** State exposed to compound Drawer slots. */
export interface DrawerSlotState {
  /** Whether the drawer is open. */
  readonly open: boolean;

  /** Whether the drawer is shown as a modal `<dialog>`. */
  readonly modal: boolean;

  /** Stable state token for styling and tests. */
  readonly state: DrawerState;

  /** Viewport edge the drawer is attached to. */
  readonly side: DrawerSide;

  /** Snap point the drawer currently rests on, or `null` without snap points. */
  readonly activeSnapPoint: DrawerSnapPoint | null;

  /** Whether a drag gesture is in progress. */
  readonly dragging: boolean;
}

/** Public instance exposed by DrawerRoot. */
export interface DrawerRootExpose extends DrawerSlotState {
  /** Root-owned base id. */
  readonly id: string;

  /** Id of the rendered `<dialog>`. */
  readonly contentId: string;

  /** Default id consumed by DrawerTitle. */
  readonly titleId: string;

  /** Default id consumed by DrawerDescription. */
  readonly descriptionId: string;

  /** Configured snap points in consumer order. */
  readonly snapPoints: readonly DrawerSnapPoint[];

  /** Request a specific open value and report whether it differs. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Request the open state. */
  readonly openDrawer: (event?: Event | null) => boolean;

  /** Request the closed state. */
  readonly close: (event?: Event | null) => boolean;

  /** Request the opposite open state. */
  readonly toggle: (event?: Event | null) => boolean;

  /** Request a snap point and report whether it differs. Unknown values are ignored. */
  readonly snapTo: (snapPoint: DrawerSnapPoint, event?: Event | null) => boolean;
}

/** Public instance exposed by DrawerContent. */
export interface DrawerContentExpose extends DrawerSlotState {
  /** Rendered native dialog element. */
  readonly element: HTMLDialogElement | null;

  /** Resolved snap offset in px toward the dismiss direction. */
  readonly snapOffset: number;

  /** Current drag offset in px toward the dismiss direction. */
  readonly dragOffset: number;

  /** Move focus to the dialog element. */
  readonly focusContent: (options?: FocusOptions) => void;
}

/** Public instance exposed by DrawerHandle. */
export interface DrawerHandleExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the handle. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by DrawerTrigger. */
export type DrawerTriggerExpose = DialogTriggerExpose;

/** Public instance exposed by DrawerTitle. */
export type DrawerTitleExpose = DialogTitleExpose;

/** Public instance exposed by DrawerDescription. */
export type DrawerDescriptionExpose = DialogDescriptionExpose;

/** Public instance exposed by DrawerClose. */
export type DrawerCloseExpose = DialogCloseExpose;

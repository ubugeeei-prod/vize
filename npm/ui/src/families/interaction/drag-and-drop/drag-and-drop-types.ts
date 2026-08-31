/** Input families that can own a drag session. */
export type DragPointerType = "keyboard" | "mouse" | "pen" | "pointer" | "touch";

/** Drop position relative to a target, used for indicators and placeholders. */
export type DropEdge = "bottom" | "inside" | "left" | "right" | "top";

/** Lifecycle phases emitted by a drag-and-drop controller. */
export type DragEventType = "dragend" | "dragmove" | "dragstart";

/** Target-scoped phases delivered to drop-target callbacks. */
export type DropTargetEventType = "drop" | "dropenter" | "dropleave" | "dropmove";

/** Serializable, axis-aligned CSS pixel rectangle. */
export interface DropTargetRect {
  readonly bottom: number;
  readonly left: number;
  readonly right: number;
  readonly top: number;
}

/** Typed payload transferred from a drag source to a drop target. */
export interface DragPayload<Data = unknown> {
  /** Consumer-defined discriminant such as a MIME type or model name. */
  readonly kind: string;

  /** Structured payload delivered verbatim to accepting drop targets. */
  readonly data: Data;

  /** Optional plain-text projection for clipboard and data-transfer adapters. */
  readonly plainText?: string;
}

/** Immutable snapshot shared by every drag lifecycle phase. */
export interface DragLifecycleEvent<Data = unknown, Type extends DragEventType = DragEventType> {
  /** Lifecycle phase represented by this immutable snapshot. */
  readonly type: Type;

  /** Input family that owns this session. */
  readonly pointerType: DragPointerType;

  /** Key of the source that owns this session. */
  readonly sourceKey: string;

  /** Payload resolved when the session started, or `null` when none was declared. */
  readonly payload: DragPayload<Data> | null;

  /** Key of the drop target currently under the drag, or `null`. */
  readonly targetKey: string | null;

  /** Drop edge resolved for the current target, or `null`. */
  readonly edge: DropEdge | null;

  /** Client coordinates when supplied by pointing hardware. */
  readonly x: number | null;
  readonly y: number | null;

  /** Native event responsible for this phase, or `null` for manual settlement. */
  readonly originalEvent: Event | null;

  /** Whether the session settled without a drop. Always `false` before `dragend`. */
  readonly isCanceled: boolean;
}

/** Snapshot emitted once when a session starts. */
export type DragStartEvent<Data = unknown> = DragLifecycleEvent<Data, "dragstart">;

/** Snapshot emitted for movement and target changes inside a session. */
export type DragMoveEvent<Data = unknown> = DragLifecycleEvent<Data, "dragmove">;

/** Snapshot emitted after a session drops or cancels. */
export type DragEndEvent<Data = unknown> = DragLifecycleEvent<Data, "dragend">;

/** Immutable snapshot delivered to one drop target's callbacks. */
export interface DropTargetEvent<Data = unknown> {
  /** Target-scoped phase represented by this immutable snapshot. */
  readonly type: DropTargetEventType;

  /** Key of the target receiving this phase. */
  readonly targetKey: string;

  /** Key of the source that owns the session. */
  readonly sourceKey: string;

  /** Input family that owns the session. */
  readonly pointerType: DragPointerType;

  /** Payload resolved when the session started, or `null` when none was declared. */
  readonly payload: DragPayload<Data> | null;

  /** Drop edge resolved against this target, or `null` after leaving. */
  readonly edge: DropEdge | null;

  /** Client coordinates when supplied by pointing hardware. */
  readonly x: number | null;
  readonly y: number | null;

  /** Native event responsible for this phase, or `null` for manual settlement. */
  readonly originalEvent: Event | null;
}

/** Reactive indicator state for the target currently under the drag. */
export interface DropIndicatorState {
  /** Key of the target the indicator belongs to. */
  readonly targetKey: string;

  /** Drop edge the indicator represents. */
  readonly edge: DropEdge;

  /** Full target rectangle for placeholder geometry, or `null` when unmeasurable. */
  readonly rect: DropTargetRect | null;

  /** Zero-thickness line rectangle along the edge; `null` for `"inside"` drops. */
  readonly line: DropTargetRect | null;
}

/** Context handed to announcement builders before speaking to assistive tech. */
export interface DragAnnouncementContext<Data = unknown> {
  readonly pointerType: DragPointerType;
  readonly sourceKey: string;
  readonly sourceLabel: string;
  readonly payload: DragPayload<Data> | null;
  readonly targetKey: string | null;
  readonly targetLabel: string | null;

  /** One-based position of the current target among valid targets, or `null`. */
  readonly targetIndex: number | null;
  readonly targetCount: number | null;
  readonly edge: DropEdge | null;
}

/** Localizable builders for grab, move, drop, and cancel announcements. */
export interface DragAnnouncements<Data = unknown> {
  /** Spoken when a session starts. Return `null` to stay silent. */
  readonly grab?: (context: DragAnnouncementContext<Data>) => string | null;

  /** Spoken when the drag enters a target or its edge changes. */
  readonly move?: (context: DragAnnouncementContext<Data>) => string | null;

  /** Spoken after a completed drop. */
  readonly drop?: (context: DragAnnouncementContext<Data>) => string | null;

  /** Spoken after a canceled session. */
  readonly cancel?: (context: DragAnnouncementContext<Data>) => string | null;
}

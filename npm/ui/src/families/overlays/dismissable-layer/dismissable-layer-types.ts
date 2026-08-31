import type { MaybeRefOrGetter, ShallowRef } from "vue";

/** Stable pointing-device family captured from an outside pointer event. */
export type DismissableLayerPointerType = "mouse" | "pen" | "pointer" | "touch" | "virtual";

/** Outside interaction that can request dismissal. */
export type DismissableLayerOutsideReason = "focus-outside" | "pointer-down-outside";

/** User action that can request dismissal. */
export type DismissableLayerDismissReason = DismissableLayerOutsideReason | "escape-key";

/** Preventable, immutable snapshot emitted before a pointer dismissal is committed. */
export interface DismissableLayerPointerDownOutsideEvent {
  readonly type: "pointer-down-outside";
  readonly reason: "pointer-down-outside";
  readonly target: Element;
  readonly originalEvent: PointerEvent | MouseEvent | TouchEvent;
  readonly pointerType: DismissableLayerPointerType;
  readonly x: number | null;
  readonly y: number | null;
  readonly altKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;
  readonly defaultPrevented: boolean;
  readonly preventDefault: () => void;
}

/** Preventable, immutable snapshot emitted before a focus dismissal is committed. */
export interface DismissableLayerFocusOutsideEvent {
  readonly type: "focus-outside";
  readonly reason: "focus-outside";
  readonly target: Element;
  readonly relatedTarget: Element | null;
  readonly originalEvent: FocusEvent;
  readonly defaultPrevented: boolean;
  readonly preventDefault: () => void;
}

/** Preventable, immutable snapshot emitted before an Escape dismissal is committed. */
export interface DismissableLayerEscapeKeyDownEvent {
  readonly type: "escape-key";
  readonly reason: "escape-key";
  readonly target: Element | null;
  readonly originalEvent: KeyboardEvent;
  readonly altKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly shiftKey: boolean;
  readonly defaultPrevented: boolean;
  readonly preventDefault: () => void;
}

/** Preventable outside interaction snapshot shared by pointer and focus callbacks. */
export type DismissableLayerInteractOutsideEvent =
  | DismissableLayerFocusOutsideEvent
  | DismissableLayerPointerDownOutsideEvent;

/** Immutable notification emitted after dismissal has not been prevented. */
export interface DismissableLayerDismissEvent {
  readonly type: "dismiss";
  readonly reason: DismissableLayerDismissReason;
  readonly target: Element | null;
  readonly originalEvent: Event;
}

/** Native props to merge onto the rendered layer root. */
export interface DismissableLayerProps {
  readonly "data-vize-dismissable-layer": "";
}

/** Native props to merge onto optional rendered branch roots. */
export interface DismissableLayerBranchProps {
  readonly "data-vize-dismissable-branch": "";
}

/** Reactive configuration for one dismissable overlay layer. */
export interface DismissableLayerOptions {
  /** Primary layer root. It may be `null` during SSR and before mount. */
  readonly root: MaybeRefOrGetter<Element | null | undefined>;

  /** Portalled or exceptional roots that count as inside this layer. @default [] */
  readonly branches?: MaybeRefOrGetter<readonly Element[] | null | undefined>;

  /** Whether an activated layer participates in the document dismissal stack. @default true */
  readonly enabled?: MaybeRefOrGetter<boolean | undefined>;

  /** Whether outside pointer-down events can request dismissal. @default true */
  readonly outsidePointerDown?: MaybeRefOrGetter<boolean | undefined>;

  /** Whether outside focus movement can request dismissal. @default true */
  readonly outsideFocus?: MaybeRefOrGetter<boolean | undefined>;

  /** Whether non-composing Escape keydown can request dismissal. @default true */
  readonly escapeKey?: MaybeRefOrGetter<boolean | undefined>;

  /** Called before an outside pointer-down dismissal is committed. */
  readonly onPointerDownOutside?: (event: DismissableLayerPointerDownOutsideEvent) => void;

  /** Called before an outside focus dismissal is committed. */
  readonly onFocusOutside?: (event: DismissableLayerFocusOutsideEvent) => void;

  /** Called for every outside pointer or focus interaction before dismissal is committed. */
  readonly onInteractOutside?: (event: DismissableLayerInteractOutsideEvent) => void;

  /** Called before an Escape dismissal is committed. */
  readonly onEscapeKeyDown?: (event: DismissableLayerEscapeKeyDownEvent) => void;

  /** Called after a top-layer dismissal request has not been prevented. */
  readonly onDismiss?: (event: DismissableLayerDismissEvent) => void;
}

/** Lifecycle, state, and root/branch props for one dismissable layer. */
export interface DismissableLayerController {
  /** Whether this controller has been activated, independently of reactive enablement. */
  readonly isActive: Readonly<ShallowRef<boolean>>;

  /** Whether this layer is the enabled, connected, topmost owner in its document. */
  readonly isTopLayer: Readonly<ShallowRef<boolean>>;

  /** Stable native props to merge onto the rendered layer root. */
  readonly layerProps: Readonly<DismissableLayerProps>;

  /** Stable native props to merge onto optional rendered branch roots. */
  readonly branchProps: Readonly<DismissableLayerBranchProps>;

  /** Register an imperative branch and receive an idempotent release function. */
  readonly registerBranch: (branch: Element) => () => void;

  /** Join the document dismissal stack. Safe to call repeatedly. */
  readonly activate: () => void;

  /** Leave the document dismissal stack. */
  readonly deactivate: () => void;

  /** Re-read roots and options after imperative DOM movement. */
  readonly refresh: () => void;

  /** Permanently release reactive and document-level ownership. */
  readonly dispose: () => void;
}

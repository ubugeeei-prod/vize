import type { MaybeRefOrGetter } from "vue";

import type { Placement, PlacementAlign, PlacementSide } from "../positioner/positioner.ts";

/** Open state token published on every Tour part. */
export type TourState = "closed" | "open";

/**
 * Element a tour step highlights.
 *
 * A string is resolved with `document.querySelector` on the client. Elements,
 * refs, and getters are read with `toValue` so template refs work directly.
 * Resolution never runs during SSR.
 */
export type TourTarget = string | MaybeRefOrGetter<Element | null | undefined>;

/** What a step does when its declared target cannot be resolved on the client. */
export type TourMissingTargetBehavior = "center" | "skip";

/**
 * Resolution status of the current step target.
 *
 * - `none`: the step declares no target and renders centered.
 * - `pending`: the target has not been resolved yet (SSR and before mount).
 * - `resolved`: the target element exists and anchors the content.
 * - `missing`: the target was declared but not found; the content renders centered.
 */
export type TourTargetState = "missing" | "none" | "pending" | "resolved";

/** Placement token published by TourContent, including the centered fallback. */
export type TourContentPlacement = Placement | "center";

/** Why an open tour closed without completing. */
export type TourDismissReason =
  | "close"
  | "escape-key"
  | "focus-outside"
  | "pointer-down-outside"
  | "skip";

/** Reason a TourClose button reports when activated. */
export type TourCloseReason = "close" | "skip";

/** Reading direction used to map ArrowLeft and ArrowRight to previous and next. */
export type TourDirection = "ltr" | "rtl";

/**
 * One step definition. Consumers may extend it with their own fields (title,
 * body, media, analytics ids); TourRoot infers the exact step type and hands
 * it back through slot state.
 */
export interface TourStepDefinition {
  /** Unique step value used by `v-model:step`. */
  readonly value: string;

  /**
   * Element highlighted by this step. `undefined` renders a centered step.
   *
   * @default undefined
   */
  readonly target?: TourTarget;

  /**
   * Preferred placement of TourContent against the target.
   *
   * @default the TourRoot `placement`
   */
  readonly placement?: Placement;

  /**
   * Remove this step from the sequence without deleting its definition.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Per-step override of the TourRoot `missingTarget` behavior.
   *
   * @default the TourRoot `missingTarget`
   */
  readonly missingTarget?: TourMissingTargetBehavior;
}

/** Step value union inferred from a step definition type. */
export type TourStepValue<Step extends TourStepDefinition = TourStepDefinition> = Step["value"];

/** State exposed to the TourRoot default slot. */
export interface TourSlotState<Step extends TourStepDefinition = TourStepDefinition> {
  /** Whether the tour is open. */
  readonly open: boolean;

  /** Stable open state token. */
  readonly state: TourState;

  /** Current step definition, or `null` when no enabled step exists. */
  readonly step: Step | null;

  /** Current step value, or `null` when no enabled step exists. */
  readonly value: TourStepValue<Step> | null;

  /** Zero-based position of the current step among enabled steps, or `-1`. */
  readonly index: number;

  /** Number of enabled steps. */
  readonly total: number;

  /** Whether the current step is the first enabled step. */
  readonly first: boolean;

  /** Whether the current step is the last enabled step. */
  readonly last: boolean;

  /** Resolution status of the current step target. */
  readonly targetState: TourTargetState;
}

/** State exposed to TourContent slots. */
export interface TourContentSlotState extends TourSlotState {
  /** Resolved placement, or `center` when no target anchors the content. */
  readonly placement: TourContentPlacement;

  /** Side of the resolved placement, or `center`. */
  readonly side: PlacementSide | "center";

  /** Alignment of the resolved placement, or `center`. */
  readonly align: PlacementAlign;
}

/** State exposed to TourStep slots. */
export interface TourStepSlotState {
  /** Step value this part renders for. */
  readonly value: string;

  /** Zero-based position of this step among enabled steps, or `-1`. */
  readonly index: number;

  /** Number of enabled steps. */
  readonly total: number;
}

/** State exposed to TourProgress slots. */
export interface TourProgressSlotState {
  /** Zero-based position of the current step, or `-1`. */
  readonly index: number;

  /** One-based position of the current step, or `0`. */
  readonly current: number;

  /** Number of enabled steps. */
  readonly total: number;

  /** Completed fraction from `0` to `1`, counting the current step as reached. */
  readonly fraction: number;
}

/** State exposed to TourPrev, TourNext, and TourClose slots. */
export interface TourControlSlotState {
  /** Whether the control is disabled. */
  readonly disabled: boolean;

  /** Whether the current step is the first enabled step. */
  readonly first: boolean;

  /** Whether the current step is the last enabled step. TourNext completes the tour there. */
  readonly last: boolean;

  /** Stable open state token. */
  readonly state: TourState;
}

/** State exposed to TourSpotlight slots. */
export interface TourSpotlightSlotState {
  /** Stable open state token. */
  readonly state: TourState;

  /** Resolution status of the current step target. */
  readonly targetState: TourTargetState;

  /** Padded target box in viewport pixels, or `null` when nothing is highlighted. */
  readonly rect: TourSpotlightRect | null;
}

/** Padded viewport box around the highlighted target. */
export interface TourSpotlightRect {
  /** Left edge in viewport pixels. */
  readonly x: number;

  /** Top edge in viewport pixels. */
  readonly y: number;

  /** Box width in pixels. */
  readonly width: number;

  /** Box height in pixels. */
  readonly height: number;
}

/** State exposed to TourArrow slots. */
export interface TourArrowSlotState {
  /** Arrow x coordinate relative to the content, when measured. */
  readonly x: number | null;

  /** Arrow y coordinate relative to the content, when measured. */
  readonly y: number | null;
}

/** Public instance exposed by TourRoot. */
export interface TourRootExpose<
  Step extends TourStepDefinition = TourStepDefinition,
> extends TourSlotState<Step> {
  /** Root-owned base id for the Tour family. */
  readonly id: string;

  /** Currently resolved target element, if any. */
  readonly target: Element | null;

  /** Open the tour at the first enabled step. Reports whether a step could be entered. */
  readonly start: (event?: Event | null) => boolean;

  /** Request an open state. Opening resumes at the current step. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Move to the next enterable step, or complete the tour on the last step. */
  readonly next: (event?: Event | null) => boolean;

  /** Move to the previous enterable step. */
  readonly previous: (event?: Event | null) => boolean;

  /** Move to an enabled step by value, regardless of target availability. */
  readonly goTo: (value: TourStepValue<Step>, event?: Event | null) => boolean;

  /** Close the tour with a dismissal reason. */
  readonly dismiss: (reason?: TourDismissReason, event?: Event | null) => boolean;

  /** Close the tour as completed. */
  readonly complete: (event?: Event | null) => boolean;

  /** Re-resolve the current target, for targets rendered after the step became active. */
  readonly refresh: () => void;
}

/** Public instance exposed by TourContent. */
export interface TourContentExpose {
  /** Rendered dialog element while mounted. */
  readonly element: HTMLDivElement | null;

  /** Stable open state token. */
  readonly state: TourState;

  /** Resolved placement, or `center`. */
  readonly placement: TourContentPlacement;

  /** Move focus to the dialog element. */
  readonly focusContent: (options?: FocusOptions) => void;

  /** Recompute the floating position against the current target. */
  readonly update: () => void;
}

/** Public instance exposed by TourSpotlight. */
export interface TourSpotlightExpose {
  /** Rendered overlay element while open. */
  readonly element: HTMLDivElement | null;

  /** Padded target box, or `null`. */
  readonly rect: TourSpotlightRect | null;

  /** Re-measure the target box. */
  readonly update: () => void;
}

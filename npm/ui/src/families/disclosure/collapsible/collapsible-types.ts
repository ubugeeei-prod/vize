/** Open state mirrored to the Collapsible data contract. */
export type CollapsibleState = "closed" | "open";

/** Optional landmark role used by CollapsibleContent. */
export type CollapsibleContentRole = "group" | "region";

/** State exposed to compound Collapsible slots. */
export interface CollapsibleSlotState {
  /** Whether the controlled content is currently visible. */
  readonly open: boolean;

  /** Whether trigger activation is disabled by the root. */
  readonly disabled: boolean;

  /** Stable state token for styling and tests. */
  readonly state: CollapsibleState;
}

/** Public instance exposed by CollapsibleRoot. */
export interface CollapsibleRootExpose extends CollapsibleSlotState {
  /** Root-owned base id for the disclosure family. */
  readonly id: string;

  /** Id wired to the native trigger button. */
  readonly triggerId: string;

  /** Id wired from CollapsibleTrigger to CollapsibleContent. */
  readonly contentId: string;

  /** Request a specific open value and report whether it differs. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Request the open state. */
  readonly expand: (event?: Event | null) => boolean;

  /** Request the closed state. */
  readonly collapse: (event?: Event | null) => boolean;

  /** Request the opposite open state. */
  readonly toggle: (event?: Event | null) => boolean;
}

/** Public instance exposed by CollapsibleTrigger. */
export interface CollapsibleTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by CollapsibleContent. */
export interface CollapsibleContentExpose extends CollapsibleSlotState {
  /** Rendered content region. */
  readonly element: HTMLDivElement | null;

  /** Move focus to the content element when it can receive focus. */
  readonly focusContent: (options?: FocusOptions) => void;
}

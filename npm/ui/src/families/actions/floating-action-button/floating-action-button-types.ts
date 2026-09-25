/** Viewport corner or edge a floating action button is anchored to. */
export type FloatingActionButtonPlacement =
  | "bottom-center"
  | "bottom-end"
  | "bottom-start"
  | "top-center"
  | "top-end"
  | "top-start";

/** Direction speed-dial actions fan out from the trigger. */
export type SpeedDialDirection = "down" | "left" | "right" | "up";

/** Open state mirrored to the SpeedDial data contract. */
export type SpeedDialState = "closed" | "open";

/** Why the speed dial last changed its open state. */
export type SpeedDialChangeReason =
  | "action"
  | "escape"
  | "hover"
  | "keyboard"
  | "outside"
  | "pointer"
  | "programmatic";

/** State exposed to FloatingActionButton slots. */
export interface FloatingActionButtonSlotState {
  /** Anchored placement. */
  readonly placement: FloatingActionButtonPlacement;

  /** Whether the button shows its text label next to the icon. */
  readonly extended: boolean;
}

/** Public instance exposed by FloatingActionButton. */
export interface FloatingActionButtonExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the button. */
  readonly focus: (options?: FocusOptions) => void;
}

/** State exposed to SpeedDial slots. */
export interface SpeedDialSlotState {
  /** Whether the actions are visible. */
  readonly open: boolean;

  /** Stable state token for styling and tests. */
  readonly state: SpeedDialState;

  /** Direction the actions fan out. */
  readonly direction: SpeedDialDirection;

  /** Whether the whole speed dial is disabled. */
  readonly disabled: boolean;
}

/** State exposed to SpeedDialAction slots. */
export interface SpeedDialActionSlotState extends SpeedDialSlotState {
  /** Action identity. */
  readonly value: string;

  /** Accessible label of the action. */
  readonly label: string;

  /** Whether this action currently owns the roving tab stop. */
  readonly active: boolean;
}

/** Cancelable action selection event. */
export interface SpeedDialSelectEvent {
  /** Selected action value. */
  readonly value: string;

  /** Native event that triggered the selection. */
  readonly originalEvent: Event;

  /** Whether `preventDefault()` was called to keep the speed dial open. */
  readonly defaultPrevented: boolean;

  /** Keep the speed dial open after this selection. */
  readonly preventDefault: () => void;
}

/** Public instance exposed by SpeedDialRoot. */
export interface SpeedDialRootExpose extends SpeedDialSlotState {
  /** Root-owned base id. */
  readonly id: string;

  /** Id wired to the trigger. */
  readonly triggerId: string;

  /** Id wired to the action menu. */
  readonly contentId: string;

  /** Request a specific open value. */
  readonly setOpen: (value: boolean, event?: Event | null) => boolean;

  /** Open and move focus to the first enabled action. */
  readonly openAndFocus: (event?: Event | null) => boolean;

  /** Close and optionally return focus to the trigger. */
  readonly close: (options?: { readonly focusTrigger?: boolean }) => boolean;
}

/** Public instance exposed by SpeedDialTrigger. */
export interface SpeedDialTriggerExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by SpeedDialContent. */
export interface SpeedDialContentExpose extends SpeedDialSlotState {
  /** Rendered menu element. */
  readonly element: HTMLDivElement | null;
}

/** Public instance exposed by SpeedDialAction. */
export interface SpeedDialActionExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the action. */
  readonly focus: (options?: FocusOptions) => void;
}

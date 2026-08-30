import type { ComputedRef } from "vue";

/** Current value held by a Stepper root. `null` represents no current step. */
export type StepperValue = string | null;

/** Whether step activation follows a linear completion gate or remains free-form. */
export type StepperNavigationMode = "free" | "linear";

/** Directional layout hint used by step trigger keyboard navigation. */
export type StepperOrientation = "horizontal" | "vertical";

/** Reading direction used to map horizontal arrow keys. */
export type StepperDirection = "ltr" | "rtl";

/** State exposed by StepperRoot and StepperList. */
export type StepperRootState = "active" | "disabled" | "empty";

/** Progress state exposed by StepperItem and StepperTrigger. */
export type StepperItemState = "completed" | "current" | "disabled" | "pending";

/** Visibility state exposed by StepperContent. */
export type StepperContentState = "active" | "inactive";

/** Optional landmark role used by StepperContent. */
export type StepperContentRole = "group" | "region";

/** Public props accepted by StepperRoot. */
export interface StepperRootProps {
  /**
   * Consumer-owned Stepper base id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Controlled current step value. `undefined` selects uncontrolled behavior; `null` clears it.
   *
   * @default undefined
   */
  readonly modelValue?: StepperValue;

  /**
   * Initial current step for uncontrolled use. `undefined` selects the first enabled step.
   *
   * @default undefined
   */
  readonly defaultValue?: StepperValue;

  /**
   * Disable every step trigger while preserving the current content panel.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Whether future steps require every previous enabled step to be completed.
   *
   * @default "linear"
   */
  readonly navigationMode?: StepperNavigationMode;

  /**
   * Directional layout hint used by arrow-key navigation.
   *
   * @default "horizontal"
   */
  readonly orientation?: StepperOrientation;

  /**
   * Reading direction used for horizontal arrow-key navigation.
   *
   * @default "ltr"
   */
  readonly dir?: StepperDirection;

  /**
   * Whether arrow-key navigation wraps at the first and last enabled trigger.
   *
   * @default false
   */
  readonly loop?: boolean;
}

/** State exposed to StepperRoot default slots. */
export interface StepperSlotState {
  /** Current step value, or `null` when no step is current. */
  readonly value: StepperValue;

  /** Values whose StepperItem is marked completed. */
  readonly completedValues: readonly string[];

  /** Current step index within all registered items, or `-1` when empty or missing. */
  readonly currentIndex: number;

  /** Whether every trigger is disabled by the root. */
  readonly disabled: boolean;

  /** Whether step activation is linear or free-form. */
  readonly navigationMode: StepperNavigationMode;

  /** Directional layout hint for ARIA and consumer-owned styles. */
  readonly orientation: StepperOrientation;

  /** Reading direction used for horizontal keyboard navigation. */
  readonly dir: StepperDirection;

  /** Stable state token for styling and tests. */
  readonly state: StepperRootState;
}

/** State exposed to StepperList default slots. */
export interface StepperListSlotState extends StepperSlotState {
  /** Deterministic id assigned to the list. */
  readonly listId: string;
}

/** State exposed to StepperItem and StepperTrigger slots. */
export interface StepperItemSlotState {
  /** Step value used for current-step state and collection identity. */
  readonly value: string;

  /** Index within all registered StepperItem instances. */
  readonly index: number;

  /** Whether this step is the current step. */
  readonly current: boolean;

  /** Whether this step is marked completed. */
  readonly completed: boolean;

  /** Whether this step or its root is disabled. */
  readonly disabled: boolean;

  /** Whether this step may be activated under the current navigation mode. */
  readonly selectable: boolean;

  /** Whether a linear Stepper is preventing activation of this otherwise enabled step. */
  readonly locked: boolean;

  /** Directional layout hint inherited from the Stepper root. */
  readonly orientation: StepperOrientation;

  /** Whether step activation is linear or free-form. */
  readonly navigationMode: StepperNavigationMode;

  /** Stable state token for styling and tests. */
  readonly state: StepperItemState;
}

/** State exposed to StepperTrigger slots. */
export interface StepperTriggerSlotState extends StepperItemSlotState {}

/** State exposed to StepperContent default slots. */
export interface StepperContentSlotState {
  /** Content value paired with a StepperItem and StepperTrigger. */
  readonly value: string;

  /** Whether this content panel is paired with the current step. */
  readonly current: boolean;

  /** Whether this content panel is currently visible. */
  readonly active: boolean;

  /** Whether the paired step is marked completed. */
  readonly completed: boolean;

  /** Whether the Stepper root or paired item is disabled. */
  readonly disabled: boolean;

  /** Directional layout hint inherited from the Stepper root. */
  readonly orientation: StepperOrientation;

  /** Stable state token for styling and tests. */
  readonly state: StepperContentState;
}

/** Public instance exposed by StepperRoot. */
export interface StepperRootExpose extends StepperSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Root-owned base id for the Stepper family. */
  readonly id: string;

  /** Id wired to StepperList. */
  readonly listId: string;

  /** Move focus to the current, active, or first enabled trigger. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a current value update and report whether it differs and is allowed. */
  readonly setValue: (value: StepperValue, event?: Event | null) => boolean;

  /** Request a current step by string value. */
  readonly selectValue: (value: string, event?: Event | null) => boolean;

  /** Request the next enabled step. */
  readonly next: (event?: Event | null) => boolean;

  /** Request the previous enabled step. */
  readonly previous: (event?: Event | null) => boolean;

  /** Restore the default value, or the first enabled step when no default exists. */
  readonly reset: () => boolean;

  /** Return whether a step can currently be activated. */
  readonly isSelectable: (value: string) => boolean;
}

/** Internal setup shape used to type StepperRoot refs before Vue unwraps them. */
export type StepperRootSetupExpose<ElementRef> = Omit<
  StepperRootExpose,
  keyof StepperSlotState | "element" | "id" | "listId"
> & {
  readonly completedValues: ComputedRef<readonly string[]>;
  readonly currentIndex: ComputedRef<number>;
  readonly dir: ComputedRef<StepperDirection>;
  readonly disabled: ComputedRef<boolean>;
  readonly element: ElementRef;
  readonly id: ComputedRef<string>;
  readonly listId: ComputedRef<string>;
  readonly navigationMode: ComputedRef<StepperNavigationMode>;
  readonly orientation: ComputedRef<StepperOrientation>;
  readonly state: ComputedRef<StepperRootState>;
  readonly value: ComputedRef<StepperValue>;
};

/** Public instance exposed by StepperList. */
export interface StepperListExpose extends StepperListSlotState {
  /** Rendered list element. */
  readonly element: HTMLDivElement | null;

  /** Move focus to the current, active, or first enabled trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by StepperItem. */
export interface StepperItemExpose extends StepperItemSlotState {
  /** Rendered list item element. */
  readonly element: HTMLDivElement | null;

  /** Deterministic id assigned to the item. */
  readonly id: string;

  /** Deterministic id wired from trigger to content. */
  readonly triggerId: string;

  /** Deterministic id wired from content to trigger. */
  readonly contentId: string;

  /** Move focus to this step's trigger when it can receive focus. */
  readonly focus: (options?: FocusOptions) => boolean;

  /** Request this step as the current value. */
  readonly select: () => boolean;
}

/** Public instance exposed by StepperTrigger. */
export interface StepperTriggerExpose extends StepperTriggerSlotState {
  /** Rendered native trigger button. */
  readonly element: HTMLButtonElement | null;

  /** Deterministic id wired from trigger to content. */
  readonly id: string;

  /** Deterministic id of the controlled content panel. */
  readonly contentId: string;

  /** Move focus to the native trigger button. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request this step as the current value. */
  readonly select: () => boolean;
}

/** Public instance exposed by StepperContent. */
export interface StepperContentExpose extends StepperContentSlotState {
  /** Rendered content panel element. */
  readonly element: HTMLDivElement | null;

  /** Deterministic id wired from content to trigger. */
  readonly id: string;

  /** Deterministic id of the controlling trigger. */
  readonly triggerId: string;

  /** Move focus to the visible content panel. */
  readonly focusContent: (options?: FocusOptions) => void;
}

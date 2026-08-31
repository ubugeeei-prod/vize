/** Selected value held by a Tabs root. `null` represents no selected tab. */
export type TabsValue = string | null;

/** Whether arrow focus also activates the newly focused tab. */
export type TabsActivationMode = "automatic" | "manual";

/** Directional layout hint used by tablist arrow-key navigation. */
export type TabsOrientation = "horizontal" | "vertical";

/** Reading direction used to map horizontal arrow keys. */
export type TabsDirection = "ltr" | "rtl";

/** State exposed by the Tabs root and list data contract. */
export type TabsState = "disabled" | "empty" | "selected";

/** State exposed by each TabsTrigger data contract. */
export type TabsTriggerState = "active" | "disabled" | "inactive";

/** State exposed by each TabsContent data contract. */
export type TabsContentState = "active" | "inactive";

/** State exposed to the TabsRoot default slot. */
export interface TabsSlotState {
  /** Current selected value, or `null` when no tab is selected. */
  readonly value: TabsValue;

  /** Whether the root suppresses all trigger activation and roving focus. */
  readonly disabled: boolean;

  /** Whether focus movement activates tabs automatically. */
  readonly activationMode: TabsActivationMode;

  /** Directional layout hint used by tablist keyboard navigation. */
  readonly orientation: TabsOrientation;

  /** Reading direction used for horizontal arrow navigation. */
  readonly dir: TabsDirection;

  /** Stable state token for styling and tests. */
  readonly state: TabsState;
}

/** State exposed to the TabsList default slot. */
export interface TabsListSlotState extends TabsSlotState {
  /** Deterministic id assigned to the tablist. */
  readonly listId: string;
}

/** State exposed to TabsTrigger slots. */
export interface TabsTriggerSlotState {
  /** Trigger value used by the Tabs selection model. */
  readonly value: string;

  /** Whether this trigger controls the selected tabpanel. */
  readonly selected: boolean;

  /** Whether this trigger or its root suppresses activation. */
  readonly disabled: boolean;

  /** Whether focus movement activates tabs automatically. */
  readonly activationMode: TabsActivationMode;

  /** Directional layout hint inherited from the Tabs root. */
  readonly orientation: TabsOrientation;

  /** Stable state token for styling and tests. */
  readonly state: TabsTriggerState;
}

/** State exposed to the TabsContent default slot. */
export interface TabsContentSlotState {
  /** Content value paired with a TabsTrigger. */
  readonly value: string;

  /** Whether this panel is currently selected and visible. */
  readonly selected: boolean;

  /** Whether the Tabs root suppresses trigger activation. */
  readonly disabled: boolean;

  /** Directional layout hint inherited from the Tabs root. */
  readonly orientation: TabsOrientation;

  /** Stable state token for styling and tests. */
  readonly state: TabsContentState;
}

/** Public instance exposed by TabsRoot. */
export interface TabsRootExpose extends TabsSlotState {
  /** Rendered root element. */
  readonly element: HTMLDivElement | null;

  /** Root-owned base id for the Tabs family. */
  readonly id: string;

  /** Id wired to TabsList. */
  readonly listId: string;

  /** Move focus to the selected, active, or first enabled trigger. */
  readonly focus: (options?: FocusOptions) => void;

  /** Request a selected value update and report whether it differs. */
  readonly setValue: (value: TabsValue, event?: Event | null) => boolean;

  /** Restore the default value, or the first enabled trigger when no default exists. */
  readonly reset: () => boolean;
}

/** Public instance exposed by TabsList. */
export interface TabsListExpose extends TabsListSlotState {
  /** Rendered tablist element. */
  readonly element: HTMLDivElement | null;

  /** Move focus to the selected, active, or first enabled trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by TabsTrigger. */
export interface TabsTriggerExpose extends TabsTriggerSlotState {
  /** Rendered native tab button. */
  readonly element: HTMLButtonElement | null;

  /** Deterministic id wired from trigger to content. */
  readonly id: string;

  /** Move focus to the trigger. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Public instance exposed by TabsContent. */
export interface TabsContentExpose extends TabsContentSlotState {
  /** Rendered tabpanel element. */
  readonly element: HTMLDivElement | null;

  /** Deterministic id wired from content to trigger. */
  readonly id: string;

  /** Move focus to the visible content panel. */
  readonly focusContent: (options?: FocusOptions) => void;
}

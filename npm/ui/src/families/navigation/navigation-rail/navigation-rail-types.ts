/** State exposed to the NavigationRail default slot. */
export interface NavigationRailSlotState<Value extends string = string> {
  /** Current destination, or `undefined` before one is chosen. */
  readonly active: Value | undefined;

  /** Whether destinations show their labels beside the icons. */
  readonly expanded: boolean;
}

/** Imperative surface exposed by NavigationRail. */
export interface NavigationRailExpose {
  /** Id of the navigation landmark. */
  readonly id: string;

  /** Whether destinations show their labels beside the icons. */
  readonly expanded: boolean;

  /** Request an expanded value. */
  readonly setExpanded: (value: boolean) => void;

  /** Flip the expanded value. */
  readonly toggle: () => void;
}

import type {
  MenuRootExpose,
  MenuSlotState,
  MenuTriggerExpose,
} from "../../menus/menu/menu-types.ts";

/**
 * Element the menu content is positioned against.
 *
 * - `group`: the whole split button, so the menu lines up with the primary action.
 * - `trigger`: the menu trigger only.
 */
export type SplitButtonAnchor = "group" | "trigger";

/** State exposed to SplitButton slots. */
export interface SplitButtonSlotState extends MenuSlotState {
  /** Whether the split button suppresses the primary action and refuses to open. */
  readonly disabled: boolean;
}

/** Imperative surface exposed by SplitButtonRoot. */
export interface SplitButtonRootExpose extends MenuRootExpose {
  /** Whether the split button suppresses the primary action and refuses to open. */
  readonly disabled: boolean;

  /** Rendered `role="group"` element. */
  readonly element: HTMLDivElement | null;
}

/** Imperative surface exposed by SplitButtonAction. */
export interface SplitButtonActionExpose {
  /** Rendered native button. */
  readonly element: HTMLButtonElement | null;

  /** Move focus to the primary action. */
  readonly focus: (options?: FocusOptions) => void;
}

/** Imperative surface exposed by SplitButtonTrigger. */
export type SplitButtonTriggerExpose = MenuTriggerExpose;

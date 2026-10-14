/**
 * Accessible, unstyled SplitButton: a primary action paired with a menu button
 * (WAI-ARIA APG) that opens related actions. Items, groups, radio/checkbox
 * items, and submenus are the shared Menu parts re-exported under SplitButton
 * names.
 */
export { default as SplitButton, default as SplitButtonRoot } from "./split-button-root.vue";
export { default as SplitButtonAction } from "./split-button-action.vue";
export { default as SplitButtonTrigger } from "./split-button-trigger.vue";
export {
  MenuArrow as SplitButtonArrow,
  MenuCheckboxItem as SplitButtonCheckboxItem,
  MenuContent as SplitButtonContent,
  MenuGroup as SplitButtonGroup,
  MenuItem as SplitButtonItem,
  MenuItemIndicator as SplitButtonItemIndicator,
  MenuLabel as SplitButtonLabel,
  MenuRadioGroup as SplitButtonRadioGroup,
  MenuRadioItem as SplitButtonRadioItem,
  MenuSeparator as SplitButtonSeparator,
  MenuSub as SplitButtonSub,
  MenuSubContent as SplitButtonSubContent,
  MenuSubTrigger as SplitButtonSubTrigger,
} from "../../menus/menu/menu.ts";
export type {
  MenuCheckedState as SplitButtonCheckedState,
  MenuContentExpose as SplitButtonContentExpose,
  MenuContentSlotState as SplitButtonContentSlotState,
  MenuDirection as SplitButtonDirection,
  MenuItemExpose as SplitButtonItemExpose,
  MenuSelectEvent as SplitButtonSelectEvent,
  MenuState as SplitButtonState,
} from "../../menus/menu/menu.ts";
export type {
  SplitButtonActionExpose,
  SplitButtonAnchor,
  SplitButtonRootExpose,
  SplitButtonSlotState,
  SplitButtonTriggerExpose,
} from "./split-button-types.ts";

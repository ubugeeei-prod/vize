/**
 * Mobile action sheet: a bottom Drawer (native modal `<dialog>`, drag to dismiss,
 * safe focus handling) whose actions follow the WAI-ARIA APG menu pattern.
 */
export { DrawerRoot as ActionSheet, DrawerRoot as ActionSheetRoot } from "../drawer/drawer.ts";
export { DrawerTrigger as ActionSheetTrigger } from "../drawer/drawer.ts";
export { DrawerContent as ActionSheetContent } from "../drawer/drawer.ts";
export { DrawerTitle as ActionSheetTitle } from "../drawer/drawer.ts";
export { DrawerDescription as ActionSheetDescription } from "../drawer/drawer.ts";
export { DrawerClose as ActionSheetCancel } from "../drawer/drawer.ts";
/** `role="menu"` list of actions with roving focus and typeahead. */
export { default as ActionSheetMenu } from "./action-sheet-menu.vue";
/** One `role="menuitem"` action; selecting it closes the sheet unless prevented. */
export { default as ActionSheetItem } from "./action-sheet-item.vue";
export type { ActionSheetSelectEvent } from "./action-sheet-types.ts";

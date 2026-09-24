/** Accessible, unstyled side drawer (sheet) built on native `<dialog>` and the Dialog contract. */
export { default as Drawer, default as DrawerRoot } from "./drawer-root.vue";
export { DialogClose as DrawerClose } from "../dialog/dialog.ts";
export { default as DrawerContent } from "./drawer-content.vue";
export { DialogDescription as DrawerDescription } from "../dialog/dialog.ts";
export { default as DrawerHandle } from "./drawer-handle.vue";
export { DialogTitle as DrawerTitle } from "../dialog/dialog.ts";
export { DialogTrigger as DrawerTrigger } from "../dialog/dialog.ts";
export {
  drawerSnapOffset,
  drawerSnapVisibleSize,
  isDrawerSnapPoint,
  resolveDrawerRelease,
  stepDrawerSnapPoint,
} from "./drawer-snap.ts";
export type { DrawerReleaseInput, DrawerReleaseResult } from "./drawer-snap.ts";
export type {
  DrawerAutoFocusEvent,
  DrawerBackdropPointerDownEvent,
  DrawerCloseExpose,
  DrawerContentExpose,
  DrawerDescriptionExpose,
  DrawerDismissEvent,
  DrawerDismissReason,
  DrawerDragEndEvent,
  DrawerDragOutcome,
  DrawerEscapeKeyDownEvent,
  DrawerHandleExpose,
  DrawerPointerDownOutsideEvent,
  DrawerRootExpose,
  DrawerSide,
  DrawerSlotState,
  DrawerSnapPoint,
  DrawerState,
  DrawerTitleExpose,
  DrawerTriggerExpose,
} from "./drawer-types.ts";

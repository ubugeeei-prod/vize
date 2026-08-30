/** Opinionated alert-dialog compound primitive built on Dialog. */
export {
  default as AlertDialog,
  default as AlertDialogRoot,
} from "./families/overlays/dialog/dialog-root.vue";
export { default as AlertDialogAction } from "./families/overlays/dialog/dialog-close.vue";
export { default as AlertDialogCancel } from "./families/overlays/dialog/dialog-close.vue";
export { default as AlertDialogContent } from "./alert-dialog-content.vue";
export { default as AlertDialogDescription } from "./families/overlays/dialog/dialog-description.vue";
export { default as AlertDialogOverlay } from "./families/overlays/dialog/dialog-overlay.vue";
export { default as AlertDialogPortal } from "./families/overlays/dialog/dialog-portal.vue";
export { default as AlertDialogTitle } from "./families/overlays/dialog/dialog-title.vue";
export { default as AlertDialogTrigger } from "./families/overlays/dialog/dialog-trigger.vue";
export type {
  AlertDialogActionExpose,
  AlertDialogAutoFocusEvent,
  AlertDialogCancelExpose,
  AlertDialogContentExpose,
  AlertDialogDescriptionExpose,
  AlertDialogOverlayExpose,
  AlertDialogPortalExpose,
  AlertDialogRootExpose,
  AlertDialogSlotState,
  AlertDialogState,
  AlertDialogTitleExpose,
  AlertDialogTriggerExpose,
} from "./alert-dialog-types.ts";

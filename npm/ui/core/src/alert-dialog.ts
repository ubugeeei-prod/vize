/** Opinionated alert-dialog compound primitive built on Dialog. */
export { default as AlertDialog, default as AlertDialogRoot } from "./dialog-root.vue";
export { default as AlertDialogAction } from "./dialog-close.vue";
export { default as AlertDialogCancel } from "./dialog-close.vue";
export { default as AlertDialogContent } from "./alert-dialog-content.vue";
export { default as AlertDialogDescription } from "./dialog-description.vue";
export { default as AlertDialogOverlay } from "./dialog-overlay.vue";
export { default as AlertDialogPortal } from "./dialog-portal.vue";
export { default as AlertDialogTitle } from "./dialog-title.vue";
export { default as AlertDialogTrigger } from "./dialog-trigger.vue";
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

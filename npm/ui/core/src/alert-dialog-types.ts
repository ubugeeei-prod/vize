import type {
  DialogAutoFocusEvent,
  DialogCloseExpose,
  DialogContentExpose,
  DialogDescriptionExpose,
  DialogOverlayExpose,
  DialogPortalExpose,
  DialogRootExpose,
  DialogSlotState,
  DialogState,
  DialogTitleExpose,
  DialogTriggerExpose,
} from "./families/overlays/dialog/dialog-types.ts";

/** Preventable AlertDialog auto-focus lifecycle event. */
export type AlertDialogAutoFocusEvent = DialogAutoFocusEvent;

/** Open state mirrored to the AlertDialog data contract. */
export type AlertDialogState = DialogState;

/** State exposed to compound AlertDialog slots. */
export type AlertDialogSlotState = DialogSlotState;

/** Public instance exposed by AlertDialogRoot. */
export type AlertDialogRootExpose = DialogRootExpose;

/** Public instance exposed by AlertDialogTrigger. */
export type AlertDialogTriggerExpose = DialogTriggerExpose;

/** Public instance exposed by AlertDialogPortal. */
export type AlertDialogPortalExpose = DialogPortalExpose;

/** Public instance exposed by AlertDialogOverlay. */
export type AlertDialogOverlayExpose = DialogOverlayExpose;

/** Public instance exposed by AlertDialogContent. */
export type AlertDialogContentExpose = DialogContentExpose;

/** Public instance exposed by AlertDialogTitle. */
export type AlertDialogTitleExpose = DialogTitleExpose;

/** Public instance exposed by AlertDialogDescription. */
export type AlertDialogDescriptionExpose = DialogDescriptionExpose;

/** Public instance exposed by AlertDialogAction. */
export type AlertDialogActionExpose = DialogCloseExpose;

/** Public instance exposed by AlertDialogCancel. */
export type AlertDialogCancelExpose = DialogCloseExpose;

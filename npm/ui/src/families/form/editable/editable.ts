/** Inline edit: a focusable preview that swaps to an input, submitting on Enter/blur and cancelling on Escape. */
export { default as Editable } from "./editable.vue";
/** Preview shown outside edit mode; activates edit mode by focus, click, or double click. */
export { default as EditablePreview } from "./editable-preview.vue";
/** Text input shown while editing. */
export { default as EditableInput } from "./editable-input.vue";
/** Button that edits, submits, or cancels, shown only when its action applies. */
export { default as EditableTrigger } from "./editable-trigger.vue";
export type {
  EditableActivationMode,
  EditableAriaInvalid,
  EditableExpose,
  EditableSlotState,
  EditableState,
  EditableSubmitMode,
  EditableTriggerAction,
} from "./editable-types.ts";

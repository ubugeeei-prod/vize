/** Accessible, unstyled TagsInput with typed tags, delimiters, paste splitting, and form association. */
export { default as TagsInput, default as TagsInputRoot } from "./tags-input-root.vue";
export { default as TagsInputInput } from "./tags-input-input.vue";
export { default as TagsInputItem } from "./tags-input-item.vue";
export { default as TagsInputItemDelete } from "./tags-input-item-delete.vue";
export { default as TagsInputItemText } from "./tags-input-item-text.vue";
export type {
  TagsInputAddSource,
  TagsInputAriaInvalid,
  TagsInputBy,
  TagsInputDirection,
  TagsInputInputExpose,
  TagsInputInvalidEvent,
  TagsInputInvalidReason,
  TagsInputItemExpose,
  TagsInputItemSlotState,
  TagsInputItemState,
  TagsInputRemoveSource,
  TagsInputRootExpose,
  TagsInputSlotState,
  TagsInputState,
  TagsInputValidator,
} from "./tags-input-types.ts";
export {
  containsTagDelimiter,
  evaluateTag,
  resolveTagEquality,
  splitTagText,
  splitTrailingTagText,
} from "./tags-input-value.ts";

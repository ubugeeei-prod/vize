/** Accessible, unstyled trigger-character mentions for textareas, inputs, and contenteditable. */
export { default as Mention, default as MentionRoot } from "./mention-root.vue";
export { default as MentionContent } from "./mention-content.vue";
export { default as MentionEditable } from "./mention-editable.vue";
export { default as MentionEmpty } from "./mention-empty.vue";
export { default as MentionInput } from "./mention-input.vue";
export { default as MentionItem } from "./mention-item.vue";
export {
  applyMentionEdit,
  containsMentionFilter,
  defaultMentionInsert,
  defaultMentionTriggers,
  detectMention,
  isSameMention,
  normalizeMentionText,
} from "./mention-core.ts";
export {
  locateTextOffset,
  measureEditableCaret,
  measureFieldCaret,
  measureTextFieldCaret,
  readFieldCaret,
  readFieldText,
  replaceEditableText,
} from "./mention-caret.ts";
export type { MentionFieldKind } from "./mention-caret.ts";
export type {
  MentionContentSlotState,
  MentionEdit,
  MentionFilter,
  MentionInsertTransform,
  MentionItemSlotState,
  MentionLoadContext,
  MentionLoader,
  MentionLoadStatus,
  MentionMatch,
  MentionRootExpose,
  MentionRootProps,
  MentionSlotState,
  MentionState,
  MentionTrigger,
} from "./mention-types.ts";

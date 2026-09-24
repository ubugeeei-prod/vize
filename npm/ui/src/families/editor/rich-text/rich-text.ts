/**
 * Headless rich-text editor: a typed document model (schema-declared node and
 * mark types), immutable transactions and commands with undo history, input
 * rules, sanitized HTML paste and serialization, and contenteditable parts.
 */
export { default as RichTextRoot, default as RichText } from "./rich-text-root.vue";
export { default as RichTextBubbleMenu } from "./rich-text-bubble-menu.vue";
export { default as RichTextContent } from "./rich-text-content.vue";
export { default as RichTextToolbar } from "./rich-text-toolbar.vue";
export { default as RichTextToolbarButton } from "./rich-text-toolbar-button.vue";
export * from "./rich-text-schema.ts";
export {
  assertRichTextDoc,
  emptyRichTextDoc,
  isRichTextDoc,
  richTextDocFromText,
  textContent as richTextTextContent,
  validateRichTextDoc,
} from "./rich-text-model.ts";
export type {
  RichTextBlock,
  RichTextDoc,
  RichTextInline,
  RichTextInlineLeaf,
  RichTextMark,
  RichTextPosition,
  RichTextSelection,
  RichTextText,
  RtBlock,
  RtDoc,
  RtInline,
  RtLeaf,
  RtMark,
  RtText,
} from "./rich-text-model.ts";
export {
  createRichTextState,
  createTransaction,
  redo,
  toRichTextState,
  toRtState,
  undo,
} from "./rich-text-state.ts";
export type {
  RichTextCommand,
  RichTextDispatch,
  RichTextHistory,
  RichTextHistoryEntry,
  RichTextState,
  RichTextStateOptions,
  RichTextTransaction,
  RichTextTransactionKind,
  RichTextTransactionOptions,
  RtState,
} from "./rich-text-state.ts";
export {
  activeMarks,
  createRichTextCommands,
  createUntypedRichTextCommands,
  isBlockActive,
  isMarkActive,
  isWrappedIn,
} from "./rich-text-commands.ts";
export type { RichTextCommands, RichTextUntypedCommands } from "./rich-text-commands.ts";
export {
  blockInputRule,
  defaultRichTextInputRules,
  insertTextWithRules,
  markInputRule,
} from "./rich-text-input-rules.ts";
export type { RichTextInputRule } from "./rich-text-input-rules.ts";
export { defaultRichTextKeymap, richTextKeyName } from "./rich-text-keymap.ts";
export type { RichTextKeymap } from "./rich-text-keymap.ts";
export { escapeHtml, richTextFromHtml, richTextToHtml } from "./rich-text-html.ts";
export type { RichTextHtmlOptions, RichTextParseOptions } from "./rich-text-html.ts";
export {
  domFromPosition,
  positionFromDom,
  readDomSelection,
  writeDomSelection,
} from "./rich-text-dom.ts";
export type { RichTextDomPoint } from "./rich-text-dom.ts";
export type {
  RichTextContentExpose,
  RichTextRootExpose,
  RichTextSlotProps,
  RichTextToolbarButtonSlotProps,
} from "./rich-text-types.ts";

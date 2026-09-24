<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, useTemplateRef, watch } from "vue";

import { richTextContext } from "./rich-text-context.ts";
import { readDomSelection, writeDomSelection } from "./rich-text-dom.ts";
import { richTextFromHtml, richTextToHtml } from "./rich-text-html.ts";
import { richTextKeyName } from "./rich-text-keymap.ts";
import {
  comparePositions,
  isCollapsed,
  richTextDocFromText,
  textContent,
} from "./rich-text-model.ts";
import type { RichTextSelection } from "./rich-text-model.ts";
import type { RichTextCommand } from "./rich-text-state.ts";
import type { RichTextContentExpose } from "./rich-text-types.ts";

const {
  ariaLabel = undefined,
  ariaLabelledby = undefined,
  ariaDescribedby = undefined,
  placeholder = undefined,
  spellcheck = true,
} = defineProps<{
  /**
   * Accessible name of the text box.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;

  /**
   * Ids that label the text box.
   *
   * @default undefined
   */
  readonly ariaLabelledby?: string;

  /**
   * Ids that describe the text box.
   *
   * @default undefined
   */
  readonly ariaDescribedby?: string;

  /**
   * Placeholder published as `data-placeholder` (and `aria-placeholder`) while empty.
   *
   * @default undefined
   */
  readonly placeholder?: string;

  /**
   * Native spellchecking.
   *
   * @default true
   */
  readonly spellcheck?: boolean;
}>();

const editor = richTextContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
// The markup is produced by richTextToHtml: every text is escaped and every
// attribute passes the schema's render specs, so v-html never sees raw input.
const html = computed(() =>
  richTextToHtml(editor.state.value.schema, editor.state.value.doc, { editor: true }),
);
const empty = computed(() => {
  const blocks = editor.state.value.doc.content;
  const first = blocks[0];
  return (
    blocks.length === 1 &&
    first !== undefined &&
    first.content.every((node) => "text" in node && node.text === "")
  );
});
let composing = false;
let compositionSelection: RichTextSelection | null = null;
let restoring = false;

function syncFromDom(): void {
  if (!element.value || composing || restoring) return;
  const selection = readDomSelection(element.value);
  if (!selection) return;
  const current = editor.state.value.selection;
  if (
    comparePositions(selection.anchor, current.anchor) !== 0 ||
    comparePositions(selection.head, current.head) !== 0
  ) {
    editor.select(selection);
  }
}

function restoreSelection(): void {
  if (!element.value || composing || element.value.ownerDocument.activeElement !== element.value) {
    return;
  }
  restoring = true;
  try {
    writeDomSelection(element.value, editor.state.value.selection);
  } finally {
    restoring = false;
  }
}

watch(
  () => editor.state.value,
  () => void nextTick(restoreSelection),
  { flush: "post" },
);

function run(command: RichTextCommand): void {
  editor.run(command);
}

function onBeforeinput(event: InputEvent): void {
  if (!editor.editable.value) {
    event.preventDefault();
    return;
  }
  // Composition text is owned by the IME until compositionend.
  if (event.inputType === "insertCompositionText" || composing) return;
  event.preventDefault();
  syncFromDom();
  const commands = editor.commands;
  switch (event.inputType) {
    case "insertText":
    case "insertReplacementText":
      if (event.data) editor.typeText(event.data);
      break;
    case "insertParagraph":
      run(commands.splitBlock);
      break;
    case "insertLineBreak":
      if (!editor.run(commands.insertInline("hardBreak"))) run(commands.splitBlock);
      break;
    case "deleteContentBackward":
      run(commands.deleteBackward("character"));
      break;
    case "deleteWordBackward":
      run(commands.deleteBackward("word"));
      break;
    case "deleteContentForward":
      run(commands.deleteForward("character"));
      break;
    case "deleteWordForward":
      run(commands.deleteForward("word"));
      break;
    case "deleteByCut":
    case "deleteContent":
      run(commands.deleteSelection);
      break;
    case "formatBold":
      run(commands.toggleMark("bold"));
      break;
    case "formatItalic":
      run(commands.toggleMark("italic"));
      break;
    case "formatUnderline":
      run(commands.toggleMark("underline"));
      break;
    case "historyUndo":
      run(commands.undo);
      break;
    case "historyRedo":
      run(commands.redo);
      break;
    default:
      // Unmodeled mutations (drag-drop, spelling fixes without data) are refused.
      break;
  }
}

function onKeydown(event: KeyboardEvent): void {
  if (event.isComposing || composing || event.defaultPrevented) return;
  const name = richTextKeyName(event);
  if (name === "Alt-F10") {
    const [first] = editor.toolbars.value;
    if (first?.()) event.preventDefault();
    return;
  }
  if (!editor.editable.value) return;
  syncFromDom();
  if (editor.runKey(name)) event.preventDefault();
}

function onPaste(event: ClipboardEvent): void {
  event.preventDefault();
  if (!editor.editable.value) return;
  const data = event.clipboardData;
  if (!data) return;
  syncFromDom();
  const schema = editor.state.value.schema;
  const htmlData = data.getData("text/html");
  const textData = data.getData("text/plain");
  const fragment =
    htmlData && element.value
      ? richTextFromHtml(schema, htmlData, { document: element.value.ownerDocument })
      : textData
        ? richTextDocFromText(schema, textData)
        : null;
  if (fragment) run(editor.commands.insertContent(fragment));
}

function onCut(event: ClipboardEvent): void {
  const selection = element.value?.ownerDocument.getSelection();
  if (!element.value || !selection || selection.rangeCount === 0 || !event.clipboardData) return;
  event.preventDefault();
  const container = element.value.ownerDocument.createElement("div");
  container.append(selection.getRangeAt(0).cloneContents());
  event.clipboardData.setData("text/html", container.innerHTML);
  event.clipboardData.setData("text/plain", selection.toString());
  if (editor.editable.value && !isCollapsed(editor.state.value.selection)) {
    syncFromDom();
    run(editor.commands.deleteSelection);
  }
}

function onCompositionstart(): void {
  syncFromDom();
  composing = true;
  compositionSelection = editor.state.value.selection;
}

function onCompositionend(event: CompositionEvent): void {
  composing = false;
  const selection = compositionSelection;
  compositionSelection = null;
  if (selection) editor.select(selection);
  const inserted = event.data ? editor.run(editor.commands.insertText(event.data)) : false;
  // The IME mutated the DOM directly; when the model did not change, re-render it.
  void nextTick(() => {
    if (!inserted && element.value) element.value.innerHTML = html.value;
    restoreSelection();
  });
}

function onSelectionchange(): void {
  if (element.value && element.value.ownerDocument.activeElement === element.value) syncFromDom();
}

function onDragstart(event: DragEvent): void {
  event.preventDefault();
}

onMounted(() => {
  editor.contentElement.value = element.value;
  element.value?.ownerDocument.addEventListener("selectionchange", onSelectionchange);
});

onUnmounted(() => {
  element.value?.ownerDocument.removeEventListener("selectionchange", onSelectionchange);
  if (editor.contentElement.value === element.value) editor.contentElement.value = null;
});

const contentProps = computed(() => ({
  role: "textbox",
  contenteditable: editor.editable.value,
  tabindex: 0,
  "aria-multiline": true,
  "aria-readonly": editor.editable.value ? undefined : true,
  "aria-placeholder": empty.value ? placeholder : undefined,
  onBeforeinput,
  onBlur: () => {
    editor.focused.value = false;
  },
  onCompositionend,
  onCompositionstart,
  onCut,
  onDragstart,
  onDrop: onDragstart,
  onFocus: () => {
    editor.focused.value = true;
    syncFromDom();
  },
  onKeydown,
  onPaste,
}));

function focus(): void {
  element.value?.focus({ preventScroll: true });
}

defineExpose({ element, focus } satisfies Omit<RichTextContentExpose, "element"> & {
  readonly element: typeof element;
});
</script>

<template>
  <!-- eslint-disable-next-line vue/no-v-html -- markup comes from richTextToHtml: text is escaped and attributes pass the schema's sanitizing render specs -->
  <div
    :id="editor.contentId.value"
    ref="element"
    v-bind="contentProps"
    :aria-label="ariaLabel"
    :aria-labelledby="ariaLabelledby"
    :aria-describedby="ariaDescribedby"
    :spellcheck="spellcheck ? 'true' : 'false'"
    data-vize-ui="rich-text-content"
    part="content"
    :data-empty="empty ? 'true' : undefined"
    :data-placeholder="placeholder"
    v-html="html"
  ></div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

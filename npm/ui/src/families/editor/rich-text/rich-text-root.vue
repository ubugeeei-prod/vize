<script setup lang="ts" generic="Schema extends RichTextSchema">
import { computed, nextTick, shallowRef, watch } from "vue";

import {
  useDeterministicId,
  deriveDeterministicId,
} from "../../foundations/id/deterministic-id.ts";
import { createUntypedRichTextCommands } from "./rich-text-commands.ts";
import { richTextContext } from "./rich-text-context.ts";
import { richTextToHtml } from "./rich-text-html.ts";
import { defaultRichTextInputRules, insertTextWithRules } from "./rich-text-input-rules.ts";
import type { RichTextInputRule } from "./rich-text-input-rules.ts";
import { defaultRichTextKeymap } from "./rich-text-keymap.ts";
import type { RichTextKeymap } from "./rich-text-keymap.ts";
import type { RichTextDoc, RichTextSelection } from "./rich-text-model.ts";
import type { RichTextSchema } from "./rich-text-schema.ts";
import {
  createRichTextState,
  createTransaction,
  toRichTextState,
  toRtState,
} from "./rich-text-state.ts";
import type { RichTextCommand, RichTextState, RichTextTransaction } from "./rich-text-state.ts";
import type { RichTextRootExpose, RichTextSlotProps } from "./rich-text-types.ts";

const {
  schema,
  modelValue = undefined,
  defaultValue = undefined,
  id = undefined,
  editable = true,
  inputRules = undefined,
  keymap = undefined,
  historyDepth = 100,
} = defineProps<{
  /**
   * Schema declaring node and mark types; documents and commands are typed from it.
   *
   * @default undefined
   */
  readonly schema: Schema;

  /**
   * Controlled document (`v-model`). `undefined` selects uncontrolled behavior.
   *
   * @default undefined
   */
  readonly modelValue?: RichTextDoc<Schema>;

  /**
   * Initial uncontrolled document.
   *
   * @default one empty default block
   */
  readonly defaultValue?: RichTextDoc<Schema>;

  /**
   * Consumer-owned editor id. `null` and `undefined` select a deterministic fallback.
   *
   * @default undefined
   */
  readonly id?: string | null;

  /**
   * Allow editing; `false` renders read-only content.
   *
   * @default true
   */
  readonly editable?: boolean;

  /**
   * Input rules applied to typed text.
   *
   * @default defaultRichTextInputRules(schema)
   */
  readonly inputRules?: readonly RichTextInputRule[];

  /**
   * Keyboard shortcuts.
   *
   * @default defaultRichTextKeymap(schema)
   */
  readonly keymap?: RichTextKeymap;

  /**
   * Maximum undo depth.
   *
   * @default 100
   */
  readonly historyDepth?: number;
}>();

const emit = defineEmits<{
  /** Fired with the new document after every change (supports `v-model`). */
  "update:modelValue": [value: RichTextDoc<Schema>];
  /** Fired for every transaction, including selection-only changes. */
  transaction: [transaction: RichTextTransaction];
}>();

defineSlots<{
  /** Editor parts (content, toolbars, bubble menus). Receives the live state and commands. */
  default?(props: RichTextSlotProps<Schema>): unknown;
}>();

const baseId = useDeterministicId({ id: () => id, hint: "rich-text" });
const contentId = computed(() => deriveDeterministicId(baseId.value, "content"));
const state = shallowRef<RichTextState<Schema>>(
  createRichTextState(schema, { ...initialDoc(), historyDepth }),
);
const focused = shallowRef(false);
const contentElement = shallowRef<HTMLElement | null>(null);
const toolbars = shallowRef<readonly (() => boolean)[]>([]);
const editableState = computed(() => editable);

function initialDoc(): { readonly doc?: RichTextDoc<Schema> } {
  const doc = modelValue ?? defaultValue;
  return doc === undefined ? {} : { doc };
}

const defaultRules = computed(() => defaultRichTextInputRules(schema));
const defaultKeymap = computed(() => defaultRichTextKeymap(schema));

// Schema-agnostic view for commands and parts. The erased document is the same
// object, re-checked by the schema's cached validation.
const erased = computed(() => toRtState(state.value));

function dispatch(transaction: RichTextTransaction): void {
  state.value = toRichTextState(schema, transaction.state);
  emit("transaction", transaction);
  if (transaction.docChanged) emit("update:modelValue", state.value.doc);
}

function run(command: RichTextCommand): boolean {
  if (!editable) return false;
  return command(erased.value, dispatch);
}

watch(
  () => modelValue,
  (next) => {
    if (next === undefined || next === state.value.doc) return;
    state.value = createRichTextState(schema, {
      doc: next,
      selection: state.value.selection,
      historyDepth,
    });
  },
);

function focus(): void {
  void nextTick(() => contentElement.value?.focus({ preventScroll: true }));
}

richTextContext.provide({
  id: baseId,
  contentId,
  state: erased,
  editable: editableState,
  focused,
  contentElement,
  commands: createUntypedRichTextCommands(schema),
  typeText: (text) => run(insertTextWithRules(text, inputRules ?? defaultRules.value)),
  runKey: (name) => {
    const command = (keymap ?? defaultKeymap.value)[name];
    return command !== undefined && run(command);
  },
  run,
  can: (command) => editable && command(erased.value),
  select: (selection: RichTextSelection) => {
    dispatch(createTransaction(erased.value, { selection }, "selection"));
  },
  focus,
  toolbars,
});

const slotProps = computed<RichTextSlotProps<Schema>>(() => ({
  state: state.value,
  editable,
  focused: focused.value,
  run,
}));

defineExpose({
  state,
  run,
  focus,
  toHtml: () => richTextToHtml(schema, state.value.doc),
} satisfies Omit<RichTextRootExpose<Schema>, "state"> & { readonly state: typeof state });
</script>

<template>
  <div
    :id="baseId"
    data-vize-ui="rich-text"
    part="root"
    :data-editable="editable ? 'true' : 'false'"
    :data-focused="focused ? 'true' : undefined"
  >
    <slot v-bind="slotProps" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

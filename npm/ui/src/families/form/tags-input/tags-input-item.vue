<script setup lang="ts" generic="T">
import { computed, nextTick, useTemplateRef, watch } from "vue";

import { tagsInputContext, tagsInputItemContext } from "./tags-input-context.ts";
import type {
  TagsInputItemExpose,
  TagsInputItemSlotState,
  TagsInputItemState,
  TagsInputRemoveSource,
} from "./tags-input-types.ts";

const {
  value,
  index,
  disabled = false,
  editLabel = "Edit tag",
} = defineProps<{
  /** Tag value rendered by this item, normally one entry of the root slot `tags`. @default required */
  readonly value: T;

  /** Zero-based position of `value` in the root tag list. @default required */
  readonly index: number;

  /**
   * Keep this tag visible but unfocusable, uneditable, and unremovable.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name prefix of the inline edit input.
   *
   * @default "Edit tag"
   */
  readonly editLabel?: string;
}>();

defineSlots<{
  /** Tag content, normally TagsInputItemText and TagsInputItemDelete. Hidden while editing. */
  default?(props: TagsInputItemSlotState<T>): unknown;
}>();

const context = tagsInputContext.use();
const groupRole = "group" as const;
const element = useTemplateRef<HTMLSpanElement>("element");
const editInput = useTemplateRef<HTMLInputElement>("editInput");
const itemDisabled = computed(() => disabled || context.disabled.value);
const editing = computed(() => context.editingIndex.value === index);
const active = computed(() => context.activeIndex.value === index);
const text = computed(() => context.tagText(value));
const itemState = computed<TagsInputItemState>(() => {
  if (itemDisabled.value) return "disabled";
  return editing.value ? "editing" : "idle";
});
const slotState = computed<TagsInputItemSlotState<T>>(() => ({
  active: active.value,
  disabled: itemDisabled.value,
  editing: editing.value,
  index,
  state: itemState.value,
  text: text.value,
  value,
}));
const itemId = computed(() => context.itemId(index));
const itemInteractiveProps = computed<{
  readonly role: "group";
  readonly onKeydown: (event: KeyboardEvent) => void;
  readonly onFocus: () => void;
  readonly onBlur: () => void;
  readonly onDblclick: () => void;
}>(() => ({
  role: groupRole,
  onBlur,
  onDblclick,
  onFocus,
  onKeydown,
}));
let restoreFocusAfterEdit = false;

watch(
  editing,
  (now, before) => {
    if (now) {
      void nextTick(() => {
        editInput.value?.focus();
        editInput.value?.select();
      });
    } else if (before && restoreFocusAfterEdit) {
      restoreFocusAfterEdit = false;
      void nextTick(() => element.value?.focus());
    }
  },
  { flush: "post" },
);

function removeSelf(source: TagsInputRemoveSource): boolean {
  if (itemDisabled.value) return false;
  const count = context.tags.value.length;
  const removed = context.removeAt(index, source);
  if (!removed) return false;
  // Deletion keeps focus inside the field: Backspace walks left, Delete and
  // the delete button keep the same slot, and an empty list returns to input.
  const remaining = count - 1;
  let target: number | null;
  if (remaining === 0) target = null;
  else if (source === "backspace") target = Math.max(0, index - 1);
  else target = index < remaining ? index : null;
  void nextTick(() => {
    if (target === null || !context.focusItem(target)) context.focusInput();
  });
  return true;
}

function move(delta: -1 | 1): void {
  const target = index + delta;
  if (target < 0) return;
  if (target >= context.tags.value.length) context.focusInput();
  else context.focusItem(target);
}

function onKeydown(event: KeyboardEvent): void {
  if (event.target !== event.currentTarget || event.defaultPrevented) return;
  if (itemDisabled.value || event.isComposing) return;
  const rtl = context.direction.value === "rtl";
  switch (event.key) {
    case "ArrowLeft":
      move(rtl ? 1 : -1);
      break;
    case "ArrowRight":
      move(rtl ? -1 : 1);
      break;
    case "Home":
      context.focusItem(0);
      break;
    case "End":
      context.focusInput();
      break;
    case "Backspace":
      removeSelf("backspace");
      break;
    case "Delete":
      removeSelf("delete");
      break;
    case "Enter":
    case "F2":
      if (!edit()) return;
      break;
    default:
      return;
  }
  event.preventDefault();
}

function onFocus(): void {
  context.activeIndex.value = index;
}

function onBlur(): void {
  if (context.activeIndex.value === index) context.activeIndex.value = null;
}

function onDblclick(): void {
  if (!itemDisabled.value) edit();
}

function onEditKeydown(event: KeyboardEvent): void {
  if (event.isComposing) return;
  const input = event.currentTarget as HTMLInputElement;
  if (event.key === "Enter") {
    event.preventDefault();
    restoreFocusAfterEdit = true;
    context.commitEdit(index, input.value, true);
    if (editing.value) restoreFocusAfterEdit = false;
  } else if (event.key === "Escape") {
    event.preventDefault();
    event.stopPropagation();
    restoreFocusAfterEdit = true;
    context.cancelEdit(index);
  }
}

function onEditBlur(event: FocusEvent): void {
  if (!editing.value) return;
  context.commitEdit(index, (event.currentTarget as HTMLInputElement).value, false);
}

function edit(): boolean {
  if (itemDisabled.value) return false;
  return context.startEdit(index);
}

function focus(options?: FocusOptions): void {
  if (!itemDisabled.value) element.value?.focus(options);
}

tagsInputItemContext.provide({
  remove: removeSelf,
  slotState,
});

type TagsInputItemSetupExpose = Omit<TagsInputItemExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = {
  edit,
  element,
  focus,
  remove: () => removeSelf("api"),
} satisfies TagsInputItemSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    v-bind="itemInteractiveProps"
    :id="itemId"
    ref="element"
    aria-roledescription="tag"
    :aria-label="text"
    :aria-disabled="itemDisabled ? 'true' : undefined"
    :tabindex="itemDisabled || editing ? undefined : -1"
    data-vize-ui="tags-input-item"
    part="item"
    :data-state="itemState"
    :data-index="index"
    :data-active="active ? 'true' : undefined"
    :data-disabled="itemDisabled ? 'true' : undefined"
  >
    <input
      v-if="editing"
      ref="editInput"
      type="text"
      :value="text"
      :aria-label="`${editLabel} ${text}`"
      autocomplete="off"
      data-vize-ui="tags-input-item-edit"
      part="item-edit"
      @keydown="onEditKeydown"
      @blur="onEditBlur"
    />
    <slot v-else v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

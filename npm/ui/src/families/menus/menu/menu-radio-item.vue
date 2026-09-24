<script setup lang="ts" generic="Value">
import { computed, useTemplateRef } from "vue";

import { menuItemIndicatorContext, menuRadioGroupContext } from "./menu-context.ts";
import { createMenuSelectEvent, useMenuItem } from "./menu-item-runtime.ts";
import type {
  MenuItemCheckedState,
  MenuRadioItemExpose,
  MenuRadioItemSlotState,
  MenuSelectEvent,
} from "./menu-types.ts";

const {
  value,
  disabled = false,
  textValue = undefined,
  closeOnSelect = true,
} = defineProps<{
  /**
   * Value committed to the enclosing MenuRadioGroup when this item is selected.
   *
   * @default undefined
   */
  readonly value: Value;

  /**
   * Block activation while keeping the item focusable.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Typeahead text when the rendered text is not representative.
   *
   * @default undefined
   */
  readonly textValue?: string;

  /**
   * Close the whole menu tree after an unprevented selection.
   *
   * @default true
   */
  readonly closeOnSelect?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the item is activated, before the group value changes; preventable. */
  select: [event: MenuSelectEvent];
}>();

defineSlots<{
  /** Item contents. Receives checked, highlight, and disabled state. */
  default?(props: MenuRadioItemSlotState): unknown;
}>();

const group = menuRadioGroupContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const checked = computed(() => group.isChecked(value));
const state = computed<MenuItemCheckedState>(() => (checked.value ? "checked" : "unchecked"));

function activate(event: Event | null): boolean {
  if (item.disabled.value) return false;
  const selectEvent = createMenuSelectEvent(element.value, event);
  emit("select", selectEvent);
  group.select(value, event);
  if (!selectEvent.defaultPrevented && closeOnSelect) item.tree.closeAll(event);
  return true;
}

const item = useMenuItem({
  role: "menuitemradio",
  element,
  hint: "menu-radio-item",
  disabled: () => disabled || group.disabled.value,
  textValue: () => textValue,
  activate,
});

menuItemIndicatorContext.provide({ state });

defineExpose({
  checked,
  disabled: item.disabled,
  element,
  focus: item.focus,
  highlighted: item.highlighted,
  id: item.id,
  select: (event: Event | null = null) => activate(event),
} satisfies Omit<MenuRadioItemExpose, "checked" | "disabled" | "element" | "highlighted" | "id"> & {
  readonly checked: typeof checked;
  readonly disabled: typeof item.disabled;
  readonly element: typeof element;
  readonly highlighted: typeof item.highlighted;
  readonly id: typeof item.id;
});
</script>

<template>
  <div
    v-bind="item.interactiveProps"
    :id="item.id.value"
    ref="element"
    :aria-checked="checked ? 'true' : 'false'"
    :aria-disabled="item.disabled.value ? 'true' : undefined"
    data-vize-ui="menu-radio-item"
    part="item"
    :data-state="state"
    :data-disabled="item.disabled.value ? 'true' : undefined"
    :data-highlighted="item.highlighted.value ? 'true' : undefined"
  >
    <slot
      :checked="checked"
      :disabled="item.disabled.value"
      :highlighted="item.highlighted.value"
      :state="state"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

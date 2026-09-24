<script setup lang="ts">
import { useTemplateRef } from "vue";

import { createMenuSelectEvent, useMenuItem } from "./menu-item-runtime.ts";
import type { MenuItemExpose, MenuItemSlotState, MenuSelectEvent } from "./menu-types.ts";

const {
  disabled = false,
  textValue = undefined,
  closeOnSelect = true,
} = defineProps<{
  /**
   * Block activation. The item stays focusable and announced as disabled (WAI-ARIA APG).
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
  /** Fired when the item is activated; call `preventDefault()` to keep the menu open. */
  select: [event: MenuSelectEvent];
}>();

defineSlots<{
  /** Item contents. Receives highlight and disabled state. */
  default(props: MenuItemSlotState): unknown;
}>();

const element = useTemplateRef<HTMLDivElement>("element");

function activate(event: Event | null): boolean {
  if (item.disabled.value) return false;
  const selectEvent = createMenuSelectEvent(element.value, event);
  emit("select", selectEvent);
  if (!selectEvent.defaultPrevented && closeOnSelect) item.tree.closeAll(event);
  return true;
}

const item = useMenuItem({
  role: "menuitem",
  element,
  hint: "menu-item",
  disabled: () => disabled,
  textValue: () => textValue,
  activate,
});

defineExpose({
  disabled: item.disabled,
  element,
  focus: item.focus,
  highlighted: item.highlighted,
  id: item.id,
  select: (event: Event | null = null) => activate(event),
} satisfies Omit<MenuItemExpose, "disabled" | "element" | "highlighted" | "id"> & {
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
    :aria-disabled="item.disabled.value ? 'true' : undefined"
    data-vize-ui="menu-item"
    part="item"
    :data-disabled="item.disabled.value ? 'true' : undefined"
    :data-highlighted="item.highlighted.value ? 'true' : undefined"
  >
    <slot :disabled="item.disabled.value" :highlighted="item.highlighted.value" />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

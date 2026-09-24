<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from "vue";

import { menuLevelContext } from "./menu-context.ts";
import { forwardKey, pointerPoint } from "./menu-dom.ts";
import { useMenuItem } from "./menu-item-runtime.ts";
import type { MenuActivationSource } from "./menu-item-runtime.ts";
import type { MenuItemExpose, MenuSubTriggerSlotState } from "./menu-types.ts";

const { disabled = false, textValue = undefined } = defineProps<{
  /**
   * Block opening the submenu while keeping the trigger focusable.
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
}>();

defineSlots<{
  /** Trigger contents. Receives submenu open state plus highlight and disabled state. */
  default(props: MenuSubTriggerSlotState): unknown;
}>();

const level = menuLevelContext.use();
const element = useTemplateRef<HTMLDivElement>("element");

function openSubmenu(event: Event | null, source: MenuActivationSource): boolean {
  if (item.disabled.value) return false;
  const entry = source === "pointer" ? "none" : "first";
  if (level.open.value) {
    if (entry === "first") level.focusContentEdge.value?.("first");
    return true;
  }
  return level.setOpen(true, event, entry);
}

const item = useMenuItem({
  role: "menuitem",
  element,
  hint: "menu-sub-trigger",
  disabled: () => disabled || level.disabled.value,
  textValue: () => textValue,
  subLevel: level,
  activate: (event, source) =>
    openSubmenu(event, event instanceof MouseEvent && event.detail === 0 ? "keyboard" : source),
  keydown: (event) => {
    if (event.key !== forwardKey(item.tree.dir.value)) return;
    event.preventDefault();
    openSubmenu(event, "keyboard");
  },
  pointermove: (event) => {
    if (!level.open.value) openSubmenu(event, "pointer");
  },
  pointerleave: (event) => {
    const content = level.contentElement.value;
    if (!level.open.value || !content) return false;
    item.content.startGrace(pointerPoint(event), content);
    return true;
  },
});
level.triggerItemKey.value = item.id.value;

onMounted(() => {
  level.triggerElement.value = element.value;
});
onUnmounted(() => {
  if (level.triggerElement.value === element.value) level.triggerElement.value = null;
});

defineExpose({
  disabled: item.disabled,
  element,
  focus: item.focus,
  highlighted: item.highlighted,
  id: item.id,
  select: (event: Event | null = null) => openSubmenu(event, "imperative"),
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
    :id="level.triggerId.value"
    ref="element"
    aria-haspopup="menu"
    :aria-expanded="level.open.value ? 'true' : 'false'"
    :aria-controls="level.open.value ? level.contentId.value : undefined"
    :aria-disabled="item.disabled.value ? 'true' : undefined"
    data-vize-ui="menu-sub-trigger"
    part="sub-trigger"
    :data-state="level.state.value"
    :data-disabled="item.disabled.value ? 'true' : undefined"
    :data-highlighted="item.highlighted.value ? 'true' : undefined"
  >
    <slot
      :disabled="item.disabled.value"
      :highlighted="item.highlighted.value"
      :open="level.open.value"
      :state="level.state.value"
    />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

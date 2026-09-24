<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { menuLevelContext, menuTreeContext } from "./menu-context.ts";
import type { MenuSlotState, MenuTriggerExpose } from "./menu-types.ts";

const { disabled = false, ariaLabel = undefined } = defineProps<{
  /**
   * Remove the trigger from activation and sequential keyboard focus.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when no visible label supplies one.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before a click toggles the menu. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
  /** Fired before keyboard handling. Call `preventDefault()` to skip the built-in keys. */
  keydown: [nativeEvent: KeyboardEvent];
}>();

defineSlots<{
  /** Trigger contents. Receives the menu open state. */
  default(props: MenuSlotState): unknown;
}>();

const tree = menuTreeContext.use();
const level = menuLevelContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
level.hasTrigger.value = true;
const disabledState = computed(() => disabled);

onMounted(() => {
  level.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (level.triggerElement.value === element.value) level.triggerElement.value = null;
});

function onClick(event: MouseEvent): void {
  if (disabledState.value) return;
  emit("click", event);
  if (event.defaultPrevented) return;
  // `detail === 0` is a keyboard or assistive-technology activation.
  level.setOpen(!level.open.value, event, event.detail === 0 ? "first" : "content");
}

function onKeydown(event: KeyboardEvent): void {
  if (disabledState.value) return;
  emit("keydown", event);
  if (event.defaultPrevented) return;
  if (event.key === "Enter" || event.key === " " || event.key === "ArrowDown") {
    event.preventDefault();
    level.setOpen(true, event, "first");
  } else if (event.key === "ArrowUp") {
    event.preventDefault();
    level.setOpen(true, event, "last");
  }
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

defineExpose({ element, focus } satisfies Omit<MenuTriggerExpose, "element"> & {
  readonly element: typeof element;
});
</script>

<template>
  <button
    :id="level.triggerId.value"
    ref="element"
    type="button"
    :disabled="disabledState"
    :aria-label="ariaLabel"
    aria-haspopup="menu"
    :aria-expanded="level.open.value ? 'true' : 'false'"
    :aria-controls="level.open.value ? level.contentId.value : undefined"
    data-vize-ui="menu-trigger"
    part="trigger"
    :data-state="level.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot
      :dir="tree.dir.value"
      :modal="tree.modal.value"
      :open="level.open.value"
      :state="level.state.value"
    />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

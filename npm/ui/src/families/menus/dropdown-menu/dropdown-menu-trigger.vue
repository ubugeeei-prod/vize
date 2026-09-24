<script setup lang="ts">
import { computed, onMounted, onUnmounted, useTemplateRef } from "vue";

import { menuLevelContext, menuTreeContext } from "../menu/menu-context.ts";
import type { MenuSlotState, MenuTriggerExpose } from "../menu/menu-types.ts";

const {
  disabled = false,
  ariaLabel = undefined,
  openOn = "pointerdown",
} = defineProps<{
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

  /**
   * Primary mouse button event that toggles the menu. `pointerdown` opens on
   * press (native menu feel, press-drag-release selection); `click` waits for release.
   *
   * @default "pointerdown"
   */
  readonly openOn?: "click" | "pointerdown";
}>();

const emit = defineEmits<{
  /** Fired before a primary-button pointer-down toggles the menu; preventable. */
  pointerdown: [nativeEvent: PointerEvent];
  /** Fired before keyboard handling; call `preventDefault()` to skip the built-in keys. */
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

let touchPress = false;

function onPointerdown(event: PointerEvent): void {
  if (disabledState.value || openOn !== "pointerdown") return;
  if (event.button !== 0 || event.ctrlKey || event.pointerType === "touch") return;
  emit("pointerdown", event);
  if (event.defaultPrevented) return;
  const opening = !level.open.value;
  level.setOpen(opening, event, "content");
  // Keep the trigger from taking focus so the opening content owns it.
  if (opening) event.preventDefault();
}

function onClick(event: MouseEvent): void {
  if (disabledState.value) return;
  // Mouse presses already toggled on pointer-down; touch, pen taps and
  // keyboard/assistive-technology clicks (`detail === 0`) toggle here.
  const handledByPointerdown = openOn === "pointerdown" && event.detail > 0 && !touchPress;
  touchPress = false;
  if (handledByPointerdown) return;
  level.setOpen(!level.open.value, event, event.detail === 0 ? "first" : "content");
}

function onPointerup(event: PointerEvent): void {
  touchPress = event.pointerType === "touch";
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
    data-vize-ui="dropdown-menu-trigger"
    part="trigger"
    :data-state="level.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
    @pointerdown="onPointerdown"
    @pointerup="onPointerup"
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

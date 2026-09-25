<script setup lang="ts">
import { onMounted, onUnmounted, useTemplateRef } from "vue";

import type { SpeedDialSlotState, SpeedDialTriggerExpose } from "./floating-action-button-types.ts";
import { speedDialContext } from "./speed-dial-context.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name, for example "Create". Required in practice for icon-only triggers.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Trigger icon and label. Receives the speed-dial state. */
  default(props: SpeedDialSlotState): unknown;
}>();

const context = speedDialContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const openKeys: Readonly<Record<string, string>> = {
  down: "ArrowDown",
  left: "ArrowLeft",
  right: "ArrowRight",
  up: "ArrowUp",
};

onMounted(() => {
  context.triggerElement.value = element.value;
});

onUnmounted(() => {
  if (context.triggerElement.value === element.value) context.triggerElement.value = null;
});

let pointerActivation = false;

function onPointerdown(): void {
  pointerActivation = true;
}

function onClick(event: MouseEvent): void {
  const fromPointer = pointerActivation;
  pointerActivation = false;
  if (context.disabled.value) return;
  // Keyboard activation (Enter/Space) moves focus into the menu per the APG menu button pattern.
  if (!context.open.value && !fromPointer) context.openAndFocus(event);
  else context.setOpen(!context.open.value, event);
}

function onKeydown(event: KeyboardEvent): void {
  if (context.disabled.value || event.defaultPrevented) return;
  if (event.key === openKeys[context.direction.value]) {
    event.preventDefault();
    context.openAndFocus(event);
  } else if (event.key === "Escape" && context.open.value) {
    event.preventDefault();
    context.close({ focusTrigger: true }, event);
  }
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type SpeedDialTriggerSetupExpose = Omit<SpeedDialTriggerExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies SpeedDialTriggerSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    :id="context.triggerId.value"
    ref="element"
    type="button"
    :disabled="context.disabled.value"
    :aria-label="ariaLabel"
    aria-haspopup="menu"
    :aria-expanded="context.open.value ? 'true' : 'false'"
    :aria-controls="context.contentId.value"
    data-vize-ui="speed-dial-trigger"
    part="trigger"
    :data-state="context.state.value"
    @pointerdown="onPointerdown"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot
      :direction="context.direction.value"
      :disabled="context.disabled.value"
      :open="context.open.value"
      :state="context.state.value"
    />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { menuLevelContext, menuTreeContext } from "../../menus/menu/menu-context.ts";
import { splitButtonContext } from "./split-button-context.ts";
import type { SplitButtonActionExpose, SplitButtonSlotState } from "./split-button-types.ts";

const {
  type = "button",
  disabled = false,
  ariaLabel = undefined,
} = defineProps<{
  /**
   * Native button submission behavior.
   *
   * @default "button"
   */
  readonly type?: "button" | "reset" | "submit";

  /**
   * Disable the primary action while the trigger stays available.
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
  /** Fired after user activation reaches the enabled primary action. */
  press: [nativeEvent: MouseEvent];
  /** Fired before keyboard handling; call `preventDefault()` to skip the built-in keys. */
  keydown: [nativeEvent: KeyboardEvent];
}>();

defineSlots<{
  /** Primary action contents. Receives the menu open state and availability. */
  default(props: SplitButtonSlotState): unknown;
}>();

const tree = menuTreeContext.use();
const level = menuLevelContext.use();
const splitButton = splitButtonContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabledState = computed(() => disabled || splitButton.disabled.value);

function onClick(event: MouseEvent): void {
  if (!disabledState.value) emit("press", event);
}

function onKeydown(event: KeyboardEvent): void {
  if (disabledState.value) return;
  emit("keydown", event);
  if (event.defaultPrevented) return;
  // Platform split buttons open their menu from the primary action with Alt+ArrowDown.
  if (event.altKey && event.key === "ArrowDown") {
    event.preventDefault();
    splitButton.openFromAction(element.value, event);
  }
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

defineExpose({ element, focus } satisfies Omit<SplitButtonActionExpose, "element"> & {
  readonly element: typeof element;
});
</script>

<template>
  <button
    ref="element"
    :type
    :disabled="disabledState"
    :aria-label
    data-vize-ui="split-button-action"
    part="action"
    :data-state="level.state.value"
    :data-disabled="disabledState ? 'true' : undefined"
    @click="onClick"
    @keydown="onKeydown"
  >
    <slot
      :dir="tree.dir.value"
      :disabled="disabledState"
      :modal="tree.modal.value"
      :open="level.open.value"
      :state="level.state.value"
    />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

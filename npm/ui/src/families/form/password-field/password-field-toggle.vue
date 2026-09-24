<script setup lang="ts">
import { computed } from "vue";

import { passwordFieldContext } from "./password-field-context.ts";

const { showLabel = "Show password", hideLabel = "Hide password" } = defineProps<{
  /**
   * Accessible name while the password is hidden.
   *
   * @default "Show password"
   */
  readonly showLabel?: string;

  /**
   * Accessible name while the password is visible.
   *
   * @default "Hide password"
   */
  readonly hideLabel?: string;
}>();

defineSlots<{
  /** Toggle contents, typically an eye icon, with the current visibility. */
  default?(props: { readonly visible: boolean }): unknown;
}>();

const context = passwordFieldContext.use();
const label = computed(() => (context.visible.value ? hideLabel : showLabel));

function onPointerdown(event: PointerEvent): void {
  // Keep focus (and the caret) in the input while toggling with a pointer.
  if (event.button === 0) event.preventDefault();
}

function onClick(): void {
  context.setVisible(!context.visible.value);
  context.focusInput();
}
</script>

<template>
  <button
    type="button"
    :aria-label="label"
    :aria-pressed="context.visible.value ? 'true' : 'false'"
    :aria-controls="context.inputId.value"
    :disabled="context.disabled.value"
    part="toggle"
    data-vize-ui="password-field-toggle"
    :data-visible="context.visible.value ? 'true' : 'false'"
    @pointerdown="onPointerdown"
    @click="onClick"
  >
    <slot :visible="context.visible.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

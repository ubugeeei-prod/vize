<script setup lang="ts">
import { swipeActionsContext } from "./swipe-actions-context.ts";

const { value, closeOnSelect = true } = defineProps<{
  /**
   * Action identifier reported by `select`.
   *
   * @default required
   */
  readonly value: string;

  /**
   * Close the tray after the action runs.
   *
   * @default true
   */
  readonly closeOnSelect?: boolean;
}>();

const emit = defineEmits<{
  /** Fired when the action button is activated, with its value and the native click. */
  select: [value: string, nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Action label or icon. */
  default(props: Record<string, never>): unknown;
}>();

const context = swipeActionsContext.use();

function onClick(event: MouseEvent): void {
  emit("select", value, event);
  if (closeOnSelect) context.close();
}
</script>

<template>
  <button
    type="button"
    part="action"
    data-vize-ui="swipe-actions-action"
    :data-value="value"
    @click="onClick"
  >
    <slot />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

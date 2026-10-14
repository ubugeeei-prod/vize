<script setup lang="ts">
import { navigationRailContext } from "./navigation-rail-context.ts";

const { disabled = false, ariaLabel = undefined } = defineProps<{
  /**
   * Disable expanding and collapsing.
   *
   * @default false
   */
  readonly disabled?: boolean;

  /**
   * Accessible name when the toggle renders only an icon.
   *
   * @default undefined
   */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Toggle contents with the expanded state. */
  default(props: { readonly expanded: boolean }): unknown;
}>();

const context = navigationRailContext.use();

function onClick(): void {
  if (!disabled) context.setExpanded(!context.expanded.value);
}
</script>

<template>
  <button
    type="button"
    :disabled
    :aria-label
    :aria-expanded="context.expanded.value ? 'true' : 'false'"
    :aria-controls="context.id.value"
    part="toggle"
    data-vize-ui="navigation-rail-toggle"
    :data-state="context.expanded.value ? 'expanded' : 'collapsed'"
    @click="onClick"
  >
    <slot :expanded="context.expanded.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

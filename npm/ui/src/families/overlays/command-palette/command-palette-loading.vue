<script setup lang="ts">
import { commandPaletteContext } from "./command-palette-context.ts";

const { label = "Loading results" } = defineProps<{
  /**
   * Accessible name of the loading indicator.
   *
   * @default "Loading results"
   */
  readonly label?: string;
}>();

defineSlots<{
  /** Loading indicator contents. */
  default?(): unknown;
}>();

const context = commandPaletteContext.use();
</script>

<template>
  <div
    role="progressbar"
    :aria-label="label"
    :hidden="context.loading.value ? undefined : true"
    data-vize-ui="command-palette-loading"
    part="loading"
  >
    <slot />
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

<script setup lang="ts">
import { pullToRefreshContext } from "./pull-to-refresh-context.ts";

defineSlots<{
  /** Button contents; the keyboard and assistive-technology alternative to pulling. */
  default(props: { readonly refreshing: boolean }): unknown;
}>();

const context = pullToRefreshContext.use();

function onClick(): void {
  context.trigger();
}
</script>

<template>
  <button
    type="button"
    :disabled="context.disabled.value || context.refreshing.value"
    :aria-controls="context.rootId.value"
    part="trigger"
    data-vize-ui="pull-to-refresh-trigger"
    @click="onClick"
  >
    <slot :refreshing="context.refreshing.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

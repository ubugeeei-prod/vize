<script setup lang="ts">
import type { DescriptionsProps } from "./types.ts";

const { label } = defineProps<DescriptionsProps>();

defineSlots<{
  /** Renders `DescriptionsItem` pairs. */
  default(): unknown;
}>();
</script>

<template>
  <dl data-vize-ui="descriptions" role="group" :aria-label="label">
    <slot />
  </dl>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

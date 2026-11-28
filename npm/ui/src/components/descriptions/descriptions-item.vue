<script setup lang="ts">
import type { DescriptionsItemProps, DescriptionsItemSlotProps } from "./types.ts";

const { label } = defineProps<DescriptionsItemProps>();

defineSlots<{
  /** Renders the detail for this term. */
  default(props: DescriptionsItemSlotProps): unknown;
}>();
</script>

<template>
  <div data-vize-ui="descriptions-item" role="group" :aria-label="label">
    <dt data-part="term" role="term">{{ label }}</dt>
    <dd data-part="detail" role="definition">
      <slot :label />
    </dd>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

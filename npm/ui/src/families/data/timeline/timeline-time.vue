<script setup lang="ts">
import { computed } from "vue";

const { datetime } = defineProps<{
  /** Machine-readable date, time, or duration for the native `<time>` element. @default required */
  readonly datetime: string | Date;
}>();

defineSlots<{
  /** Human-readable, consumer-formatted time text. Receives the ISO datetime string. */
  default(props: {
    /** Machine-readable datetime rendered into the `datetime` attribute. */
    readonly datetime: string;
  }): unknown;
}>();

// Dates serialize through toISOString so server and client render the same attribute.
const machineValue = computed(() =>
  typeof datetime === "string" ? datetime : datetime.toISOString(),
);
</script>

<template>
  <time :datetime="machineValue" data-vize-ui="timeline-time" part="time">
    <slot :datetime="machineValue" />
  </time>
</template>

<style scoped>
/* Headless by design. Formatting and styling remain consumer-owned. */
</style>

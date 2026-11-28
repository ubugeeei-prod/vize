<script setup lang="ts">
import type { ResultProps, ResultSlotProps, ResultStatus } from "./types.ts";

const { status = "info", title, description = undefined } = defineProps<ResultProps>();

defineSlots<{
  /** Renders follow-up actions. Receives the current status. */
  default(props: ResultSlotProps): unknown;
}>();

function roleFor(value: ResultStatus): "alert" | "status" {
  return value === "error" ? "alert" : "status";
}
</script>

<template>
  <div data-vize-ui="result" :role="roleFor(status)" :data-status="status" :aria-label="title">
    <p data-part="title">{{ title }}</p>
    <p v-if="description" data-part="description">{{ description }}</p>
    <div data-part="actions">
      <slot :status />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

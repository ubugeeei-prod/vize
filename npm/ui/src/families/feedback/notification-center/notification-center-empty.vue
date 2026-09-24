<script setup lang="ts">
import { notificationCenterContext } from "./notification-center-context.ts";

defineSlots<{
  /** Empty-state contents, shown only while no active notification exists. */
  default?(): unknown;
}>();

const context = notificationCenterContext.use();
</script>

<template>
  <p
    :hidden="context.store.notifications.value.length === 0 ? undefined : true"
    data-vize-ui="notification-center-empty"
    part="empty"
  >
    <slot>No notifications</slot>
  </p>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

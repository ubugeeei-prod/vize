<script setup lang="ts">
import { mediaPlayerContext } from "./media-player-context.ts";
import type { MediaPlayerSlotState } from "./media-player-types.ts";

defineSlots<{
  /** Indicator content, rendered only while loading. Defaults to `messages.loading`. */
  default(props: MediaPlayerSlotState): unknown;
}>();

const context = mediaPlayerContext.use();
</script>

<template>
  <span
    role="status"
    data-vize-ui="media-player-loading-indicator"
    part="loading-indicator"
    :data-loading="context.slotState.value.loading ? 'true' : undefined"
  >
    <template v-if="context.slotState.value.loading">
      <slot v-bind="context.slotState.value">{{ context.messages.value.loading }}</slot>
    </template>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. The polite live
   region stays mounted so announcements are not lost. */
</style>

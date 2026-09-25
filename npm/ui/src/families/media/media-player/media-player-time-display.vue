<script setup lang="ts">
import { computed } from "vue";

import { mediaPlayerContext } from "./media-player-context.ts";
import { formatMediaTime } from "./media-player-format.ts";
import type { MediaPlayerSlotState, MediaPlayerTimeDisplayMode } from "./media-player-types.ts";

const { mode = "current" } = defineProps<{
  /**
   * Which time to show: the position, the duration, or the time remaining
   * (rendered with a leading minus sign).
   *
   * @default "current"
   */
  readonly mode?: MediaPlayerTimeDisplayMode;
}>();

defineSlots<{
  /** Custom rendering. Receives the formatted text, raw seconds, and playback state. */
  default(
    props: MediaPlayerSlotState & { readonly text: string; readonly seconds: number },
  ): unknown;
}>();

const context = mediaPlayerContext.use();
const seconds = computed(() => {
  const { currentTime, duration } = context.slotState.value;
  if (mode === "duration") return duration;
  if (mode === "remaining") return -Math.max(0, duration - currentTime);
  return currentTime;
});
const text = computed(() => formatMediaTime(seconds.value, context.slotState.value.duration));
const slotState = computed(() => ({
  ...context.slotState.value,
  seconds: seconds.value,
  text: text.value,
}));
</script>

<template>
  <span data-vize-ui="media-player-time-display" part="time-display" :data-mode="mode">
    <slot v-bind="slotState">{{ text }}</slot>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

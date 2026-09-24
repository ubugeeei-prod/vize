<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { mediaPlayerContext } from "./media-player-context.ts";
import type { MediaPlayerButtonExpose, MediaPlayerSlotState } from "./media-player-types.ts";

const { ariaLabel = undefined } = defineProps<{
  /**
   * Accessible name override. `undefined` uses the state-dependent label from the
   * root `messages`; `null` omits `aria-label` so visible slot text names the button.
   *
   * @default undefined
   */
  readonly ariaLabel?: string | null;
}>();

const emit = defineEmits<{
  /** Fired before the action. Call `preventDefault()` to skip it. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content, e.g. an icon. Receives the playback state. */
  default(props: MediaPlayerSlotState): unknown;
}>();

const context = mediaPlayerContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const disabled = computed(() => context.media.value === null);
const defaultLabel = computed(() =>
  context.state.value === "playing"
    ? context.messages.value.pause
    : context.state.value === "ended"
      ? context.messages.value.replay
      : context.messages.value.play,
);
const label = computed(() =>
  ariaLabel === undefined ? defaultLabel.value : (ariaLabel ?? undefined),
);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (event.defaultPrevented || disabled.value) return;
  void context.togglePlay();
}

type SetupExpose = Omit<MediaPlayerButtonExpose, "disabled" | "element"> & {
  readonly disabled: ComputedRef<boolean>;
  readonly element: typeof element;
};

const exposed = { disabled, element } satisfies SetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="label"
    :aria-controls="context.media.value?.id || undefined"
    data-vize-ui="media-player-play-button"
    part="play-button"
    :data-state="context.state.value"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

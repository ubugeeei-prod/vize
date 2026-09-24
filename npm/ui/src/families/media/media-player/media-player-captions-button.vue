<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { mediaPlayerContext } from "./media-player-context.ts";
import type { MediaPlayerButtonExpose, MediaPlayerSlotState } from "./media-player-types.ts";

import { isCaptionTrack } from "./media-player-format.ts";

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
const disabled = computed(() =>
  context.slotState.value.textTracks.every((track) => !isCaptionTrack(track)),
);
const defaultLabel = computed(() =>
  context.slotState.value.captionTrack >= 0
    ? context.messages.value.hideCaptions
    : context.messages.value.showCaptions,
);
const label = computed(() =>
  ariaLabel === undefined ? defaultLabel.value : (ariaLabel ?? undefined),
);

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (event.defaultPrevented || disabled.value) return;
  context.toggleCaptions();
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
    data-vize-ui="media-player-captions-button"
    part="captions-button"
    :data-state="context.state.value"
    :data-captions="context.slotState.value.captionTrack >= 0 ? 'true' : undefined"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

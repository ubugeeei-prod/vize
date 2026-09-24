<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { mediaPlayerContext } from "./media-player-context.ts";
import { MEDIA_PLAYER_DEFAULT_RATES } from "./media-player-format.ts";
import type { MediaPlayerButtonExpose, MediaPlayerSlotState } from "./media-player-types.ts";

const { ariaLabel = undefined, rates = MEDIA_PLAYER_DEFAULT_RATES } = defineProps<{
  /**
   * Accessible name override. `undefined` uses the state-dependent label from the
   * root `messages`; `null` omits `aria-label` so visible slot text names the button.
   *
   * @default undefined
   */
  readonly ariaLabel?: string | null;

  /**
   * Rates cycled on click, in order. The next rate after the current one is chosen;
   * an unknown current rate selects the first.
   *
   * @default [0.5, 1, 1.25, 1.5, 2]
   */
  readonly rates?: readonly number[];
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
const disabled = computed(() => context.media.value === null || rates.length === 0);
const defaultLabel = computed(() =>
  context.messages.value.playbackRate(context.slotState.value.playbackRate),
);
const label = computed(() =>
  ariaLabel === undefined ? defaultLabel.value : (ariaLabel ?? undefined),
);

function nextRate(current: number): number {
  const index = rates.indexOf(current);
  return rates[(index + 1) % rates.length] ?? current;
}

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (event.defaultPrevented || disabled.value) return;
  context.setPlaybackRate(nextRate(context.slotState.value.playbackRate));
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
    data-vize-ui="media-player-playback-rate-button"
    part="playback-rate-button"
    :data-state="context.state.value"
    :data-rate="context.slotState.value.playbackRate"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

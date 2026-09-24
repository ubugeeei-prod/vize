<script setup lang="ts">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import { mediaPlayerContext } from "../media-player/media-player-context.ts";
import type {
  MediaPlayerButtonExpose,
  MediaPlayerSlotState,
} from "../media-player/media-player-types.ts";
import type { VideoPlayerUnsupportedBehavior } from "./video-player-types.ts";

const { ariaLabel = undefined, unsupported = "disable" } = defineProps<{
  /**
   * Accessible name override. `undefined` uses the state-dependent label from the
   * root `messages`; `null` omits `aria-label` so visible slot text names the button.
   *
   * @default undefined
   */
  readonly ariaLabel?: string | null;

  /**
   * Behavior when the platform lacks support, detected after mount (the server
   * always renders the unsupported state): disable the button or hide it.
   *
   * @default "disable"
   */
  readonly unsupported?: VideoPlayerUnsupportedBehavior;
}>();

const emit = defineEmits<{
  /** Fired before the request. Call `preventDefault()` to skip it. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button content, e.g. an icon. Receives the playback state. */
  default(props: MediaPlayerSlotState): unknown;
}>();

const context = mediaPlayerContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const supported = computed(() => context.fullscreenSupported.value);
const disabled = computed(() => !supported.value);
const active = computed(() => context.slotState.value.fullscreen);
const label = computed(() => {
  if (ariaLabel !== undefined) return ariaLabel ?? undefined;
  return active.value
    ? context.messages.value.exitFullscreen
    : context.messages.value.enterFullscreen;
});

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented && supported.value) void context.toggleFullscreen();
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
    :hidden="!supported && unsupported === 'hide' ? true : undefined"
    :aria-label="label"
    data-vize-ui="video-player-fullscreen-button"
    part="fullscreen-button"
    :data-active="active ? 'true' : undefined"
    :data-supported="supported ? 'true' : 'false'"
    @click="onClick"
  >
    <slot v-bind="context.slotState.value" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

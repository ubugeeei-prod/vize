<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, useTemplateRef, watch } from "vue";
import type { ComputedRef } from "vue";

import { imageContext } from "./image-context.ts";
import type { ImagePartSlotState, ImagePlaceholderExpose } from "./image-types.ts";

const { delay = 0 } = defineProps<{
  /**
   * Milliseconds the image must stay pending before the placeholder renders, so
   * cached images do not flash a skeleton. Positive delays render nothing on the server.
   *
   * @default 0
   */
  readonly delay?: number;
}>();

defineSlots<{
  /** Placeholder content such as a skeleton, blur-up preview, or spinner. */
  default(props: ImagePartSlotState): unknown;
}>();

const context = imageContext.use();
const element = useTemplateRef<HTMLSpanElement>("element");
const pending = computed(
  () => context.status.value === "idle" || context.status.value === "loading",
);
const elapsed = shallowRef(delay <= 0);
let timer: ReturnType<typeof setTimeout> | undefined;

function clearTimer(): void {
  if (timer !== undefined) clearTimeout(timer);
  timer = undefined;
}

function schedule(isPending: boolean): void {
  clearTimer();
  if (!isPending || delay <= 0) {
    elapsed.value = delay <= 0;
    return;
  }
  elapsed.value = false;
  timer = setTimeout(() => {
    timer = undefined;
    elapsed.value = true;
  }, delay);
}

// Timers start only on the client, so positive delays never render on the server.
onMounted(() => schedule(pending.value));
watch(pending, schedule);

onScopeDispose(clearTimer);

const visible = computed(() => pending.value && elapsed.value);

type ImagePlaceholderSetupExpose = Omit<ImagePlaceholderExpose, "element" | "visible"> & {
  readonly element: typeof element;
  readonly visible: ComputedRef<boolean>;
};

const exposed = { element, visible } satisfies ImagePlaceholderSetupExpose;

defineExpose(exposed);
</script>

<template>
  <!-- eslint-disable vue/no-root-v-if -->
  <span
    v-if="visible"
    ref="element"
    aria-hidden="true"
    data-vize-ui="image-placeholder"
    part="placeholder"
    :data-status="context.status.value"
  >
    <slot v-bind="context.slotState.value" />
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

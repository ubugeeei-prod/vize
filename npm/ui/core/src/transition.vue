<script setup lang="ts">
import { computed, onMounted, onUpdated, toRef, useTemplateRef } from "vue";

import { useTransition } from "./transition-runtime.ts";
import type { PresenceStatus } from "./presence-types.ts";

const {
  present = false,
  forceMount = false,
  respectReducedMotion = true,
  timeoutPadding = 0,
  motion = undefined,
} = defineProps<{
  /**
   * Whether the content should occupy the tree.
   *
   * @default false
   */
  readonly present?: boolean;

  /**
   * Keep the slot mounted even while presence would otherwise unmount it.
   *
   * @default false
   */
  readonly forceMount?: boolean;

  /**
   * Skip enter and exit animation when `prefers-reduced-motion: reduce` matches.
   *
   * @default true
   */
  readonly respectReducedMotion?: boolean;

  /**
   * Extra milliseconds added to the computed motion duration before auto-complete.
   *
   * @default 0
   */
  readonly timeoutPadding?: number;

  /**
   * Named enter/exit recipe published on `data-vize-motion`, pairing the host
   * with the packaged motion stylesheet (`@vizejs/ui/motion`).
   *
   * @default undefined
   */
  readonly motion?: "fade" | "scale" | "slide";
}>();

defineSlots<{
  /** Transitioning contents. Receives status for styling hooks. */
  default(props: { readonly present: boolean; readonly status: PresenceStatus }): unknown;
}>();

const transition = useTransition({
  present: toRef(() => present),
  respectReducedMotion: toRef(() => respectReducedMotion),
  timeoutPadding: toRef(() => timeoutPadding),
});
const show = computed(() => transition.isPresent.value || forceMount);
const element = useTemplateRef<HTMLDivElement>("element");

onMounted(() => {
  transition.setElement(element.value);
});

onUpdated(() => {
  transition.setElement(element.value);
});

defineExpose({
  completeAnimation: transition.completeAnimation,
  element,
  isPresent: transition.isPresent,
  status: transition.status,
});
</script>

<template>
  <div data-vize-ui="transition-host">
    <div
      v-if="show"
      ref="element"
      data-vize-ui="transition"
      :data-vize-motion="motion"
      :data-vize-transition="transition.status.value"
      v-bind="transition.presenceProps"
    >
      <slot :present="transition.isPresent.value" :status="transition.status.value" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

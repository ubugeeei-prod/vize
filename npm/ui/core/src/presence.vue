<script setup lang="ts">
import { computed, toRef, useTemplateRef } from "vue";

import { usePresence } from "./presence-runtime.ts";
import type { PresenceStatus } from "./presence-types.ts";

const {
  present = false,
  forceMount = false,
  respectReducedMotion = true,
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
}>();

defineSlots<{
  /** Present contents. Receives presence status for styling hooks. */
  default(props: { readonly present: boolean; readonly status: PresenceStatus }): unknown;
}>();

const presence = usePresence({
  present: toRef(() => present),
  respectReducedMotion: toRef(() => respectReducedMotion),
});
const show = computed(() => presence.isPresent.value || forceMount);
const element = useTemplateRef<HTMLDivElement>("element");

defineExpose({
  element,
  isPresent: presence.isPresent,
  status: presence.status,
  completeAnimation: presence.completeAnimation,
});
</script>

<template>
  <div data-vize-ui="presence-host">
    <div
      v-if="show"
      ref="element"
      data-vize-ui="presence"
      :data-vize-presence="presence.status.value"
      v-bind="presence.presenceProps"
    >
      <slot :present="presence.isPresent.value" :status="presence.status.value" />
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

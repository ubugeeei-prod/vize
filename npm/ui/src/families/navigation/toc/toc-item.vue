<script setup lang="ts">
import { computed, shallowRef } from "vue";

import { tocContext, tocItemContext } from "./toc-context.ts";
import type { TocLinkSlotState } from "./toc-types.ts";

const { targetId: explicitTargetId = undefined } = defineProps<{
  /**
   * Target id of this entry. Supply it for exact server-rendered `data-active`;
   * otherwise it is learned from the nested TocLink after it renders.
   *
   * @default undefined
   */
  readonly targetId?: string;
}>();

defineSlots<{
  /** TocLink and optional nested TocList. Receives the link target and active state. */
  default(props: TocLinkSlotState): unknown;
}>();

const context = tocContext.use();
const learnedTargetId = shallowRef<string | null>(null);
const targetId = computed(() => explicitTargetId ?? learnedTargetId.value);
const active = computed(() => targetId.value !== null && context.activeId.value === targetId.value);
const slotState = computed<TocLinkSlotState>(() => ({
  active: active.value,
  targetId: targetId.value ?? "",
}));

tocItemContext.provide({
  setTargetId: (next) => {
    learnedTargetId.value = next;
  },
});
</script>

<template>
  <li data-vize-ui="toc-item" part="item" :data-active="active ? 'true' : undefined">
    <slot v-bind="slotState" />
  </li>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

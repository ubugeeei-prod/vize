<script setup lang="ts" generic="Item">
import { computed, useTemplateRef } from "vue";
import type { ComputedRef } from "vue";

import Avatar from "../avatar/avatar.vue";
import { resolveAvatarGroupMessages } from "./avatar-group-state.ts";
import type {
  AvatarGroupMessageOverrides,
  AvatarGroupOverflowExpose,
  AvatarGroupOverflowSlotState,
} from "./avatar-group-types.ts";

const {
  count,
  items = undefined,
  messages = undefined,
} = defineProps<{
  /** Number of hidden people announced by the tile. @default required */
  readonly count: number;

  /**
   * Hidden items forwarded to the slot, e.g. for a tooltip listing names.
   *
   * @default undefined
   */
  readonly items?: readonly Item[];

  /**
   * Localized overflow strings.
   *
   * @default undefined
   */
  readonly messages?: AvatarGroupMessageOverrides | undefined;
}>();

defineSlots<{
  /** Overflow tile content. Defaults to the `overflowText` message, e.g. "+3". */
  default?(props: AvatarGroupOverflowSlotState<Item>): unknown;
}>();

const element = useTemplateRef<HTMLSpanElement>("element");
const hiddenCount = computed(() => (Number.isFinite(count) ? Math.max(0, Math.floor(count)) : 0));
const resolved = computed(() => resolveAvatarGroupMessages(messages));
const label = computed(() => resolved.value.overflow(hiddenCount.value));
const text = computed(() => resolved.value.overflowText(hiddenCount.value));
const slotState = computed<AvatarGroupOverflowSlotState<Item>>(() => ({
  count: hiddenCount.value,
  hiddenItems: items ?? [],
  label: label.value,
  text: text.value,
}));

type AvatarGroupOverflowSetupExpose = Omit<AvatarGroupOverflowExpose, "count" | "element"> & {
  readonly count: ComputedRef<number>;
  readonly element: typeof element;
};

const exposed = { count: hiddenCount, element } satisfies AvatarGroupOverflowSetupExpose;

defineExpose(exposed);
</script>

<template>
  <span
    ref="element"
    role="img"
    :aria-label="label"
    data-vize-ui="avatar-group-overflow"
    part="overflow"
    :data-count="hiddenCount"
  >
    <Avatar aria-hidden="true" :fallback="text">
      <slot v-bind="slotState">{{ text }}</slot>
    </Avatar>
  </span>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

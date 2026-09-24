<script setup lang="ts">
import { computed, useTemplateRef, watch } from "vue";

import {
  navigationMenuContentContext,
  navigationMenuContext,
  navigationMenuItemContext,
} from "./navigation-menu-context.ts";
import type { NavigationMenuItemSlotState, NavigationMenuMotion } from "./navigation-menu-types.ts";

const { forceMount = false } = defineProps<{
  /**
   * Keep closed flyout content rendered (inside the hidden container) for exit animations
   * or crawlable links. Without it, closed flyouts keep an empty hidden container.
   *
   * @default false
   */
  readonly forceMount?: boolean;
}>();

defineSlots<{
  /** Flyout content, usually NavigationMenuLink lists. Receives the open state. */
  default(props: NavigationMenuItemSlotState): unknown;
}>();

const context = navigationMenuContext.use();
const item = navigationMenuItemContext.use();
const element = useTemplateRef<HTMLDivElement>("element");
const contentId = computed(() => context.getContentId(item.value.value));
const triggerId = computed(() => context.getTriggerId(item.value.value));
const present = computed(() => forceMount || item.open.value);
const motion = computed<NavigationMenuMotion | null>(() => context.getMotion(item.value.value));
const slotState = computed<NavigationMenuItemSlotState>(() => ({
  open: item.open.value,
  state: item.open.value ? "open" : "closed",
  value: item.value.value,
}));

watch(
  item.value,
  (value, _previous, onCleanup) => onCleanup(context.registerContent(value, element)),
  {
    flush: "sync",
    immediate: true,
  },
);

navigationMenuContentContext.provide({ value: item.value });

function onPointerenter(event: PointerEvent): void {
  if (event.pointerType === "mouse" || event.pointerType === "pen") context.onContentEnter();
}

function onPointerleave(event: PointerEvent): void {
  if (event.pointerType === "mouse" || event.pointerType === "pen") context.onPointerLeave();
}

const contentProps = computed<{
  readonly onPointerenter: (event: PointerEvent) => void;
  readonly onPointerleave: (event: PointerEvent) => void;
}>(() => ({ onPointerenter, onPointerleave }));
</script>

<template>
  <div
    v-bind="contentProps"
    :id="contentId"
    ref="element"
    :hidden="item.open.value ? undefined : true"
    :data-mounted="present ? 'true' : undefined"
    :aria-labelledby="triggerId"
    data-vize-ui="navigation-menu-content"
    part="content"
    :data-state="item.open.value ? 'open' : 'closed'"
    :data-motion="motion ?? undefined"
    :data-orientation="context.orientation.value"
  >
    <slot v-if="present" v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Positioning and motion remain consumer-owned. */
</style>

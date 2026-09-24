<script setup lang="ts">
import { computed, onScopeDispose, useTemplateRef, watch } from "vue";

import type { CollectionRegistration } from "../../foundations/collection/collection.ts";
import { navigationMenuContext, navigationMenuItemContext } from "./navigation-menu-context.ts";
import type { NavigationMenuItemSlotState } from "./navigation-menu-types.ts";

const { value } = defineProps<{
  /** Stable item value used as the open value of its flyout. @default required */
  readonly value: string;
}>();

defineSlots<{
  /** Trigger, content, or a plain link. Receives the open state of this item. */
  default(props: NavigationMenuItemSlotState): unknown;
}>();

const context = navigationMenuContext.use();
const element = useTemplateRef<HTMLLIElement>("element");
const valueState = computed(() => value);
const open = computed(() => context.value.value === value);
const slotState = computed<NavigationMenuItemSlotState>(() => ({
  open: open.value,
  state: open.value ? "open" : "closed",
  value,
}));
let registration: CollectionRegistration<string> | null = null;

watch(
  valueState,
  (next) => {
    registration?.unregister();
    registration = context.registerItem({ element, value: next });
  },
  { flush: "sync", immediate: true },
);
onScopeDispose(() => {
  registration?.unregister();
  registration = null;
});

navigationMenuItemContext.provide({ open, value: valueState });
</script>

<template>
  <li
    ref="element"
    data-vize-ui="navigation-menu-item"
    part="item"
    :data-state="open ? 'open' : 'closed'"
    :data-value="value"
  >
    <slot v-bind="slotState" />
  </li>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, watch } from "vue";

import { navigationMenuContext } from "./navigation-menu-context.ts";
import type {
  NavigationMenuMeasuredSlotState,
  NavigationMenuOpenState,
} from "./navigation-menu-types.ts";

defineSlots<{
  /** Optional backdrop content. Receives the open value. */
  default?(props: NavigationMenuMeasuredSlotState): unknown;
}>();

interface ViewportSize {
  readonly width: number;
  readonly height: number;
}

const context = navigationMenuContext.use();
const size = shallowRef<ViewportSize | null>(null);
const state = computed<NavigationMenuOpenState>(() =>
  context.value.value === null ? "closed" : "open",
);
const style = computed(() =>
  size.value === null
    ? undefined
    : {
        "--vize-navigation-menu-viewport-height": `${size.value.height}px`,
        "--vize-navigation-menu-viewport-width": `${size.value.width}px`,
      },
);
const slotState = computed<NavigationMenuMeasuredSlotState>(() => ({
  state: state.value,
  value: context.value.value,
}));

let observer: ResizeObserver | null = null;

function measure(): void {
  const content = context.getContentElement(context.value.value);
  observer?.disconnect();
  if (content === null) {
    size.value = null;
    return;
  }
  const rect = content.getBoundingClientRect();
  size.value = { height: rect.height, width: rect.width };
  observer?.observe(content);
}

// The viewport is a measured backdrop: flyouts stay inline after their trigger so the
// tab order is native, and consumers size a shared surface from these CSS variables.
onMounted(() => {
  if (typeof ResizeObserver === "function") observer = new ResizeObserver(() => measure());
  watch(context.value, measure, { flush: "post", immediate: true });
});
onScopeDispose(() => {
  observer?.disconnect();
  observer = null;
});
</script>

<template>
  <div
    aria-hidden="true"
    :style="style"
    data-vize-ui="navigation-menu-viewport"
    part="viewport"
    :data-state="state"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="slotState" />
  </div>
</template>

<style scoped>
/* Headless by design. Consumers size a shared flyout surface from its CSS variables. */
</style>

<script setup lang="ts">
import { computed, onMounted, onScopeDispose, shallowRef, watch } from "vue";

import { navigationMenuContext } from "./navigation-menu-context.ts";
import type {
  NavigationMenuMeasuredSlotState,
  NavigationMenuOpenState,
} from "./navigation-menu-types.ts";

defineSlots<{
  /** Optional indicator content such as an arrow. Receives the open value. */
  default?(props: NavigationMenuMeasuredSlotState): unknown;
}>();

interface IndicatorBox {
  readonly offset: number;
  readonly size: number;
}

const context = navigationMenuContext.use();
const box = shallowRef<IndicatorBox | null>(null);
const state = computed<NavigationMenuOpenState>(() =>
  context.value.value === null ? "closed" : "open",
);
const style = computed(() =>
  box.value === null
    ? undefined
    : {
        "--vize-navigation-menu-indicator-offset": `${box.value.offset}px`,
        "--vize-navigation-menu-indicator-size": `${box.value.size}px`,
      },
);
const slotState = computed<NavigationMenuMeasuredSlotState>(() => ({
  state: state.value,
  value: context.value.value,
}));

// Measurement is client-only; the server renders the indicator without geometry.
function measure(): void {
  const trigger = context.getTriggerElement(context.value.value);
  if (trigger === null) {
    box.value = null;
    return;
  }
  const horizontal = context.orientation.value === "horizontal";
  box.value = {
    offset: horizontal ? trigger.offsetLeft : trigger.offsetTop,
    size: horizontal ? trigger.offsetWidth : trigger.offsetHeight,
  };
}

let stopResize: (() => void) | null = null;
onMounted(() => {
  watch([context.value, context.orientation], measure, { flush: "post", immediate: true });
  const view = typeof window === "undefined" ? null : window;
  view?.addEventListener("resize", measure, { passive: true });
  stopResize = () => view?.removeEventListener("resize", measure);
});
onScopeDispose(() => stopResize?.());
</script>

<template>
  <span
    aria-hidden="true"
    :style="style"
    data-vize-ui="navigation-menu-indicator"
    part="indicator"
    :data-state="state"
    :data-orientation="context.orientation.value"
  >
    <slot v-bind="slotState" />
  </span>
</template>

<style scoped>
/* Headless by design. Consumers position the indicator from its CSS variables. */
</style>

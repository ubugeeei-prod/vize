<script setup lang="ts">
import { computed } from "vue";

import { windowManagerContext } from "./window-manager-context.ts";
import type { WindowDockFilter, WindowDockSlotState } from "./window-manager-types.ts";

const { label = "Windows", filter = "all" } = defineProps<{
  /**
   * Accessible name of the dock navigation.
   *
   * @default "Windows"
   */
  readonly label?: string;

  /**
   * List every window, or only minimized ones.
   *
   * @default "all"
   */
  readonly filter?: WindowDockFilter;
}>();

defineSlots<{
  /** Custom dock content. Receives the listed window summaries; defaults to one button per window. */
  default?(props: WindowDockSlotState): unknown;
}>();

const context = windowManagerContext.use();
const windows = computed(() =>
  context.windows.value.filter((window) => filter === "all" || window.mode === "minimized"),
);

function toggle(id: string): void {
  const summary = context.windows.value.find((window) => window.id === id);
  if (!summary) return;
  if (summary.active && summary.mode !== "minimized") summary.minimize();
  else summary.activate();
}
</script>

<template>
  <nav :aria-label="label" data-vize-ui="window-dock">
    <slot :windows="windows">
      <button
        v-for="window in windows"
        :key="window.id"
        type="button"
        data-part="dock-item"
        :data-mode="window.mode"
        :aria-pressed="window.active && window.mode !== 'minimized' ? 'true' : 'false'"
        @click="() => toggle(window.id)"
      >
        {{ window.title }}
      </button>
    </slot>
  </nav>
</template>

<style scoped>
/* Headless by design. Dock layout is consumer-owned. */
</style>

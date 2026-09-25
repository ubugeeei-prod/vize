<script setup lang="ts">
import { computed } from "vue";

import { schedulerContext } from "./scheduler-context.ts";

const { action, ariaLabel = undefined } = defineProps<{
  /** Navigation performed by the button. @default undefined (required) */
  readonly action: "previous" | "next" | "today";
  /** Accessible name; defaults to "Previous period", "Next period", or "Today". @default undefined */
  readonly ariaLabel?: string;
}>();

defineSlots<{
  /** Button content. Receives the action. */
  default(props: { readonly action: "previous" | "next" | "today" }): unknown;
}>();

const context = schedulerContext.use();
const label = computed(
  () => ariaLabel ?? { previous: "Previous period", next: "Next period", today: "Today" }[action],
);

function onClick(): void {
  if (action === "today") context.goToToday();
  else context.navigate(action === "previous" ? -1 : 1);
}
</script>

<template>
  <button
    type="button"
    :disabled="!context.canNavigate()"
    :aria-label="label"
    :aria-controls="context.id.value"
    data-vize-ui="scheduler-nav"
    part="nav"
    :data-action="action"
    @click="onClick"
  >
    <slot :action="action">{{
      action === "previous" ? "‹" : action === "next" ? "›" : label
    }}</slot>
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

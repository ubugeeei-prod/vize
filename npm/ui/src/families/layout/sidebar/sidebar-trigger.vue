<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import { sidebarContext } from "./sidebar-context.ts";
import type { SidebarSlotState, SidebarToggleExpose } from "./sidebar-types.ts";

const { ariaLabel = "Toggle sidebar" } = defineProps<{
  /**
   * Accessible name when no visible label supplies one.
   *
   * @default "Toggle sidebar"
   */
  readonly ariaLabel?: string;
}>();

const emit = defineEmits<{
  /** Fired before toggling. Call `preventDefault()` to keep state unchanged. */
  click: [nativeEvent: MouseEvent];
}>();

defineSlots<{
  /** Button contents. Receives the current Sidebar state. */
  default?(props: SidebarSlotState): unknown;
}>();

/* Button that toggles the sidebar (the mobile sheet on mobile). */
const context = sidebarContext.use();
const element = useTemplateRef<HTMLButtonElement>("element");
const expanded = computed(() =>
  context.isMobile.value ? context.openMobile.value : context.open.value,
);
const disabled = computed(() => !context.isMobile.value && context.collapsible.value === "none");
const slotState = computed<SidebarSlotState>(() => ({
  collapsible: context.collapsible.value,
  isMobile: context.isMobile.value,
  open: context.open.value,
  openMobile: context.openMobile.value,
  side: context.side.value,
  state: context.state.value,
  variant: context.variant.value,
}));

function onClick(event: MouseEvent): void {
  emit("click", event);
  if (!event.defaultPrevented) context.toggle(event);
}

function focus(options?: FocusOptions): void {
  element.value?.focus(options);
}

type SidebarToggleSetupExpose = Omit<SidebarToggleExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element, focus } satisfies SidebarToggleSetupExpose;

defineExpose(exposed);
</script>

<template>
  <button
    ref="element"
    type="button"
    :disabled
    :aria-label="ariaLabel"
    :aria-expanded="expanded ? 'true' : 'false'"
    :aria-controls="context.sidebarId.value"
    data-vize-ui="sidebar-trigger"
    part="trigger"
    :data-state="expanded ? 'expanded' : 'collapsed'"
    :data-side="context.side.value"
    @click="onClick"
  >
    <slot v-bind="slotState" />
  </button>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

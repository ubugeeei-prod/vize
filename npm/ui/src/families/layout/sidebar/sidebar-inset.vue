<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import { sidebarContext } from "./sidebar-context.ts";
import type { SidebarSectionExpose } from "./sidebar-types.ts";

const { as = "main" } = defineProps<{
  /**
   * Element or component rendered for the main content beside the sidebar.
   *
   * @default "main"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Main application content. */
  default(): unknown;
}>();

const context = sidebarContext.use();
const element = useTemplateRef<PrimitiveElement>("element");

type SidebarInsetSetupExpose = Omit<SidebarSectionExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SidebarInsetSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="sidebar-inset"
    part="inset"
    :data-state="context.state.value"
    :data-side="context.side.value"
    :data-mobile="context.isMobile.value ? 'true' : undefined"
  >
    <slot />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

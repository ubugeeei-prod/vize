<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import { sidebarContext } from "./sidebar-context.ts";
import type { SidebarSectionExpose } from "./sidebar-types.ts";

const { as = "div" } = defineProps<{
  /**
   * Element or component to render.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Sticky top area of the sidebar. */
  default(): unknown;
}>();

const context = sidebarContext.use();
const element = useTemplateRef<PrimitiveElement>("element");

type SidebarSectionSetupExpose = Omit<SidebarSectionExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SidebarSectionSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    ref="element"
    data-vize-ui="sidebar-header"
    part="header"
    :data-state="context.state.value"
  >
    <slot />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

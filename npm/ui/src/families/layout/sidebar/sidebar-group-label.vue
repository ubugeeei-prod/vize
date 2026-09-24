<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { PrimitiveAs, PrimitiveElement } from "../../foundations/primitive/primitive.ts";
import { sidebarContext, sidebarGroupContext } from "./sidebar-context.ts";
import type { SidebarSectionExpose } from "./sidebar-types.ts";

const { as = "div" } = defineProps<{
  /**
   * Element or component to render, for example a heading.
   *
   * @default "div"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Visible group label. */
  default(): unknown;
}>();

const context = sidebarContext.use();
const group = sidebarGroupContext.use();
const element = useTemplateRef<PrimitiveElement>("element");

type SidebarGroupLabelSetupExpose = Omit<SidebarSectionExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies SidebarGroupLabelSetupExpose;

defineExpose(exposed);
</script>

<template>
  <component
    :is="as"
    :id="group.labelId.value"
    ref="element"
    data-vize-ui="sidebar-group-label"
    part="group-label"
    :data-state="context.state.value"
  >
    <slot />
  </component>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

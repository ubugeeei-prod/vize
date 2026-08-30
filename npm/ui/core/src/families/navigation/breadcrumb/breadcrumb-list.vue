<template>
  <component :is="as" ref="element" data-vize-ui="breadcrumb-list" part="list">
    <slot />
  </component>
</template>

<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { BreadcrumbListExpose } from "./breadcrumb-types.ts";
import type { PrimitiveAs, PrimitiveElement } from "../../../primitive.ts";

const { as = "ol" } = defineProps<{
  /**
   * Native element, custom element, or component to render.
   *
   * @default "ol"
   */
  readonly as?: PrimitiveAs;
}>();

defineSlots<{
  /** Renders ordered breadcrumb items. */
  default(): unknown;
}>();

const element = useTemplateRef<PrimitiveElement>("element");

type BreadcrumbListSetupExpose = Omit<BreadcrumbListExpose, "element"> & {
  readonly element: typeof element;
};

const exposed = { element } satisfies BreadcrumbListSetupExpose;

defineExpose(exposed);
</script>

<style scoped>
/* Headless by design. List marker removal, wrapping, and gap are consumer-owned. */
</style>

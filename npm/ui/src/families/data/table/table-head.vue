<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { TableHeadExpose } from "./table-contracts.ts";
import type { TableHeadSlotState } from "./table-types.ts";

defineSlots<{
  /** Renders one or more native table rows inside the head section. */
  default(props: TableHeadSlotState): unknown;
}>();

const element = useTemplateRef<HTMLTableSectionElement>("element");
const section = "head" as const;
const slotState = { section } satisfies TableHeadSlotState;

type TableHeadSetupExpose = Omit<TableHeadExpose, "element" | "section"> & {
  readonly element: typeof element;
  readonly section: typeof section;
};

const exposed = {
  element,
  section,
} satisfies TableHeadSetupExpose;

defineExpose(exposed);
</script>

<template>
  <thead ref="element" data-vize-ui="table-head" part="head" data-section="head">
    <slot v-bind="slotState" />
  </thead>
</template>

<style scoped>
/* Headless by design. Sticky headers, separators, and color remain consumer-owned. */
</style>

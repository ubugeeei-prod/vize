<script setup lang="ts">
import { useTemplateRef } from "vue";

import type { TableBodyExpose } from "./table-contracts.ts";
import type { TableBodySlotState } from "./table-types.ts";

defineSlots<{
  /** Renders one or more native table rows inside the body section. */
  default(props: TableBodySlotState): unknown;
}>();

const element = useTemplateRef<HTMLTableSectionElement>("element");
const section = "body" as const;
const slotState = { section } satisfies TableBodySlotState;

type TableBodySetupExpose = Omit<TableBodyExpose, "element" | "section"> & {
  readonly element: typeof element;
  readonly section: typeof section;
};

const exposed = {
  element,
  section,
} satisfies TableBodySetupExpose;

defineExpose(exposed);
</script>

<template>
  <tbody ref="element" data-vize-ui="table-body" part="body" data-section="body">
    <slot v-bind="slotState" />
  </tbody>
</template>

<style scoped>
/* Headless by design. Row grouping, striping, and empty-state treatment remain consumer-owned. */
</style>

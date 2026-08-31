<script setup lang="ts">
import { computed, useTemplateRef } from "vue";

import type { TableCaptionExpose } from "./table-contracts.ts";
import type {
  TableCaptionProps,
  TableCaptionSide,
  TableCaptionSlotState,
  TableCaptionStyle,
} from "./table-types.ts";

const { side = "top" } = defineProps<TableCaptionProps>();

defineSlots<{
  /** Renders the native caption content with caption placement state. */
  default(props: TableCaptionSlotState): unknown;
}>();

const element = useTemplateRef<HTMLTableCaptionElement>("element");
const sideState = computed<TableCaptionSide>(() => side);
const captionStyle = computed<TableCaptionStyle>(() => ({
  "--vize-ui-table-caption-side": sideState.value,
  captionSide: sideState.value,
}));
const intrinsicProps = computed(() => ({ style: captionStyle.value }));
const slotState = computed<TableCaptionSlotState>(() => ({
  side: sideState.value,
  style: captionStyle.value,
}));

type TableCaptionSetupExpose = Omit<TableCaptionExpose, "element" | "side" | "style"> & {
  readonly element: typeof element;
  readonly side: typeof sideState;
  readonly style: typeof captionStyle;
};

const exposed = {
  element,
  side: sideState,
  style: captionStyle,
} satisfies TableCaptionSetupExpose;

defineExpose(exposed);
</script>

<template>
  <caption
    ref="element"
    data-vize-ui="table-caption"
    part="caption"
    :data-caption-side="sideState"
    v-bind="intrinsicProps"
  >
    <slot v-bind="slotState" />
  </caption>
</template>

<style scoped>
/* Headless by design. Caption alignment, spacing, and visibility remain consumer-owned. */
</style>

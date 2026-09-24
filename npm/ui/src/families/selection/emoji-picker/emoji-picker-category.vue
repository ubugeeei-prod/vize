<script setup lang="ts" generic="T">
import { computed } from "vue";

import { emojiPickerCategoryContext, emojiPickerContext } from "./emoji-picker-context.ts";
import type { EmojiPickerSection } from "./emoji-picker-model.ts";
import type { EmojiPickerCellSlotState } from "./emoji-picker-types.ts";

const { section } = defineProps<{
  /** Section from the root slot state `sections`. @default required */
  readonly section: EmojiPickerSection<T>;
}>();

defineSlots<{
  /** Render one `EmojiPickerItem` per cell with `:item` and `:index`. */
  default?(props: EmojiPickerCellSlotState<T>): unknown;

  /** Custom section label content. */
  label?(props: { readonly label: string }): unknown;
}>();

/** One rendered row with stable keys. */
interface CategoryRow {
  readonly key: string;
  readonly cells: readonly { readonly key: string; readonly props: EmojiPickerCellSlotState<T> }[];
}

const context = emojiPickerContext.use();
const sectionIndex = computed(() =>
  context.sections.value.findIndex((candidate) => candidate.id === section.id),
);
const labelId = computed(() => context.labelId(Math.max(0, sectionIndex.value)));
const rows = computed<readonly CategoryRow[]>(() =>
  section.rows.map((row, rowIndex) => ({
    cells: row.map((item, column) => {
      const index = rowIndex * context.columns.value + column;
      return { key: `${section.id}:${index}`, props: { column, index, item } };
    }),
    key: `${section.id}:row:${rowIndex}`,
  })),
);

emojiPickerCategoryContext.provide({ section: sectionIndex });
</script>

<template>
  <div
    role="rowgroup"
    :aria-labelledby="labelId"
    data-vize-ui="emoji-picker-category"
    part="category"
    :data-category="section.id"
  >
    <div role="row" data-vize-ui="emoji-picker-category-label-row" part="category-label-row">
      <div
        :id="labelId"
        role="columnheader"
        :aria-colspan="context.columns.value"
        part="category-label"
      >
        <slot name="label" :label="section.label">{{ section.label }}</slot>
      </div>
    </div>
    <div
      v-for="row in rows as readonly CategoryRow[]"
      :key="row.key"
      role="row"
      data-vize-ui="emoji-picker-row"
      part="row"
    >
      <template v-for="cell in row.cells" :key="cell.key">
        <slot v-bind="cell.props" />
      </template>
    </div>
  </div>
</template>

<style scoped>
/* Headless by design. Native CSS remains entirely consumer-owned. */
</style>

<script setup lang="ts" generic="T">
import { computed } from "vue";

import { defaultCellFormatter } from "./chart-format.ts";
import type { ChartTableColumn } from "./chart-types.ts";

const {
  data,
  columns,
  caption,
  visuallyHidden = true,
  locale = "en-US",
} = defineProps<{
  /** Rows of the chart. @default required */
  readonly data: readonly T[];

  /** Columns read from each row. @default required */
  readonly columns: readonly ChartTableColumn<T>[];

  /** Table caption, usually the chart title. @default required */
  readonly caption: string;

  /**
   * Keep the table available to assistive technology but visually hidden.
   * Set `false` to show it, for example behind a "Show data" disclosure.
   *
   * @default true
   */
  readonly visuallyHidden?: boolean;

  /**
   * Locale of the default cell formatter.
   *
   * @default "en-US"
   */
  readonly locale?: string;
}>();

const fallback = computed(() => defaultCellFormatter(locale));
const rows = computed(() =>
  data.map((datum, index) => ({
    id: `row-${index}`,
    cells: columns.map((column) => {
      const value = column.value(datum, index);
      return {
        key: column.key,
        rowHeader: column.rowHeader === true,
        text: column.format === undefined ? fallback.value(value) : column.format(value, datum),
      };
    }),
  })),
);
const hiddenStyle = computed(() =>
  visuallyHidden
    ? {
        border: "0",
        clipPath: "inset(50%)",
        height: "1px",
        margin: "-1px",
        overflow: "hidden",
        padding: "0",
        position: "absolute" as const,
        whiteSpace: "nowrap" as const,
        width: "1px",
      }
    : undefined,
);
</script>

<template>
  <table
    :style="hiddenStyle"
    data-vize-ui="chart-data-table"
    part="table"
    :data-visually-hidden="visuallyHidden ? 'true' : undefined"
  >
    <caption>
      {{
        caption
      }}
    </caption>
    <thead>
      <tr>
        <th v-for="column in columns" :key="column.key" scope="col">{{ column.header }}</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="row in rows" :key="row.id">
        <template v-for="cell in row.cells" :key="cell.key">
          <th v-if="cell.rowHeader" scope="row">{{ cell.text }}</th>
          <td v-else>{{ cell.text }}</td>
        </template>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
/* Headless by design. Only the optional visually-hidden clip is inline. */
</style>

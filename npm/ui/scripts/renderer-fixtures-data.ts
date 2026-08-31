/** Data-display component fixtures compiled by every supported renderer lane. */
export const dataRendererFixtures = [
  {
    filename: "TableConsumer.vue",
    source: String.raw`<script setup lang="ts">
import { ref } from "vue";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "./families/data/table/table.ts";

const density = ref<"compact" | "normal" | "spacious">("compact");
</script>

<template>
  <Table layout="fixed" :density>
    <template #default="{ layout }">
      <TableCaption side="bottom">Release health {{ layout }}</TableCaption>
      <TableHead>
        <TableRow>
          <TableHeader id="signal" scope="col">Signal</TableHeader>
          <TableHeader id="value" scope="col" align="end">Value</TableHeader>
        </TableRow>
      </TableHead>
      <TableBody>
        <TableRow state="selected">
          <TableHeader scope="row">CI</TableHeader>
          <TableCell headers="signal value" align="end">green</TableCell>
        </TableRow>
      </TableBody>
    </template>
  </Table>
</template>
`,
  },
] as const;

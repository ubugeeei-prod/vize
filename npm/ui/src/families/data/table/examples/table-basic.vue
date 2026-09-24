<!-- Native data table with caption, column headers, row headers, and end-aligned numbers. -->
<script setup lang="ts">
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "../table.ts";

const rows = [
  { quarter: "Q1", revenue: "$12,400" },
  { quarter: "Q2", revenue: "$15,900" },
  { quarter: "Q3", revenue: "$14,200" },
] as const;
</script>

<template>
  <Table density="compact">
    <TableCaption>Revenue by quarter</TableCaption>
    <TableHead>
      <TableRow>
        <TableHeader scope="col">Quarter</TableHeader>
        <TableHeader scope="col" align="end">Revenue</TableHeader>
      </TableRow>
    </TableHead>
    <TableBody>
      <TableRow v-for="row in rows" :key="row.quarter">
        <TableHeader scope="row">{{ row.quarter }}</TableHeader>
        <TableCell align="end">{{ row.revenue }}</TableCell>
      </TableRow>
    </TableBody>
  </Table>
</template>
